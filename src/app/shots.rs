//! Offscreen screenshot mode used to regenerate the images in `docs/`.
//!
//! It is compiled only with the `docs-shots` feature and only activates when
//! the `SAC_SHOT_DIR` environment variable is set. It walks through a fixed
//! list of scenes, waits for the animations to settle, captures the framebuffer
//! and writes one BMP per scene (converted to PNG afterwards).
//!
//!   cargo build --release --features docs-shots
//!   $env:SAC_SHOT_DIR = "docs"; .\target\release\SimpleAutoClicker.exe

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use eframe::egui::{Color32, ColorImage, Context, Pos2, Rect, Vec2, ViewportCommand};

use crate::dialogs::Dialog;
use crate::i18n::Lang;
use crate::theme::{self, Palette};

use super::{App, Menu, Screen};

/// Frames to wait before capturing, so fades and menu animations finish.
const SETTLE_FRAMES: u32 = 45;

#[derive(Clone, Copy)]
enum Scene {
    MainDark,
    MainLight,
    LanguageLight,
    AdvancedDark,
    HotkeyDark,
}

impl Scene {
    fn file(self) -> &'static str {
        match self {
            Scene::MainDark => "main-dark.bmp",
            Scene::MainLight => "main-light.bmp",
            Scene::LanguageLight => "language-menu.bmp",
            Scene::AdvancedDark => "advanced.bmp",
            Scene::HotkeyDark => "hotkey-dialog.bmp",
        }
    }

    /// Solid colour the translucent window is composited over.
    fn backdrop(self) -> Color32 {
        match self {
            Scene::MainLight | Scene::LanguageLight => Color32::from_rgb(240, 243, 250),
            _ => Color32::from_rgb(11, 13, 19),
        }
    }
}

pub struct Shots {
    scenes: Vec<Scene>,
    index: usize,
    frames: u32,
    requested: bool,
    saved: Arc<AtomicUsize>,
}

impl Shots {
    pub fn new() -> Self {
        Self {
            scenes: vec![
                Scene::MainDark,
                Scene::MainLight,
                Scene::LanguageLight,
                Scene::AdvancedDark,
                Scene::HotkeyDark,
            ],
            index: 0,
            frames: 0,
            requested: false,
            saved: Arc::new(AtomicUsize::new(0)),
        }
    }

    fn scene(&self) -> Scene {
        self.scenes[self.index]
    }
}

impl App {
    pub(super) fn run_shots(&mut self, ctx: &Context) {
        let Some(mut shots) = self.shots.take() else {
            return;
        };
        shots.frames += 1;

        if shots.frames == 1 {
            self.apply_scene(ctx, shots.scene());
        }

        if !shots.requested && shots.frames >= SETTLE_FRAMES {
            shots.requested = true;
            let path = shot_dir().join(shots.scene().file());
            let backdrop = shots.scene().backdrop();
            let saved = Arc::clone(&shots.saved);
            ctx.request_screenshot(move |image: Arc<ColorImage>| {
                write_bmp(&path, &image, backdrop);
                saved.fetch_add(1, Ordering::SeqCst);
            });
        }

        if shots.saved.load(Ordering::SeqCst) > shots.index {
            shots.index += 1;
            shots.frames = 0;
            shots.requested = false;
            if shots.index == shots.scenes.len() {
                println!("screenshots written to {}", shot_dir().display());
                ctx.send_viewport_cmd(ViewportCommand::Close);
                return;
            }
        }

        self.shots = Some(shots);
    }

    pub(super) fn shots_active(&self) -> bool {
        self.shots.is_some()
    }

    fn apply_scene(&mut self, ctx: &Context, scene: Scene) {
        let now = Instant::now();
        self.screen = Screen::Main;
        self.screen_since = now - Duration::from_secs(1);
        self.menu = None;
        self.dialog = None;
        self.dialog_since = None;
        self.lang = Lang::Ru;

        let dark = !matches!(scene, Scene::MainLight | Scene::LanguageLight);
        self.dark = dark;
        self.palette = if dark {
            Palette::dark()
        } else {
            Palette::light()
        };
        theme::apply_style(ctx, &self.palette, dark);

        match scene {
            Scene::MainDark | Scene::MainLight => {}
            Scene::LanguageLight => {
                self.menu = Some(Menu::Language(Rect::from_min_size(
                    Pos2::new(366.0, 17.0),
                    Vec2::splat(30.0),
                )));
                self.menu_since = now - Duration::from_secs(1);
            }
            Scene::AdvancedDark => self.screen = Screen::Advanced,
            Scene::HotkeyDark => {
                self.dialog = Some(Dialog::Hotkey {
                    value: self.settings.hotkey,
                });
                self.dialog_since = Some(now - Duration::from_secs(1));
            }
        }
    }
}

fn shot_dir() -> PathBuf {
    std::env::var_os("SAC_SHOT_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("docs"))
}

/// Composites a premultiplied pixel over the solid backdrop.
fn over(px: Color32, backdrop: Color32) -> (u8, u8, u8) {
    let alpha = px.a() as u32;
    if alpha >= 255 {
        return (px.r(), px.g(), px.b());
    }
    let inv = 255 - alpha;
    let blend = |src: u8, dst: u8| (src as u32 + dst as u32 * inv / 255).min(255) as u8;
    (
        blend(px.r(), backdrop.r()),
        blend(px.g(), backdrop.g()),
        blend(px.b(), backdrop.b()),
    )
}

/// Writes a 24-bit BMP; PowerShell converts it to a compressed PNG.
fn write_bmp(path: &Path, image: &ColorImage, backdrop: Color32) {
    let [w, h] = image.size;
    let row_bytes = w * 3;
    let padding = (4 - row_bytes % 4) % 4;
    let stride = row_bytes + padding;
    let data_size = stride * h;
    let file_size = 54 + data_size;

    let mut out = Vec::with_capacity(file_size);
    out.extend_from_slice(b"BM");
    out.extend_from_slice(&(file_size as u32).to_le_bytes());
    out.extend_from_slice(&[0, 0, 0, 0]);
    out.extend_from_slice(&54u32.to_le_bytes());
    out.extend_from_slice(&40u32.to_le_bytes());
    out.extend_from_slice(&(w as i32).to_le_bytes());
    out.extend_from_slice(&(h as i32).to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&24u16.to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes());
    out.extend_from_slice(&(data_size as u32).to_le_bytes());
    out.extend_from_slice(&[0; 16]);

    for y in (0..h).rev() {
        for x in 0..w {
            let (r, g, b) = over(image.pixels[y * w + x], backdrop);
            out.extend_from_slice(&[b, g, r]);
        }
        out.extend_from_slice(&vec![0u8; padding]);
    }

    if let Err(error) = fs::write(path, out) {
        eprintln!("could not write {}: {error}", path.display());
    }
}
