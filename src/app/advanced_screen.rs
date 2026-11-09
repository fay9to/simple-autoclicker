//! Advanced screen: record & playback of clicks and mouse movement.

use eframe::egui::{Align2, Context, Pos2, Rect, Ui, Vec2};

use crate::theme::{font_small, IconKind};
use crate::widgets::{self, BtnStyle};

use super::{App, Menu};

fn rect(x: f32, y: f32, w: f32, h: f32) -> Rect {
    Rect::from_min_size(Pos2::new(x, y), Vec2::new(w, h))
}

impl App {
    pub(super) fn draw_advanced(&mut self, ctx: &Context, ui: &mut Ui, enabled: bool) {
        let p = self.palette;

        let options_rect = rect(14.0, 12.0, 74.0, 26.0);
        if widgets::text_button(ui, options_rect, "options", "Options", enabled, &p).clicked() {
            self.toggle_menu(Menu::Options(options_rect));
        }

        let help_rect = rect(94.0, 12.0, 50.0, 26.0);
        if widgets::text_button(ui, help_rect, "help", "Help", enabled, &p).clicked() {
            self.toggle_menu(Menu::Help(help_rect));
        }

        let hint = "Record clicks and mouse movement, then play them back to repeat the same sequence.";
        let galley = ui.painter().layout(hint.to_string(), font_small(), p.muted, 344.0);
        ui.painter().galley(
            Pos2::new(18.0, 54.0 + (34.0 - galley.size().y).max(0.0) * 0.5),
            galley,
            p.muted,
        );

        let playing = self.rec.playing;
        let recording = self.rec.recording;

        let play_rect = rect(24.0, 100.0, 104.0, 76.0);
        let play_icon = if playing { IconKind::Stop } else { IconKind::Play };
        let play_color = if playing { p.ink } else { p.green };
        if widgets::button(
            ui,
            play_rect,
            "play",
            "",
            BtnStyle::Secondary,
            Some(play_icon),
            26.0,
            Some(play_color),
            enabled && !recording,
            &p,
        )
        .clicked()
        {
            self.toggle_playback(ctx);
        }

        let record_rect = rect(138.0, 100.0, 104.0, 76.0);
        let record_icon = if recording { IconKind::Stop } else { IconKind::Record };
        let record_color = if recording { p.ink } else { p.danger };
        if widgets::button(
            ui,
            record_rect,
            "record-adv",
            "",
            BtnStyle::Secondary,
            Some(record_icon),
            24.0,
            Some(record_color),
            enabled && !playing,
            &p,
        )
        .clicked()
        {
            self.toggle_recording(ctx);
        }

        let more_rect = rect(252.0, 100.0, 104.0, 76.0);
        let more_response = widgets::button(
            ui,
            more_rect,
            "more",
            "More",
            BtnStyle::Secondary,
            Some(IconKind::More),
            14.0,
            None,
            enabled && !playing && !recording,
            &p,
        );
        if more_response.clicked() {
            self.toggle_menu(Menu::More(more_rect));
        }

        let status = if recording {
            format!("Recording… events: {}", self.rec.events.len())
        } else if playing {
            "Playing back…".to_string()
        } else {
            format!(
                "Recorded: {} clicks, {} move points",
                self.rec.click_count(),
                self.rec.move_count()
            )
        };

        ui.painter().text(
            Pos2::new(24.0, 196.0),
            Align2::LEFT_CENTER,
            status,
            font_small(),
            p.muted,
        );
    }
}
