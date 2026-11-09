//! Application state and the eframe update loop.

mod advanced_screen;
mod main_screen;
mod menus;

use std::time::{Duration, Instant};

use eframe::egui::{self, Context, Rect, Vec2, ViewportCommand};
use windows::Win32::Foundation::HWND;

use crate::dialogs::{Dialog, DialogEvent};
use crate::engine::{ClickConfig, Engine};
use crate::hotkeys::{Hotkeys, ID_PLAYBACK, ID_RECORD, ID_START_STOP};
use crate::keys::key_name;
use crate::native::{self, MouseButtonKind};
use crate::record::Recorder;
use crate::settings::{AppTheme, Settings};
use crate::theme::{self, Palette};

pub const BASE_TITLE: &str = "Simple Auto Clicker 3.1.1";

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Main,
    Advanced,
}

#[derive(Clone, Copy)]
pub enum Menu {
    Theme(Rect),
    Button(Rect),
    Type(Rect),
    Options(Rect),
    Help(Rect),
    More(Rect),
}

pub struct HotkeySync {
    pub registered: bool,
    pub attempts: u32,
    pub next_try: Option<Instant>,
}

pub struct App {
    pub settings: Settings,
    pub dark: bool,
    pub palette: Palette,
    pub screen: Screen,

    pub hwnd: Option<HWND>,
    pub centered: bool,
    pub ever_started: bool,
    pub was_running: bool,
    pub last_title: String,

    // Main screen inputs.
    pub hours: String,
    pub mins: String,
    pub secs: String,
    pub millis: String,
    pub button_idx: usize,
    pub type_idx: usize,
    pub repeat_until_stopped: bool,
    pub repeat_text: String,
    pub cursor_picked: bool,
    pub x: String,
    pub y: String,

    pub engine: Engine,
    pub hotkeys: Hotkeys,
    pub hotkey_sync: HotkeySync,

    pub dialog: Option<Dialog>,
    pub menu: Option<Menu>,
    pub menu_fresh: bool,

    pub rec: Recorder,
    pub adv_keys_registered: bool,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>, settings: Settings) -> Self {
        theme::install_fonts(&cc.egui_ctx);

        let dark = theme::resolve(settings.theme);
        let palette = if dark {
            Palette::dark()
        } else {
            Palette::light()
        };
        theme::apply_style(&cc.egui_ctx, &palette, dark);

        let hotkeys = Hotkeys::new();
        let mut hotkey_sync = HotkeySync {
            registered: false,
            attempts: 0,
            next_try: None,
        };
        if settings.hotkey != 0 && hotkeys.set(ID_START_STOP, settings.hotkey) {
            hotkey_sync.registered = true;
        } else {
            hotkey_sync.next_try = Some(Instant::now() + Duration::from_millis(250));
        }

        Self {
            settings,
            dark,
            palette,
            screen: Screen::Main,
            hwnd: None,
            centered: false,
            ever_started: false,
            was_running: false,
            last_title: BASE_TITLE.to_string(),
            hours: "0".to_string(),
            mins: "0".to_string(),
            secs: "0".to_string(),
            millis: "100".to_string(),
            button_idx: 0,
            type_idx: 0,
            repeat_until_stopped: true,
            repeat_text: "1".to_string(),
            cursor_picked: false,
            x: "0".to_string(),
            y: "0".to_string(),
            engine: Engine::new(),
            hotkeys,
            hotkey_sync,
            dialog: None,
            menu: None,
            menu_fresh: false,
            rec: Recorder::new(),
            adv_keys_registered: false,
        }
    }

    // ------------------------------------------------------------ state polling

    fn poll_state(&mut self, ctx: &Context) {
        let running = self.engine.is_running();
        if self.was_running && !running {
            self.ever_started = true;
        }
        self.was_running = running;
        let _ = self.engine.take_finished();

        if self.dialog.is_none() {
            let title = match self.screen {
                Screen::Main => {
                    if running {
                        format!("Running - {BASE_TITLE}")
                    } else if self.ever_started {
                        format!("Stopped - {BASE_TITLE}")
                    } else {
                        BASE_TITLE.to_string()
                    }
                }
                Screen::Advanced => "Simple Auto Clicker Advanced".to_string(),
            };

            if title != self.last_title {
                ctx.send_viewport_cmd(ViewportCommand::Title(title.clone()));
                self.last_title = title;
            }
        }

        // The previous instance may still hold the key for a moment after a
        // restart, so retry a few times before giving up silently.
        if !self.hotkey_sync.registered && self.settings.hotkey != 0 {
            let due = self
                .hotkey_sync
                .next_try
                .map(|at| Instant::now() >= at)
                .unwrap_or(false);
            if due && self.hotkey_sync.attempts < 12 {
                self.hotkey_sync.attempts += 1;
                if self.hotkeys.set(ID_START_STOP, self.settings.hotkey) {
                    self.hotkey_sync.registered = true;
                    self.hotkey_sync.attempts = 0;
                    self.hotkey_sync.next_try = None;
                } else {
                    self.hotkey_sync.next_try =
                        Some(Instant::now() + Duration::from_millis(250));
                }
            }
        }
    }

    fn handle_hotkeys(&mut self, ctx: &Context) {
        for id in self.hotkeys.poll() {
            match id {
                ID_START_STOP => self.toggle_clicking(),
                ID_PLAYBACK if self.screen == Screen::Advanced => self.toggle_playback(ctx),
                ID_RECORD if self.screen == Screen::Advanced => self.toggle_recording(ctx),
                _ => {}
            }
        }
    }

    // ---------------------------------------------------------------- clicking

    fn build_config(&self) -> ClickConfig {
        let total = read_number(&self.hours) as u64 * 3_600_000
            + read_number(&self.mins) as u64 * 60_000
            + read_number(&self.secs) as u64 * 1_000
            + read_number(&self.millis) as u64;
        let interval_ms = if total == 0 {
            1
        } else {
            total.min(u32::MAX as u64)
        };

        ClickConfig {
            interval_ms,
            button: match self.button_idx {
                1 => MouseButtonKind::Right,
                2 => MouseButtonKind::Middle,
                _ => MouseButtonKind::Left,
            },
            double_click: self.type_idx == 1,
            repeat_forever: self.repeat_until_stopped,
            repeat_count: read_number(&self.repeat_text).max(1),
            fixed_position: self.cursor_picked,
            x: read_number(&self.x) as i32,
            y: read_number(&self.y) as i32,
        }
    }

    fn start_clicking(&mut self) {
        if self.engine.is_running() {
            return;
        }
        self.ever_started = true;
        let config = self.build_config();
        self.engine.start(config);
    }

    fn stop_clicking(&mut self) {
        self.engine.stop();
    }

    fn toggle_clicking(&mut self) {
        if self.engine.is_running() {
            self.stop_clicking();
        } else {
            self.start_clicking();
        }
    }

    // --------------------------------------------------------------- recording

    fn own_window(&self) -> Option<(i32, i32, i32, i32)> {
        let hwnd = self.hwnd?;
        if native::is_minimized(hwnd) {
            return None;
        }
        native::window_rect(hwnd)
    }

    pub fn toggle_recording(&mut self, ctx: &Context) {
        if self.rec.recording {
            self.stop_recording(ctx);
        } else {
            self.start_recording(ctx);
        }
    }

    fn start_recording(&mut self, ctx: &Context) {
        if self.rec.playing {
            return;
        }

        if !self.rec.start_recording() {
            self.open_dialog(
                ctx,
                Dialog::Info {
                    text: "Could not start recording: the system did not allow the mouse hook."
                        .to_string(),
                    warning: true,
                },
            );
            return;
        }

        if self.settings.minimize_when_recording {
            self.rec.restore_after = true;
            ctx.send_viewport_cmd(ViewportCommand::Minimized(true));
        }
    }

    fn stop_recording(&mut self, ctx: &Context) {
        self.rec.stop_recording();
        self.restore_if_needed(ctx);
    }

    pub fn toggle_playback(&mut self, ctx: &Context) {
        if self.rec.playing {
            self.stop_playback(ctx);
        } else {
            self.start_playback(ctx);
        }
    }

    fn start_playback(&mut self, ctx: &Context) {
        if self.rec.recording {
            return;
        }
        if self.rec.events.is_empty() {
            self.open_dialog(
                ctx,
                Dialog::Info {
                    text: "Record something first, using the Record button.".to_string(),
                    warning: false,
                },
            );
            return;
        }

        if !self.rec.start_playback() {
            return;
        }

        if self.settings.minimize_when_playing {
            self.rec.restore_after = true;
            ctx.send_viewport_cmd(ViewportCommand::Minimized(true));
        }
    }

    fn stop_playback(&mut self, ctx: &Context) {
        self.rec.stop_playback();
        self.restore_if_needed(ctx);
    }

    fn restore_if_needed(&mut self, ctx: &Context) {
        if !self.rec.restore_after {
            return;
        }
        self.rec.restore_after = false;
        ctx.send_viewport_cmd(ViewportCommand::Minimized(false));
        ctx.send_viewport_cmd(ViewportCommand::Focus);
    }

    // ------------------------------------------------------------------ screens

    pub fn set_screen(&mut self, ctx: &Context, screen: Screen) {
        if self.screen == screen {
            return;
        }

        match screen {
            Screen::Main => {
                self.stop_recording(ctx);
                self.stop_playback(ctx);
                self.unregister_advanced_hotkeys();
            }
            Screen::Advanced => self.register_advanced_hotkeys(),
        }

        self.screen = screen;
        self.menu = None;

        let (size, title) = match screen {
            Screen::Main => (Vec2::new(460.0, 510.0), BASE_TITLE.to_string()),
            Screen::Advanced => (
                Vec2::new(380.0, 220.0),
                "Simple Auto Clicker Advanced".to_string(),
            ),
        };

        ctx.send_viewport_cmd(ViewportCommand::InnerSize(size));
        ctx.send_viewport_cmd(ViewportCommand::Title(title.clone()));
        self.last_title = title;
        center_window(ctx, size);
    }

    fn register_advanced_hotkeys(&mut self) {
        let play = self.settings.playback_hotkey;
        let record = self.settings.record_hotkey;
        if play != 0 {
            self.hotkeys.set(ID_PLAYBACK, play);
        }
        if record != 0 {
            self.hotkeys.set(ID_RECORD, record);
        }
        self.adv_keys_registered = true;
    }

    fn unregister_advanced_hotkeys(&mut self) {
        if !self.adv_keys_registered {
            return;
        }
        self.hotkeys.set(ID_PLAYBACK, 0);
        self.hotkeys.set(ID_RECORD, 0);
        self.adv_keys_registered = false;
    }

    // ------------------------------------------------------------------- theme

    fn set_theme(&mut self, ctx: &Context, mode: AppTheme) {
        if self.settings.theme == mode && self.dark == theme::resolve(mode) {
            return;
        }

        self.settings.theme = mode;
        self.settings.save();
        self.dark = theme::resolve(mode);
        self.palette = if self.dark {
            Palette::dark()
        } else {
            Palette::light()
        };
        theme::apply_style(ctx, &self.palette, self.dark);

        if let Some(hwnd) = self.hwnd {
            native::set_dark_title_bar(hwnd, self.dark);
        }
    }

    // ----------------------------------------------------------------- dialogs

    fn open_dialog(&mut self, ctx: &Context, dialog: Dialog) {
        self.menu = None;

        // The Advanced screen is only 380x220 points, smaller than the dialogs.
        // Grow the window to the dialog size plus a margin while it is open, so
        // it reads as a modal instead of filling the whole window.
        if self.screen == Screen::Advanced {
            let size = dialog.size(ctx) + Vec2::splat(24.0);
            ctx.send_viewport_cmd(ViewportCommand::InnerSize(size));
            ctx.send_viewport_cmd(ViewportCommand::Title(dialog.title().to_string()));
            self.last_title = dialog.title().to_string();
            center_window(ctx, size);
        }

        self.dialog = Some(dialog);
        // Steals focus from any text field so typing goes to the dialog only.
        ctx.memory_mut(|m| m.request_focus(egui::Id::unique("dialog-input")));
    }

    /// Restores the window of the current screen after a dialog closed.
    fn close_dialog(&mut self, ctx: &Context) {
        if self.screen != Screen::Advanced {
            return;
        }
        let size = Vec2::new(380.0, 220.0);
        ctx.send_viewport_cmd(ViewportCommand::InnerSize(size));
        ctx.send_viewport_cmd(ViewportCommand::Title(
            "Simple Auto Clicker Advanced".to_string(),
        ));
        self.last_title = "Simple Auto Clicker Advanced".to_string();
        center_window(ctx, size);
    }

    fn apply_dialog_event(&mut self, ctx: &Context, event: DialogEvent) {
        let Some(dialog) = self.dialog.take() else {
            return;
        };

        match (dialog, event) {
            (Dialog::Info { .. }, _) => {}
            (Dialog::Hotkey { value }, DialogEvent::Ok) => self.apply_start_hotkey(ctx, value),
            (
                Dialog::AdvSettings {
                    minimize_play,
                    minimize_record,
                },
                DialogEvent::Ok,
            ) => {
                self.settings.minimize_when_playing = minimize_play;
                self.settings.minimize_when_recording = minimize_record;
                self.settings.save();
            }
            (Dialog::AdvHotkeys { play, record, .. }, DialogEvent::Ok) => {
                self.apply_advanced_hotkeys(ctx, play, record);
            }
            (Dialog::Pick { result: Some((x, y)), .. }, DialogEvent::Ok) => {
                self.x = x.to_string();
                self.y = y.to_string();
                self.cursor_picked = true;
            }
            _ => {}
        }

        // Dialog is gone (or replaced by another one): restore the screen size.
        if self.dialog.is_none() {
            self.close_dialog(ctx);
        }
    }

    fn apply_start_hotkey(&mut self, ctx: &Context, value: u32) {
        let previous = self.settings.hotkey;
        if self.hotkeys.set(ID_START_STOP, value) {
            self.settings.hotkey = value;
            self.settings.save();
            self.hotkey_sync.registered = true;
            self.hotkey_sync.attempts = 0;
            self.hotkey_sync.next_try = None;
        } else {
            let _ = self.hotkeys.set(ID_START_STOP, previous);
            self.open_dialog(
                ctx,
                Dialog::Info {
                    text: format!(
                        "Could not assign {} - another program may already use this key.",
                        key_name(value)
                    ),
                    warning: true,
                },
            );
        }
    }

    fn apply_advanced_hotkeys(&mut self, ctx: &Context, play: u32, record: u32) {
        let old_play = self.settings.playback_hotkey;
        let old_record = self.settings.record_hotkey;

        let play_ok = self.hotkeys.set(ID_PLAYBACK, play);
        let record_ok = if play_ok {
            self.hotkeys.set(ID_RECORD, record)
        } else {
            false
        };
        if play_ok && record_ok {
            self.settings.playback_hotkey = play;
            self.settings.record_hotkey = record;
            self.settings.save();
        } else {
            let _ = self.hotkeys.set(ID_PLAYBACK, old_play);
            let _ = self.hotkeys.set(ID_RECORD, old_record);
            self.open_dialog(
                ctx,
                Dialog::Info {
                    text: "Those hotkeys are already used by another program. The previous ones were kept."
                        .to_string(),
                    warning: true,
                },
            );
        }
    }
}

pub fn read_number(text: &str) -> u32 {
    text.trim()
        .parse::<u64>()
        .unwrap_or(0)
        .min(u32::MAX as u64) as u32
}

fn center_window(ctx: &Context, size: Vec2) {
    if let Some(monitor) = ctx.input(|i| i.viewport().monitor_size) {
        if monitor.x > 0.0 && monitor.y > 0.0 {
            ctx.send_viewport_cmd(ViewportCommand::OuterPosition(egui::pos2(
                ((monitor.x - size.x) * 0.5).max(0.0),
                ((monitor.y - size.y) * 0.5).max(0.0),
            )));
        }
    }
}

impl eframe::App for App {
    /// eframe runs no UI pass while the window is minimized and calls this
    /// instead, so all state that must keep working in the background
    /// (hotkeys, recording pump, playback completion) lives here.
    fn logic(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint_after(Duration::from_millis(80));

        self.poll_state(ctx);
        self.handle_hotkeys(ctx);

        let own_window = self.own_window();
        let minimized = self.hwnd.map(native::is_minimized).unwrap_or(false);
        self.rec.pump(own_window, minimized);

        let was_playing = self.rec.playing;
        self.rec.poll_playback();
        if was_playing && !self.rec.playing {
            self.restore_if_needed(ctx);
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();

        // Closing the Advanced screen returns to the main one, just like
        // closing the separate window did in the previous build.
        if self.screen == Screen::Advanced && ctx.input(|i| i.viewport().close_requested()) {
            ctx.send_viewport_cmd(ViewportCommand::CancelClose);
            self.set_screen(&ctx, Screen::Main);
        }

        if self.hwnd.is_none() {
            self.hwnd = native::find_app_window();
            if let Some(hwnd) = self.hwnd {
                native::set_dark_title_bar(hwnd, self.dark);
            }
        }

        if !self.centered {
            center_window(&ctx, Vec2::new(460.0, 510.0));
            self.centered = true;
        }

        ui.spacing_mut().item_spacing = Vec2::ZERO;
        let enabled = self.dialog.is_none();

        match self.screen {
            Screen::Main => self.draw_main(&ctx, ui, enabled),
            Screen::Advanced => {
                if self.dialog.is_none() {
                    self.draw_advanced(&ctx, ui, enabled);
                }
            }
        }

        self.draw_menu(&ctx);

        let mut dialog_event = None;
        if let Some(dialog) = self.dialog.as_mut() {
            dialog_event = dialog.show(&ctx, &self.palette);
        }
        if let Some(event) = dialog_event {
            self.apply_dialog_event(&ctx, event);
        }
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        let c = self.palette.bg;
        [
            c.r() as f32 / 255.0,
            c.g() as f32 / 255.0,
            c.b() as f32 / 255.0,
            1.0,
        ]
    }
}
