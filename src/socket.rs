use anyhow::Result;
use rust_socketio::{
    asynchronous::{Client, ClientBuilder},
    Event, Payload,
};
use serde_json::json;
use std::{iter::Skip, sync::mpsc::Sender};

use crate::{
    events::AppEvent,
    media::MediaChat,
    ui::{wake, CtxWaker},
};

pub async fn run_socket(
    server_url: String,
    room: String,
    tx: Sender<AppEvent>,
    waker: CtxWaker,
) -> Result<()> {
    let tx_media = tx.clone();
    let tx_flush = tx.clone();
    let tx_skip = tx.clone();
    let waker_media = waker.clone();
    let waker_flush = waker.clone();
    let waker_skip = waker.clone();
    let room_join = room;

    let client = ClientBuilder::new(server_url)
        .on(Event::Connect, move |_, socket: Client| {
            let room = room_join.clone();
            Box::pin(async move {
                match socket.emit("join", json!(room)).await {
                    Ok(_) => log::info!("Joined room '{room}'"),
                    Err(e) => log::error!("join failed: {e}"),
                }
            })
        })
        .on("mediachat", move |payload, _| {
            let tx = tx_media.clone();
            let waker = waker_media.clone();
            Box::pin(async move {
                if let Payload::Text(values) = payload {
                    for val in values {
                        match serde_json::from_value::<MediaChat>(val) {
                            Ok(mc) => {
                                let _ = tx.send(AppEvent::NewMediaChat(Box::new(mc)));
                                wake(&waker);
                            }
                            Err(e) => log::warn!("mediachat parse error: {e}"),
                        }
                    }
                }
            })
        })
        .on(Event::Message, move |payload, _| {
            let tx_flush = tx_flush.clone();
            let waker_flush = waker_flush.clone();
            let tx_skip = tx_skip.clone();
            let waker_skip = waker_skip.clone();
            Box::pin(async move {
                if let Payload::Text(values) = payload {
                    for val in values {
                        match serde_json::from_value::<String>(val) {
                            Ok(text) => match text.as_str() {
                                "flush" => {
                                    let _ = tx_flush.send(AppEvent::Flush);
                                    wake(&waker_flush);
                                }
                                "skip" => {
                                    let _ = tx_skip.send(AppEvent::Skip);
                                    wake(&waker_skip);
                                }
                                unkown => log::warn!("unkown socket message {}", unkown),
                            },
                            Err(e) => log::warn!("socket message parse error: {e}"),
                        }
                    }
                }
            })
        })
        .on(Event::Error, |err, _| {
            Box::pin(async move {
                log::error!("Socket.IO error: {err:?}");
            })
        })
        .connect()
        .await?;

    // Keep the client alive until Ctrl-C
    tokio::signal::ctrl_c().await?;
    client.disconnect().await?;
    Ok(())
}
