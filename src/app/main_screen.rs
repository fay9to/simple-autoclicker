//! Main screen: click interval, click options, repeat, cursor position and the
//! bottom action row.

use std::time::{Duration, Instant};

use eframe::egui::{Context, Pos2, Rect, Ui, Vec2};

use crate::dialogs::Dialog;
use crate::keys::key_name;
use crate::theme::IconKind;
use crate::widgets::{self, BtnStyle};

use super::{App, Menu, Screen};

fn rect(x: f32, y: f32, w: f32, h: f32) -> Rect {
    Rect::from_min_size(Pos2::new(x, y), Vec2::new(w, h))
}

impl App {
    pub(super) fn draw_main(&mut self, ctx: &Context, ui: &mut Ui, enabled: bool) {
        let p = self.palette;

        widgets::card(ui.painter(), Rect::from_min_size(Pos2::new(16.0, 16.0), Vec2::new(428.0, 104.0)), "Click interval", &p);
        widgets::card(ui.painter(), Rect::from_min_size(Pos2::new(16.0, 132.0), Vec2::new(206.0, 150.0)), "Click options", &p);
        widgets::card(ui.painter(), Rect::from_min_size(Pos2::new(238.0, 132.0), Vec2::new(206.0, 150.0)), "Click repeat", &p);
        widgets::card(ui.painter(), Rect::from_min_size(Pos2::new(16.0, 294.0), Vec2::new(428.0, 96.0)), "Cursor position", &p);

        // ---- click interval fields ----
        let fields = [
            ("hours", 32.0, &mut self.hours),
            ("mins", 134.0, &mut self.mins),
            ("secs", 236.0, &mut self.secs),
            ("milliseconds", 338.0, &mut self.millis),
        ];
        let captions = [
            ("hours", 77.0),
            ("mins", 179.0),
            ("secs", 281.0),
            ("milliseconds", 383.0),
        ];
        for (text, center) in captions {
            widgets::caption(ui.painter(), Pos2::new(center, 62.0), text, &p);
        }
        for (key, x, value) in fields {
            widgets::number_field(ui, rect(x, 74.0, 90.0, 34.0), key, value, enabled, &p);
        }

        // ---- theme button on the interval card ----
        let theme_rect = rect(404.0, 24.0, 26.0, 26.0);
        let icon = if self.dark { IconKind::Sun } else { IconKind::Moon };
        let theme_response = widgets::button(
            ui,
            theme_rect,
            "theme",
            "",
            BtnStyle::Ghost,
            Some(icon),
            15.0,
            None,
            enabled,
            &p,
        );
        let theme_clicked = theme_response.clicked();
        let _ = theme_response.on_hover_text("Appearance: system, light or dark");
        if theme_clicked {
            self.toggle_menu(Menu::Theme(theme_rect));
        }

        // ---- click options ----
        widgets::caption_left(ui.painter(), Pos2::new(30.0, 178.0), "Mouse button", &p);
        let button_text = ["Left", "Right", "Middle"][self.button_idx.min(2)];
        if widgets::select_field(
            ui,
            rect(30.0, 188.0, 178.0, 32.0),
            "button",
            button_text,
            matches!(self.menu, Some(Menu::Button(_))),
            enabled,
            &p,
        )
        .clicked()
        {
            self.toggle_menu(Menu::Button(rect(30.0, 188.0, 178.0, 32.0)));
        }

        widgets::caption_left(ui.painter(), Pos2::new(30.0, 236.0), "Click type", &p);
        let type_text = if self.type_idx == 1 { "Double" } else { "Single" };
        if widgets::select_field(
            ui,
            rect(30.0, 246.0, 178.0, 32.0),
            "type",
            type_text,
            matches!(self.menu, Some(Menu::Type(_))),
            enabled,
            &p,
        )
        .clicked()
        {
            self.toggle_menu(Menu::Type(rect(30.0, 246.0, 178.0, 32.0)));
        }

        // ---- click repeat ----
        if widgets::radio(
            ui,
            rect(252.0, 164.0, 120.0, 22.0),
            "repeat",
            "Repeat",
            !self.repeat_until_stopped,
            enabled,
            &p,
        )
        .clicked()
        {
            self.repeat_until_stopped = false;
        }

        let spin_enabled = enabled && !self.repeat_until_stopped;
        let (_, delta) = widgets::spin_field(
            ui,
            rect(252.0, 194.0, 84.0, 34.0),
            "repeat-count",
            &mut self.repeat_text,
            spin_enabled,
            &p,
        );
        if delta != 0 {
            let current = super::read_number(&self.repeat_text).max(1) as i64;
            let next = (current + delta as i64).clamp(1, 1_000_000);
            self.repeat_text = next.to_string();
        }
        widgets::caption_left(ui.painter(), Pos2::new(342.0, 211.0), "times", &p);

        if widgets::radio(
            ui,
            rect(252.0, 240.0, 186.0, 22.0),
            "until-stopped",
            "Repeat until stopped",
            self.repeat_until_stopped,
            enabled,
            &p,
        )
        .clicked()
        {
            self.repeat_until_stopped = true;
        }

        // ---- cursor position ----
        if widgets::radio(
            ui,
            rect(30.0, 326.0, 170.0, 22.0),
            "current",
            "Current location",
            !self.cursor_picked,
            enabled,
            &p,
        )
        .clicked()
        {
            self.cursor_picked = false;
        }

        if widgets::radio(
            ui,
            rect(31.0, 358.0, 20.0, 22.0),
            "picked",
            "",
            self.cursor_picked,
            enabled,
            &p,
        )
        .clicked()
        {
            self.cursor_picked = true;
        }

        let pick_rect = rect(57.0, 352.0, 132.0, 34.0);
        let pick_response = widgets::button(
            ui,
            pick_rect,
            "pick",
            "Pick location",
            BtnStyle::Ghost,
            Some(IconKind::Pick),
            16.0,
            None,
            enabled,
            &p,
        );
        let pick_clicked = pick_response.clicked();
        let _ = pick_response.on_hover_text("Capture the current cursor position");
        if pick_clicked {
            self.open_dialog(
                ctx,
                Dialog::Pick {
                    deadline: Instant::now() + Duration::from_secs(3),
                    result: None,
                },
            );
        }

        widgets::caption_left(ui.painter(), Pos2::new(200.0, 370.0), "X", &p);
        widgets::number_field(
            ui,
            rect(216.0, 354.0, 58.0, 34.0),
            "x",
            &mut self.x,
            enabled && self.cursor_picked,
            &p,
        );
        widgets::caption_left(ui.painter(), Pos2::new(282.0, 370.0), "Y", &p);
        widgets::number_field(
            ui,
            rect(298.0, 354.0, 58.0, 34.0),
            "y",
            &mut self.y,
            enabled && self.cursor_picked,
            &p,
        );

        // ---- bottom buttons ----
        let running = self.engine.is_running();
        let name = key_name(self.settings.hotkey);

        if widgets::button(
            ui,
            rect(16.0, 404.0, 206.0, 44.0),
            "start",
            &format!("Start ({name})"),
            BtnStyle::Primary,
            None,
            0.0,
            None,
            enabled && !running,
            &p,
        )
        .clicked()
        {
            self.start_clicking();
        }

        if widgets::button(
            ui,
            rect(238.0, 404.0, 206.0, 44.0),
            "stop",
            &format!("Stop ({name})"),
            BtnStyle::Danger,
            None,
            0.0,
            None,
            enabled && running,
            &p,
        )
        .clicked()
        {
            self.stop_clicking();
        }

        if widgets::button(
            ui,
            rect(16.0, 458.0, 206.0, 36.0),
            "hotkey",
            "Hotkey setting",
            BtnStyle::Ghost,
            None,
            0.0,
            None,
            enabled,
            &p,
        )
        .clicked()
        {
            let value = self.settings.hotkey;
            self.open_dialog(ctx, Dialog::Hotkey { value });
        }

        if widgets::button(
            ui,
            rect(238.0, 458.0, 206.0, 36.0),
            "record",
            "Record & Playback",
            BtnStyle::Ghost,
            None,
            0.0,
            None,
            enabled,
            &p,
        )
        .clicked()
        {
            self.set_screen(ctx, Screen::Advanced);
        }
    }
}
