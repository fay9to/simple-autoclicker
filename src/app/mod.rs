//! Application state, the title bar and the eframe update loop.

mod advanced_screen;
mod main_screen;
mod menus;
#[cfg(feature = "docs-shots")]
mod shots;

use std::time::{Duration, Instant};

use eframe::egui::{
    self, Align2, Color32, Context, CornerRadius, Id, LayerId, Pos2, Rect, Sense, Stroke,
    StrokeKind, Ui, Vec2, ViewportCommand,
};
use windows::Win32::Foundation::HWND;

use crate::dialogs::{Dialog, DialogEvent};
use crate::engine::{ClickConfig, Engine};
use crate::hotkeys::{Hotkeys, ID_PLAYBACK, ID_RECORD, ID_START_STOP};
use crate::i18n::{self, Key, Lang};
use crate::keys::key_name;
use crate::native::{self, MouseButtonKind};
use crate::record::Recorder;
use crate::settings::{AppTheme, Settings};
use crate::theme::{
    self, ease_out, fade, font_caption, font_semibold, soft_circle, IconKind, Palette,
};
use crate::widgets;

pub const BASE_TITLE: &str = "Simple Auto Clicker";
pub const WINDOW_WIDTH: f32 = 480.0;
pub const WINDOW_HEIGHT: f32 = 640.0;
/// Height of the custom title bar.
pub const HEADER_HEIGHT: f32 = 64.0;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Main,
    Advanced,
}

#[derive(Clone, Copy)]
pub enum Menu {
    Theme(Rect),
    Language(Rect),
    Button(Rect),
    Type(Rect),
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
    pub lang: Lang,
    pub screen: Screen,

    pub hwnd: Option<HWND>,
    pub started: bool,
    pub ever_started: bool,
    pub was_running: bool,

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
    pub dialog_since: Option<Instant>,
    pub menu: Option<Menu>,
    pub menu_since: Instant,
    pub screen_since: Instant,

    pub rec: Recorder,
    pub adv_keys_registered: bool,

    #[cfg(feature = "docs-shots")]
    pub shots: Option<shots::Shots>,
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

        let lang = settings.language.unwrap_or_else(Lang::detect);

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

        let now = Instant::now();
        Self {
            settings,
            dark,
            palette,
            lang,
            screen: Screen::Main,
            hwnd: None,
            started: false,
            ever_started: false,
            was_running: false,
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
            dialog_since: None,
            menu: None,
            menu_since: now,
            screen_since: now,
            rec: Recorder::new(),
            adv_keys_registered: false,
            #[cfg(feature = "docs-shots")]
            shots: if std::env::var_os("SAC_SHOT_DIR").is_some() {
                Some(shots::Shots::new())
            } else {
                None
            },
        }
    }

    pub fn tr(&self, key: Key) -> &'static str {
        i18n::t(self.lang, key)
    }

    // ------------------------------------------------------------ state polling

    fn poll_state(&mut self) {
        let running = self.engine.is_running();
        if self.was_running && !running {
            self.ever_started = true;
        }
        self.was_running = running;
        let _ = self.engine.take_finished();

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
                    self.hotkey_sync.next_try = Some(Instant::now() + Duration::from_millis(250));
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
        let interval_ms = if total == 0 { 1 } else { total.min(u32::MAX as u64) };

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
                    text: self.tr(Key::HookFailed).to_string(),
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
                    text: self.tr(Key::NothingRecordedMessage).to_string(),
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
        self.screen_since = Instant::now();
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

    fn set_language(&mut self, lang: Lang) {
        if self.lang == lang {
            return;
        }
        self.lang = lang;
        self.settings.language = Some(lang);
        self.settings.save();
    }

    // ----------------------------------------------------------------- dialogs

    fn open_dialog(&mut self, ctx: &Context, dialog: Dialog) {
        self.menu = None;
        self.dialog_since = Some(Instant::now());
        self.dialog = Some(dialog);
        // Steals focus from any text field so typing goes to the dialog only.
        ctx.memory_mut(|m| m.request_focus(egui::Id::unique("dialog-input")));
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

        self.dialog_since = None;
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
                    text: i18n::hotkey_busy(self.lang, &key_name(value)),
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
                    text: self.tr(Key::HotkeysBusy).to_string(),
                    warning: true,
                },
            );
        }
    }

    // ------------------------------------------------------------------ chrome

    fn startup(&mut self, ctx: &Context) {
        if let Some(hwnd) = native::find_app_window() {
            self.hwnd = Some(hwnd);
            native::lock_size(hwnd);
            native::round_corners(hwnd);
            native::set_dark_title_bar(hwnd, self.dark);
        } else {
            self.hwnd = native::find_app_window();
        }
        center_window(ctx, Vec2::new(WINDOW_WIDTH, WINDOW_HEIGHT));
        self.started = true;
    }

    /// Keeps the window exactly at its design size, even if something asks the
    /// system to maximize it (the window has no maximize box to begin with).
    fn enforce_size(&self, ctx: &Context) {
        if self.hwnd.map(native::is_minimized).unwrap_or(false) {
            return;
        }
        let size = ctx.input(|i| i.viewport().inner_rect.map(|r| r.size()));
        if let Some(size) = size {
            if (size.x - WINDOW_WIDTH).abs() > 1.0 || (size.y - WINDOW_HEIGHT).abs() > 1.0 {
                ctx.send_viewport_cmd(ViewportCommand::InnerSize(Vec2::new(
                    WINDOW_WIDTH,
                    WINDOW_HEIGHT,
                )));
            }
        }
    }

    fn paint_background(&self, ctx: &Context) {
        let rect = ctx.viewport_rect();
        let p = self.palette;
        let painter = ctx.layer_painter(LayerId::background());

        theme::vertical_gradient(&painter, rect, 18, p.bg_top, p.bg_bottom);

        let time = ctx.input(|i| i.time) as f32;
        let drift = Vec2::new((time * 0.22).sin() * 46.0, (time * 0.16).cos() * 30.0);
        soft_circle(
            &painter,
            Pos2::new(110.0, 80.0) + drift,
            280.0,
            fade(p.accent, 0.20),
        );
        soft_circle(
            &painter,
            Pos2::new(430.0, 590.0) - drift,
            300.0,
            fade(p.accent2, 0.16),
        );
        soft_circle(
            &painter,
            Pos2::new(400.0, -60.0),
            220.0,
            fade(Color32::WHITE, 0.06),
        );

        painter.rect_stroke(
            rect.shrink(0.5),
            CornerRadius::same(18),
            Stroke::new(1.0, fade(p.card_border, 0.9)),
            StrokeKind::Inside,
        );
    }

    fn status(&self) -> (&'static str, Color32) {
        let p = self.palette;
        if self.screen == Screen::Advanced {
            if self.rec.recording {
                (self.tr(Key::Recording), p.danger)
            } else if self.rec.playing {
                (self.tr(Key::Playing), p.green)
            } else {
                (self.tr(Key::Ready), p.muted)
            }
        } else if self.engine.is_running() {
            (self.tr(Key::Running), p.green)
        } else if self.ever_started {
            (self.tr(Key::Stopped), p.muted)
        } else {
            (self.tr(Key::Ready), p.muted)
        }
    }

    fn draw_header(&mut self, ctx: &Context, ui: &mut Ui) {
        let p = self.palette;

        // Drag region: buttons drawn afterwards sit on top of it.
        let bar = Rect::from_min_size(Pos2::ZERO, Vec2::new(WINDOW_WIDTH, HEADER_HEIGHT));
        let drag = ui.interact(bar, Id::unique("title-drag"), Sense::click_and_drag());
        if drag.dragged() {
            ctx.send_viewport_cmd(ViewportCommand::StartDrag);
        }

        let advanced = self.screen == Screen::Advanced;
        if advanced {
            let back_rect = Rect::from_min_size(Pos2::new(18.0, 16.0), Vec2::splat(32.0));
            if widgets::icon_button(ui, back_rect, "nav-back", IconKind::ChevronLeft, 17.0, true, &p)
                .on_hover_text(self.tr(Key::Back))
                .clicked()
            {
                self.set_screen(ctx, Screen::Main);
            }
        } else {
            let logo = Rect::from_min_size(Pos2::new(20.0, 16.0), Vec2::splat(32.0));
            theme::vertical_gradient(ui.painter(), logo, 10, p.accent, p.accent2);
            theme::draw_icon(
                ui.painter(),
                Rect::from_center_size(logo.center(), Vec2::splat(18.0)),
                IconKind::Cursor,
                p.on_accent,
                p.accent,
            );
        }

        let title = if advanced { self.tr(Key::Details) } else { BASE_TITLE };
        let painter = ui.painter();
        painter.text(
            Pos2::new(64.0, 15.0),
            Align2::LEFT_TOP,
            title,
            font_semibold(15.5),
            p.ink,
        );

        // Status pill with a pulsing dot while the app is busy.
        let (label, color) = self.status();
        let galley = painter.layout_no_wrap(label.to_owned(), font_caption(), color);
        let pill = Rect::from_min_size(
            Pos2::new(64.0, 36.0),
            Vec2::new(galley.size().x + 28.0, 20.0),
        );
        painter.rect_filled(pill, CornerRadius::same(10), fade(color, 0.11));
        let busy = self.engine.is_running() || self.rec.recording || self.rec.playing;
        let time = ctx.input(|i| i.time) as f32;
        let pulse = if busy { 0.75 + 0.25 * (time * 3.4).sin() } else { 1.0 };
        painter.circle_filled(
            Pos2::new(pill.left() + 10.0, pill.center().y),
            3.2 * pulse,
            fade(color, pulse),
        );
        painter.galley(
            Pos2::new(pill.left() + 17.0, pill.center().y - galley.size().y * 0.5),
            galley,
            color,
        );

        // Window buttons.
        let size = 30.0;
        let y = 17.0;
        let close = Rect::from_min_size(Pos2::new(434.0, y), Vec2::splat(size));
        let minimize = close.translate(Vec2::new(-34.0, 0.0));
        let language = minimize.translate(Vec2::new(-34.0, 0.0));
        let appearance = language.translate(Vec2::new(-34.0, 0.0));

        let theme_icon = if self.dark { IconKind::Sun } else { IconKind::Moon };
        if widgets::icon_button(ui, appearance, "win-theme", theme_icon, 15.0, true, &p)
            .on_hover_text(self.tr(Key::TipTheme))
            .clicked()
        {
            self.toggle_menu(Menu::Theme(appearance));
        }

        if widgets::icon_button(ui, language, "win-language", IconKind::Globe, 15.0, true, &p)
            .on_hover_text(self.tr(Key::TipLanguage))
            .clicked()
        {
            self.toggle_menu(Menu::Language(language));
        }

        if widgets::icon_button(ui, minimize, "win-min", IconKind::Minus, 15.0, true, &p)
            .on_hover_text(self.tr(Key::TipMinimize))
            .clicked()
        {
            ctx.send_viewport_cmd(ViewportCommand::Minimized(true));
        }

        if widgets::icon_button(ui, close, "win-close", IconKind::Close, 14.0, true, &p)
            .on_hover_text(self.tr(Key::TipClose))
            .clicked()
        {
            ctx.send_viewport_cmd(ViewportCommand::Close);
        }

        ui.painter().line_segment(
            [
                Pos2::new(20.0, HEADER_HEIGHT + 0.5),
                Pos2::new(WINDOW_WIDTH - 20.0, HEADER_HEIGHT + 0.5),
            ],
            Stroke::new(1.0, fade(p.card_border, 0.9)),
        );
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

        // A hidden (e.g. occluded) window is normally not painted; the
        // screenshot mode needs every frame to run.
        #[cfg(feature = "docs-shots")]
        if self.shots_active() {
            ctx.send_viewport_cmd(ViewportCommand::RequestPaintWhileHidden);
        }

        self.poll_state();
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

    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();

        if !self.started {
            self.startup(&ctx);
        }
        self.enforce_size(&ctx);

        self.paint_background(&ctx);

        let fade_in = ease_out(self.screen_since.elapsed().as_secs_f32() / 0.22);
        let enabled = self.dialog.is_none();

        ui.spacing_mut().item_spacing = Vec2::ZERO;
        ui.scope(|ui| {
            ui.set_opacity(fade_in);
            self.draw_header(&ctx, ui);
            match self.screen {
                Screen::Main => self.draw_main(&ctx, ui, enabled),
                Screen::Advanced => self.draw_advanced(&ctx, ui, enabled),
            }
        });

        self.draw_menu(&ctx);

        let mut dialog_event = None;
        if let Some(dialog) = self.dialog.as_mut() {
            let appear = self
                .dialog_since
                .map(|since| ease_out(since.elapsed().as_secs_f32() / 0.20))
                .unwrap_or(1.0);
            dialog_event = dialog.show(&ctx, &self.palette, self.lang, appear);
        }
        if let Some(event) = dialog_event {
            self.apply_dialog_event(&ctx, event);
        }

        #[cfg(feature = "docs-shots")]
        self.run_shots(&ctx);

        ctx.request_repaint_after(Duration::from_millis(16));
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.0, 0.0, 0.0, 0.0]
    }
}
