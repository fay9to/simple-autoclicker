//! Record & playback screen.

use eframe::egui::{Align2, Context, Pos2, Rect, Ui, Vec2};

use crate::i18n::Key;
use crate::theme::{font_semibold, font_small, IconKind};
use crate::widgets::{self, BtnStyle};

use super::{App, Menu};

fn rect(x: f32, y: f32, w: f32, h: f32) -> Rect {
    Rect::from_min_size(Pos2::new(x, y), Vec2::new(w, h))
}

impl App {
    pub(super) fn draw_advanced(&mut self, ctx: &Context, ui: &mut Ui, enabled: bool) {
        let p = self.palette;

        let hint = self.tr(Key::RecordHint);
        let galley = ui
            .painter()
            .layout(hint.to_string(), font_small(), p.muted, 440.0);
        ui.painter()
            .galley(Pos2::new(20.0, 78.0), galley, p.muted);

        let playing = self.rec.playing;
        let recording = self.rec.recording;

        let play_rect = rect(20.0, 116.0, 214.0, 180.0);
        let play = widgets::tile(
            ui,
            play_rect,
            "play-tile",
            if playing { IconKind::Stop } else { IconKind::Play },
            if playing {
                self.tr(Key::StopPlayback)
            } else {
                self.tr(Key::Playback)
            },
            self.tr(Key::PlaybackHint),
            if playing { p.danger } else { p.green },
            playing,
            enabled && !recording,
            &p,
        );
        if play.clicked() {
            self.toggle_playback(ctx);
        }

        let record_rect = rect(246.0, 116.0, 214.0, 180.0);
        let record = widgets::tile(
            ui,
            record_rect,
            "record-tile",
            if recording {
                IconKind::Stop
            } else {
                IconKind::Record
            },
            if recording {
                self.tr(Key::StopRecording)
            } else {
                self.tr(Key::Record)
            },
            self.tr(Key::RecordHintTile),
            p.danger,
            recording,
            enabled && !playing,
            &p,
        );
        if record.clicked() {
            self.toggle_recording(ctx);
        }

        // ---- recording details ----
        widgets::card(
            ui.painter(),
            rect(20.0, 312.0, 440.0, 128.0),
            self.tr(Key::Details),
            &p,
        );

        let stats = [
            (self.rec.click_count(), Key::Clicks),
            (self.rec.move_count(), Key::Moves),
            (self.rec.events.len(), Key::Events),
        ];
        for (index, (value, label)) in stats.iter().enumerate() {
            let center = 105.0 + index as f32 * 135.0;
            ui.painter().text(
                Pos2::new(center, 372.0),
                Align2::CENTER_CENTER,
                value.to_string(),
                font_semibold(25.0),
                if *value > 0 { p.ink } else { p.muted },
            );
            ui.painter().text(
                Pos2::new(center, 400.0),
                Align2::CENTER_CENTER,
                self.tr(*label),
                font_small(),
                p.muted,
            );
        }
        if self.rec.events.is_empty() {
            ui.painter().text(
                Pos2::new(240.0, 424.0),
                Align2::CENTER_CENTER,
                self.tr(Key::NothingRecorded),
                font_small(),
                p.disabled_ink,
            );
        }

        // ---- bottom buttons ----
        let more_rect = rect(20.0, 456.0, 214.0, 44.0);
        if widgets::button(
            ui,
            more_rect,
            "more",
            self.tr(Key::More),
            BtnStyle::Ghost,
            Some(IconKind::More),
            14.0,
            None,
            enabled,
            &p,
        )
        .clicked()
        {
            self.toggle_menu(Menu::More(more_rect));
        }

        let can_clear = enabled && !recording && !playing && !self.rec.events.is_empty();
        if widgets::button(
            ui,
            rect(246.0, 456.0, 214.0, 44.0),
            "clear",
            self.tr(Key::ClearRecording),
            BtnStyle::Ghost,
            None,
            0.0,
            None,
            can_clear,
            &p,
        )
        .clicked()
        {
            self.rec.events.clear();
        }

        if widgets::button(
            ui,
            rect(20.0, 516.0, 440.0, 44.0),
            "back",
            self.tr(Key::Back),
            BtnStyle::Secondary,
            Some(IconKind::ChevronLeft),
            13.0,
            None,
            enabled,
            &p,
        )
        .clicked()
        {
            self.set_screen(ctx, super::Screen::Main);
        }

        let tip = self.tr(Key::RecordingDetailsHint);
        let galley = ui
            .painter()
            .layout(tip.to_string(), font_small(), p.muted, 420.0);
        ui.painter().galley(
            Pos2::new(
                240.0 - galley.size().x * 0.5,
                578.0 + (galley.size().y - 16.0) * 0.0,
            ),
            galley,
            p.muted,
        );
    }
}
