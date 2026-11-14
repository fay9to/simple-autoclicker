//! Main screen: click interval, click options, repeat, cursor position and the
//! bottom action row.

use std::time::{Duration, Instant};

use eframe::egui::{Context, Pos2, Rect, Ui, Vec2};

use crate::dialogs::Dialog;
use crate::i18n::{self, Key};
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

        widgets::card(
            ui.painter(),
            rect(20.0, 78.0, 440.0, 112.0),
            self.tr(Key::ClickInterval),
            &p,
        );
        widgets::card(
            ui.painter(),
            rect(20.0, 202.0, 212.0, 172.0),
            self.tr(Key::ClickOptions),
            &p,
        );
        widgets::card(
            ui.painter(),
            rect(236.0, 202.0, 224.0, 172.0),
            self.tr(Key::ClickRepeat),
            &p,
        );
        widgets::card(
            ui.painter(),
            rect(20.0, 386.0, 440.0, 104.0),
            self.tr(Key::CursorPosition),
            &p,
        );

        // ---- click interval fields ----
        for (key, center) in [
            (Key::Hours, 82.5),
            (Key::Mins, 187.5),
            (Key::Secs, 292.5),
            (Key::Millis, 397.5),
        ] {
            widgets::caption(ui.painter(), Pos2::new(center, 124.0), self.tr(key), &p);
        }
        widgets::number_field(ui, rect(36.0, 136.0, 93.0, 36.0), "hours", &mut self.hours, enabled, &p);
        widgets::number_field(ui, rect(141.0, 136.0, 93.0, 36.0), "mins", &mut self.mins, enabled, &p);
        widgets::number_field(ui, rect(246.0, 136.0, 93.0, 36.0), "secs", &mut self.secs, enabled, &p);
        widgets::number_field(ui, rect(351.0, 136.0, 93.0, 36.0), "millis", &mut self.millis, enabled, &p);

        // ---- click options ----
        widgets::caption_left(
            ui.painter(),
            Pos2::new(36.0, 252.0),
            self.tr(Key::MouseButton),
            &p,
        );
        let button_text = [
            self.tr(Key::Left),
            self.tr(Key::Right),
            self.tr(Key::Middle),
        ][self.button_idx.min(2)];
        if widgets::select_field(
            ui,
            rect(36.0, 262.0, 180.0, 34.0),
            "button",
            button_text,
            matches!(self.menu, Some(Menu::Button(_))),
            enabled,
            &p,
        )
        .clicked()
        {
            self.toggle_menu(Menu::Button(rect(36.0, 262.0, 180.0, 34.0)));
        }

        widgets::caption_left(
            ui.painter(),
            Pos2::new(36.0, 320.0),
            self.tr(Key::ClickType),
            &p,
        );
        let type_text = if self.type_idx == 1 {
            self.tr(Key::Double)
        } else {
            self.tr(Key::Single)
        };
        if widgets::select_field(
            ui,
            rect(36.0, 330.0, 180.0, 34.0),
            "type",
            type_text,
            matches!(self.menu, Some(Menu::Type(_))),
            enabled,
            &p,
        )
        .clicked()
        {
            self.toggle_menu(Menu::Type(rect(36.0, 330.0, 180.0, 34.0)));
        }

        // ---- click repeat ----
        if widgets::radio(
            ui,
            rect(252.0, 248.0, 170.0, 22.0),
            "repeat",
            self.tr(Key::Repeat),
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
            rect(252.0, 280.0, 104.0, 36.0),
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
        widgets::caption_left(ui.painter(), Pos2::new(366.0, 298.0), self.tr(Key::Times), &p);

        if widgets::radio(
            ui,
            rect(252.0, 330.0, 196.0, 22.0),
            "until-stopped",
            self.tr(Key::UntilStopped),
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
            rect(36.0, 430.0, 190.0, 22.0),
            "current",
            self.tr(Key::CurrentLocation),
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
            rect(36.0, 460.0, 20.0, 22.0),
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

        let pick_rect = rect(64.0, 450.0, 142.0, 40.0);
        let pick_response = widgets::button(
            ui,
            pick_rect,
            "pick",
            self.tr(Key::PickLocation),
            BtnStyle::Ghost,
            Some(IconKind::Pick),
            16.0,
            None,
            enabled,
            &p,
        );
        let pick_clicked = pick_response.clicked();
        let _ = pick_response.on_hover_text(self.tr(Key::TipPick));
        if pick_clicked {
            self.open_dialog(
                ctx,
                Dialog::Pick {
                    deadline: Instant::now() + Duration::from_secs(3),
                    result: None,
                },
            );
        }

        widgets::caption_left(ui.painter(), Pos2::new(222.0, 470.0), "X", &p);
        widgets::number_field(
            ui,
            rect(236.0, 452.0, 58.0, 36.0),
            "x",
            &mut self.x,
            enabled && self.cursor_picked,
            &p,
        );
        widgets::caption_left(ui.painter(), Pos2::new(306.0, 470.0), "Y", &p);
        widgets::number_field(
            ui,
            rect(320.0, 452.0, 58.0, 36.0),
            "y",
            &mut self.y,
            enabled && self.cursor_picked,
            &p,
        );

        // ---- bottom buttons ----
        let running = self.engine.is_running();
        let name = key_name(self.settings.hotkey);
        let start_label = i18n::start_button(self.lang, &name);
        let stop_label = i18n::stop_button(self.lang, &name);

        if widgets::button(
            ui,
            rect(20.0, 512.0, 214.0, 54.0),
            "start",
            &start_label,
            BtnStyle::Hero,
            Some(IconKind::Play),
            14.0,
            Some(p.on_accent),
            enabled && !running,
            &p,
        )
        .clicked()
        {
            self.start_clicking();
        }

        if widgets::button(
            ui,
            rect(246.0, 512.0, 214.0, 54.0),
            "stop",
            &stop_label,
            BtnStyle::Danger,
            Some(IconKind::Stop),
            13.0,
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
            rect(20.0, 580.0, 214.0, 40.0),
            "hotkey",
            self.tr(Key::HotkeySetting),
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
            rect(246.0, 580.0, 214.0, 40.0),
            "record",
            self.tr(Key::RecordPlayback),
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
