use std::sync::{Arc, OnceLock};

use egui::{Align2, Color32, ColorImage, FontId, Pos2, Rect, Vec2};

use crate::app::App;

// ─── egui wakeup helper ────────────────────────────────────────────────────
/// A cheaply-cloneable handle that lets background threads request an egui repaint.
/// The inner OnceLock is set once, in App::new, after the egui context is ready.
pub type CtxWaker = Arc<OnceLock<egui::Context>>;

pub fn new_waker() -> CtxWaker {
    Arc::new(OnceLock::new())
}

pub fn wake(w: &CtxWaker) {
    if let Some(ctx) = w.get() {
        ctx.request_repaint();
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  eframe::App — render loop
// ─────────────────────────────────────────────────────────────────────────────
impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // One-time Win32 overlay setup: color-key transparency + remove decorations
        if !self.win32_initialized {
            self.win32_initialized = true;
            #[cfg(windows)]
            unsafe {
                crate::windows::win32_setup_overlay();
            }
        }

        // Color key: DWM makes RGB(1,0,1) transparent at compositor level.
        // with_transparent(true) is NOT used — on NVIDIA the glow renderer outputs
        // alpha=0 for all pixels, making per-pixel alpha compositing invisible.
        // Disable AA feathering: blended edge pixels don't match the color key
        // and show up as visible artifacts against the transparent background.
        ctx.tessellation_options_mut(|o| o.feathering = false);

        let key = Color32::BLACK;
        ctx.set_visuals(egui::Visuals {
            window_fill: key,
            panel_fill: key,
            window_shadow: egui::Shadow::NONE,
            popup_shadow: egui::Shadow::NONE,
            ..egui::Visuals::dark()
        });

        self.process_events(ctx);
        self.update_video_frame(ctx);

        if self
            .current
            .as_ref()
            .map(|a| a.should_advance())
            .unwrap_or(false)
        {
            self.advance();
        }

        // Repaint scheduling: event-driven when idle, frame-rate-locked when active.
        // Background threads call ctx.request_repaint() via the CtxWaker when new
        // events arrive, so we don't need to poll when there is nothing to show.
        match &self.current {
            Some(a) if a.is_video() => {
                ctx.request_repaint_after(std::time::Duration::from_millis(16));
            }
            Some(_) => {
                // Image / sound / text: repaint at 30 fps for the bob animation.
                ctx.request_repaint_after(std::time::Duration::from_millis(33));
            }
            None => {
                // Idle: sleep until a background thread wakes us via ctx.request_repaint().
            }
        }

        let Some(active) = &self.current else {
            // DEBUG: draw nothing at all — only the GL clear (key color) is active.
            // If dots still appear, they come from Win32/DWM, not egui drawing.
            return;
        };

        let chat = active.chat.clone();
        let avatar_tex = active.avatar_tex.clone();
        let media_tex = active
            .frame_tex
            .as_ref()
            .or(active.media_tex.as_ref())
            .cloned();
        let time = ctx.input(|i| i.time);

        let screen = ctx.screen_rect();
        let w = screen.width();
        let h = screen.height();

        let row_top = h / 6.0;
        let row_mid = h * 4.0 / 6.0;
        let row_bot = h / 6.0;

        let hide_author = chat
            .options
            .as_ref()
            .and_then(|o| o.hide_author)
            .unwrap_or(false);

        let text_opts = chat.options.as_ref().and_then(|o| o.text.as_ref());
        let text_color = text_opts
            .and_then(|t| parse_color(t.color.as_deref()))
            .unwrap_or(Color32::WHITE);
        let text_size = text_opts.and_then(|t| t.font_size).unwrap_or(36.0);

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(key))
            .show(ctx, |ui| {
                let p = ui.painter();

                if !hide_author {
                    let float_y = screen.top()
                        + row_top / 2.0
                        + (time * std::f64::consts::TAU / 4.0).sin() as f32 * 8.0;
                    let cx = w / 2.0;
                    if let Some(ref tex) = avatar_tex {
                        let sz = 72.0_f32;
                        let rect = Rect::from_center_size(Pos2::new(cx, float_y), Vec2::splat(sz));
                        p.image(
                            tex.id(),
                            rect,
                            egui::Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                            Color32::WHITE,
                        );
                        outlined_text(
                            p,
                            &chat.author.name.to_uppercase(),
                            Pos2::new(cx, float_y + sz / 2.0 + 10.0),
                            FontId::proportional(18.0),
                            Color32::WHITE,
                            Color32::BLACK,
                        );
                    }
                }

                let mid = Rect::from_min_size(
                    Pos2::new(screen.left(), screen.top() + row_top),
                    Vec2::new(w, row_mid),
                );
                if let Some(ref tex) = media_tex {
                    let ts = tex.size_vec2();
                    let scale = (mid.width() / ts.x).min(mid.height() / ts.y);
                    let disp = Rect::from_center_size(mid.center(), ts * scale);
                    p.image(
                        tex.id(),
                        disp,
                        egui::Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                        Color32::WHITE,
                    );
                }

                if let Some(ref msg) = chat.message {
                    if !msg.is_empty() {
                        let msg_y = screen.top() + row_top + row_mid + row_bot / 2.0;
                        outlined_text(
                            p,
                            &msg.to_uppercase(),
                            Pos2::new(w / 2.0, msg_y),
                            FontId::proportional(text_size),
                            text_color,
                            Color32::BLACK,
                        );
                    }
                }
            });
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.0, 0.0, 0.0, 1.0] // black = color key → transparent
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  Helpers
// ─────────────────────────────────────────────────────────────────────────────

pub fn decode_image(data: &[u8]) -> Option<ColorImage> {
    let img = image::load_from_memory(data).ok()?.to_rgba8();
    let (w, h) = (img.width() as usize, img.height() as usize);
    Some(ColorImage::from_rgba_unmultiplied([w, h], &img))
}

/// Decode and bake a circular alpha mask (for avatars).
pub fn decode_circular(data: &[u8]) -> Option<ColorImage> {
    let img = image::load_from_memory(data).ok()?;
    let size = img.width().min(img.height());
    let img = img.resize_to_fill(size, size, image::imageops::FilterType::Lanczos3);
    let mut rgba = img.to_rgba8();
    let c = size as f32 / 2.0;
    let r2 = c * c;
    for y in 0..size {
        for x in 0..size {
            let dx = x as f32 - c;
            let dy = y as f32 - c;
            if dx * dx + dy * dy > r2 {
                // Fill with opaque black (the color key) so DWM keys it out cleanly.
                // alpha=0 would cause blending artifacts at the circle edges.
                let p = rgba.get_pixel_mut(x, y);
                p[0] = 0;
                p[1] = 0;
                p[2] = 0;
                p[3] = 255;
            }
        }
    }
    Some(ColorImage::from_rgba_unmultiplied(
        [size as usize, size as usize],
        &rgba,
    ))
}

/// Draw text with a 1 px black outline on all four diagonal corners.
pub fn outlined_text(
    p: &egui::Painter,
    text: &str,
    center: Pos2,
    font: FontId,
    fill: Color32,
    outline: Color32,
) {
    for (dx, dy) in [(-1.0_f32, -1.0_f32), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)] {
        p.text(
            Pos2::new(center.x + dx, center.y + dy),
            Align2::CENTER_CENTER,
            text,
            font.clone(),
            outline,
        );
    }
    p.text(center, Align2::CENTER_CENTER, text, font, fill);
}

/// Parse a CSS hex colour string (#rrggbb or #rgb) → egui Color32.
pub fn parse_color(s: Option<&str>) -> Option<Color32> {
    let s = s?.trim_start_matches('#');
    match s.len() {
        6 => {
            let r = u8::from_str_radix(&s[0..2], 16).ok()?;
            let g = u8::from_str_radix(&s[2..4], 16).ok()?;
            let b = u8::from_str_radix(&s[4..6], 16).ok()?;
            Some(Color32::from_rgb(r, g, b))
        }
        3 => {
            let r = u8::from_str_radix(&s[0..1], 16).ok()? * 17;
            let g = u8::from_str_radix(&s[1..2], 16).ok()? * 17;
            let b = u8::from_str_radix(&s[2..3], 16).ok()? * 17;
            Some(Color32::from_rgb(r, g, b))
        }
        _ => None,
    }
}
