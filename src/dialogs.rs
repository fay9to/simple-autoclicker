//! Themed modal dialogs drawn inside the main viewport (no extra windows):
//! messages, hotkey capture, view settings and the pick-location countdown.

use std::time::Instant;

use eframe::egui::{
    Align2, Area, Color32, Context, CornerRadius, Event, Id, Key, LayerId, Order, Pos2, Rect,
    Stroke, StrokeKind, Ui, Vec2,
};

use crate::i18n::{self, Key as Text, Lang};
use crate::keys::{key_name, key_to_vk};
use crate::native;
use crate::theme::{
    fade, font_body, font_caption, font_semibold, glow, soft_shadow, Palette, RADIUS_CARD,
};
use crate::widgets::{self, BtnStyle};

pub enum Dialog {
    Info {
        text: String,
        warning: bool,
    },
    Hotkey {
        value: u32,
    },
    AdvSettings {
        minimize_play: bool,
        minimize_record: bool,
    },
    AdvHotkeys {
        play: u32,
        record: u32,
        active: usize,
    },
    Pick {
        deadline: Instant,
        result: Option<(i32, i32)>,
    },
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DialogEvent {
    Ok,
    Cancel,
}

fn panel(painter: &eframe::egui::Painter, rect: Rect, p: &Palette, accent: Color32) {
    soft_shadow(painter, rect, RADIUS_CARD, p.shadow, 22.0, 8.0);
    glow(painter, rect, RADIUS_CARD, accent, 0.12);
    painter.rect_filled(rect, CornerRadius::same(RADIUS_CARD), p.popup);
    painter.rect_stroke(
        rect,
        CornerRadius::same(RADIUS_CARD),
        Stroke::new(1.0, p.card_border),
        StrokeKind::Inside,
    );

    // Thin accent line under the top edge, kept clear of the rounded corners.
    let strip = Rect::from_min_max(
        Pos2::new(rect.left() + 24.0, rect.top() + 0.5),
        Pos2::new(rect.right() - 24.0, rect.top() + 3.5),
    );
    painter.rect_filled(strip, CornerRadius::same(2), fade(accent, 0.9));
}

fn title(painter: &eframe::egui::Painter, pos: Pos2, text: &str, p: &Palette) {
    painter.text(pos, Align2::LEFT_TOP, text, font_semibold(15.0), p.ink);
}

fn label(painter: &eframe::egui::Painter, pos: Pos2, text: &str, p: &Palette) {
    painter.text(pos, Align2::LEFT_TOP, text, font_caption(), p.muted);
}

fn buttons(ui: &mut Ui, rect: Rect, p: &Palette, lang: Lang, with_cancel: bool) -> Option<DialogEvent> {
    let mut event = None;
    let top = rect.bottom() - 72.0;
    let ok_rect = Rect::from_min_size(Pos2::new(rect.left() + 28.0, top), Vec2::new(160.0, 44.0));
    if widgets::button(
        ui,
        ok_rect,
        "dlg-ok",
        i18n::t(lang, Text::Ok),
        BtnStyle::Primary,
        None,
        0.0,
        None,
        true,
        p,
    )
    .clicked()
    {
        event = Some(DialogEvent::Ok);
    }

    if with_cancel {
        let cancel_rect = Rect::from_min_size(
            Pos2::new(rect.left() + 212.0, top),
            Vec2::new(160.0, 44.0),
        );
        if widgets::button(
            ui,
            cancel_rect,
            "dlg-cancel",
            i18n::t(lang, Text::Cancel),
            BtnStyle::Secondary,
            None,
            0.0,
            None,
            true,
            p,
        )
        .clicked()
        {
            event = Some(DialogEvent::Cancel);
        }
    }

    event
}

impl Dialog {
    /// Draws the dimmed backdrop and the dialog itself. Returns the user's
    /// decision for this frame, if any. `appear` fades and lifts it in (0..1).
    pub fn show(
        &mut self,
        ctx: &Context,
        p: &Palette,
        lang: Lang,
        appear: f32,
    ) -> Option<DialogEvent> {
        let screen = ctx.viewport_rect();
        let dim = ctx.layer_painter(LayerId::new(Order::Middle, Id::unique("dialog-dim")));
        dim.rect_filled(screen, CornerRadius::ZERO, Color32::from_black_alpha((110.0 * appear) as u8));

        match self {
            Dialog::Info { text, warning } => {
                let text = text.clone();
                let warning = *warning;
                let width = 400.0;
                let galley = ctx.fonts_mut(|f| f.layout(text, font_body(), p.ink, width - 56.0));
                let text_height = galley.size().y.max(20.0);
                let height = 28.0 + text_height + 26.0 + 44.0 + 26.0;
                let rect = centered(screen, Vec2::new(width, height), appear);

                let mut event = None;
                if ctx.input(|i| i.key_pressed(Key::Escape) || i.key_pressed(Key::Enter)) {
                    event = Some(DialogEvent::Ok);
                }

                Area::new(Id::unique("dialog"))
                    .order(Order::Foreground)
                    .fixed_pos(rect.min)
                    .show(ctx, |ui| {
                        ui.set_opacity(appear);
                        let painter = ui.painter().clone();
                        panel(
                            &painter,
                            rect,
                            p,
                            if warning { p.danger } else { p.accent },
                        );
                        painter.galley(
                            Pos2::new(rect.left() + 28.0, rect.top() + 28.0),
                            galley.clone(),
                            p.ink,
                        );

                        let ok_rect = Rect::from_min_size(
                            Pos2::new(
                                rect.right() - 28.0 - 140.0,
                                rect.bottom() - 72.0,
                            ),
                            Vec2::new(140.0, 44.0),
                        );
                        if widgets::button(
                            ui,
                            ok_rect,
                            "dlg-ok",
                            i18n::t(lang, Text::Ok),
                            BtnStyle::Primary,
                            None,
                            0.0,
                            None,
                            true,
                            p,
                        )
                        .clicked()
                        {
                            event = Some(DialogEvent::Ok);
                        }
                    });

                event
            }

            Dialog::Hotkey { value } => {
                if let Some(event) = capture_single_key(ctx, value) {
                    return Some(event);
                }

                let rect = centered(screen, Vec2::new(400.0, 232.0), appear);
                let mut event = None;
                Area::new(Id::unique("dialog"))
                    .order(Order::Foreground)
                    .fixed_pos(rect.min)
                    .show(ctx, |ui| {
                        ui.set_opacity(appear);
                        let painter = ui.painter().clone();
                        panel(&painter, rect, p, p.accent);
                        title(
                            &painter,
                            Pos2::new(rect.left() + 28.0, rect.top() + 20.0),
                            i18n::t(lang, Text::DialogHotkey),
                            p,
                        );
                        label(
                            &painter,
                            Pos2::new(rect.left() + 28.0, rect.top() + 52.0),
                            i18n::t(lang, Text::StartStop),
                            p,
                        );

                        let field = Rect::from_min_size(
                            Pos2::new(rect.left() + 28.0, rect.top() + 68.0),
                            Vec2::new(344.0, 40.0),
                        );
                        widgets::key_field(ui, field, "start-stop-key", &key_name(*value), true, p);

                        if let Some(result) = buttons(ui, rect, p, lang, true) {
                            event = Some(result);
                        }
                    });
                event
            }

            Dialog::AdvSettings {
                minimize_play,
                minimize_record,
            } => {
                let rect = centered(screen, Vec2::new(400.0, 240.0), appear);
                let mut event = None;
                Area::new(Id::unique("dialog"))
                    .order(Order::Foreground)
                    .fixed_pos(rect.min)
                    .show(ctx, |ui| {
                        ui.set_opacity(appear);
                        let painter = ui.painter().clone();
                        panel(&painter, rect, p, p.accent);
                        title(
                            &painter,
                            Pos2::new(rect.left() + 28.0, rect.top() + 20.0),
                            i18n::t(lang, Text::DialogSettings),
                            p,
                        );

                        let play_rect = Rect::from_min_size(
                            Pos2::new(rect.left() + 28.0, rect.top() + 60.0),
                            Vec2::new(344.0, 24.0),
                        );
                        if widgets::checkbox(
                            ui,
                            play_rect,
                            "min-play",
                            i18n::t(lang, Text::MinimizePlaying),
                            *minimize_play,
                            true,
                            p,
                        )
                        .clicked()
                        {
                            *minimize_play = !*minimize_play;
                        }

                        let record_rect = Rect::from_min_size(
                            Pos2::new(rect.left() + 28.0, rect.top() + 96.0),
                            Vec2::new(344.0, 24.0),
                        );
                        if widgets::checkbox(
                            ui,
                            record_rect,
                            "min-record",
                            i18n::t(lang, Text::MinimizeRecording),
                            *minimize_record,
                            true,
                            p,
                        )
                        .clicked()
                        {
                            *minimize_record = !*minimize_record;
                        }

                        if let Some(result) = buttons(ui, rect, p, lang, true) {
                            event = Some(result);
                        }
                    });
                event
            }

            Dialog::AdvHotkeys { play, record, active } => {
                if let Some(event) = capture_double_key(ctx, play, record, active) {
                    return Some(event);
                }

                let rect = centered(screen, Vec2::new(400.0, 284.0), appear);
                let mut event = None;
                Area::new(Id::unique("dialog"))
                    .order(Order::Foreground)
                    .fixed_pos(rect.min)
                    .show(ctx, |ui| {
                        ui.set_opacity(appear);
                        let painter = ui.painter().clone();
                        panel(&painter, rect, p, p.accent);
                        title(
                            &painter,
                            Pos2::new(rect.left() + 28.0, rect.top() + 20.0),
                            i18n::t(lang, Text::DialogHotkeys),
                            p,
                        );

                        label(
                            &painter,
                            Pos2::new(rect.left() + 28.0, rect.top() + 50.0),
                            i18n::t(lang, Text::PlaybackStop),
                            p,
                        );
                        let play_rect = Rect::from_min_size(
                            Pos2::new(rect.left() + 28.0, rect.top() + 66.0),
                            Vec2::new(344.0, 40.0),
                        );
                        if widgets::key_field(
                            ui,
                            play_rect,
                            "play-key",
                            &key_name(*play),
                            *active == 0,
                            p,
                        )
                        .clicked()
                        {
                            *active = 0;
                        }

                        label(
                            &painter,
                            Pos2::new(rect.left() + 28.0, rect.top() + 122.0),
                            i18n::t(lang, Text::RecordStop),
                            p,
                        );
                        let record_rect = Rect::from_min_size(
                            Pos2::new(rect.left() + 28.0, rect.top() + 138.0),
                            Vec2::new(344.0, 40.0),
                        );
                        if widgets::key_field(
                            ui,
                            record_rect,
                            "record-key",
                            &key_name(*record),
                            *active == 1,
                            p,
                        )
                        .clicked()
                        {
                            *active = 1;
                        }

                        if let Some(result) = buttons(ui, rect, p, lang, true) {
                            event = Some(result);
                        }
                    });
                event
            }

            Dialog::Pick { deadline, result } => {
                if result.is_none() && Instant::now() >= *deadline {
                    let (x, y) = native::cursor_pos();
                    *result = Some((x, y));
                    return Some(DialogEvent::Ok);
                }

                let rect = centered(screen, Vec2::new(380.0, 168.0), appear);
                let remaining = deadline
                    .saturating_duration_since(Instant::now())
                    .as_secs_f32();
                let fraction = (remaining / 3.0).clamp(0.0, 1.0);
                let seconds = remaining.ceil().max(1.0) as i32;

                Area::new(Id::unique("dialog"))
                    .order(Order::Foreground)
                    .fixed_pos(rect.min)
                    .show(ctx, |ui| {
                        ui.set_opacity(appear);
                        let painter = ui.painter().clone();
                        panel(&painter, rect, p, p.accent);
                        title(
                            &painter,
                            Pos2::new(rect.left() + 28.0, rect.top() + 18.0),
                            i18n::t(lang, Text::DialogPick),
                            p,
                        );

                        let hint = i18n::t(lang, Text::PickHint);
                        let galley =
                            painter.layout(hint.to_owned(), font_body(), p.muted, 324.0);
                        painter.galley(
                            Pos2::new(rect.center().x - galley.size().x * 0.5, rect.top() + 44.0),
                            galley,
                            p.muted,
                        );

                        let center = Pos2::new(rect.center().x, rect.top() + 116.0);
                        let radius = 22.0;
                        for i in 0..=44 {
                            let angle = -std::f32::consts::FRAC_PI_2
                                + i as f32 * std::f32::consts::TAU / 44.0;
                            painter.circle_filled(
                                center + Vec2::new(angle.cos(), angle.sin()) * radius,
                                2.0,
                                fade(p.accent, 0.15),
                            );
                        }
                        let steps = (fraction * 44.0).round() as i32;
                        for i in 0..=steps {
                            let angle = -std::f32::consts::FRAC_PI_2
                                + i as f32 * std::f32::consts::TAU / 44.0;
                            painter.circle_filled(
                                center + Vec2::new(angle.cos(), angle.sin()) * radius,
                                2.4,
                                p.accent,
                            );
                        }
                        painter.text(
                            center,
                            Align2::CENTER_CENTER,
                            seconds.to_string(),
                            font_semibold(22.0),
                            p.ink,
                        );
                    });

                None
            }
        }
    }
}

fn centered(screen: Rect, size: Vec2, appear: f32) -> Rect {
    let slide = (1.0 - appear) * 12.0;
    Rect::from_center_size(screen.center() + Vec2::new(0.0, slide), size)
}

fn pressed_key(ctx: &Context) -> Option<Key> {
    ctx.input(|i| {
        i.events.iter().find_map(|event| match event {
            Event::Key {
                key,
                pressed: true,
                repeat: false,
                ..
            } => Some(*key),
            _ => None,
        })
    })
}

fn capture_single_key(ctx: &Context, value: &mut u32) -> Option<DialogEvent> {
    let key = pressed_key(ctx)?;
    match key {
        Key::Escape => Some(DialogEvent::Cancel),
        Key::Enter => Some(DialogEvent::Ok),
        _ => {
            if let Some(vk) = key_to_vk(key) {
                *value = vk;
            }
            None
        }
    }
}

fn capture_double_key(
    ctx: &Context,
    play: &mut u32,
    record: &mut u32,
    active: &usize,
) -> Option<DialogEvent> {
    let key = pressed_key(ctx)?;
    match key {
        Key::Escape => Some(DialogEvent::Cancel),
        Key::Enter => Some(DialogEvent::Ok),
        _ => {
            if let Some(vk) = key_to_vk(key) {
                if *active == 0 {
                    *play = vk;
                } else {
                    *record = vk;
                }
            }
            None
        }
    }
}
