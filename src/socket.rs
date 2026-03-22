use anyhow::Result;
use rust_socketio::{
    asynchronous::{Client, ClientBuilder},
    Payload,
};
use serde_json::json;
use std::sync::mpsc::Sender;

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
        .on("connect", move |_, socket: Client| {
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
        .on("flush", move |_, _| {
            let tx = tx_flush.clone();
            let waker = waker_flush.clone();
            Box::pin(async move {
                let _ = tx.send(AppEvent::Flush);
                wake(&waker);
            })
        })
        .on("skip", move |_, _| {
            let tx = tx_skip.clone();
            let waker = waker_skip.clone();
            Box::pin(async move {
                let _ = tx.send(AppEvent::Skip);
                wake(&waker);
            })
        })
        .on("error", |err, _| {
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
