//! Themed modal dialogs drawn inside the main viewport (no extra windows):
//! messages, hotkey capture, view settings and the pick-location countdown.

use std::time::Instant;

use eframe::egui::{
    Align2, Area, Color32, Context, CornerRadius, Event, Id, Key, LayerId, Order, Pos2, Rect,
    Stroke, StrokeKind, Ui, Vec2,
};

use crate::keys::{key_name, key_to_vk};
use crate::native;
use crate::theme::{font_body, font_caption, font_semibold, Palette, RADIUS_CARD};
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

fn frame(painter: &eframe::egui::Painter, rect: Rect, p: &Palette, bar: Color32) {
    painter.rect_filled(
        rect.translate(Vec2::new(0.0, 6.0)).expand(2.0),
        CornerRadius::same(14),
        Color32::from_black_alpha(45),
    );
    painter.rect_filled(rect, CornerRadius::same(RADIUS_CARD), p.bg);
    painter.rect_stroke(
        rect,
        CornerRadius::same(RADIUS_CARD),
        Stroke::new(1.0, p.border),
        StrokeKind::Inside,
    );

    let strip = Rect::from_min_max(rect.min, Pos2::new(rect.right(), rect.top() + 5.0));
    painter.rect_filled(
        strip,
        CornerRadius {
            nw: 12,
            ne: 12,
            sw: 0,
            se: 0,
        },
        bar,
    );
}

fn title(painter: &eframe::egui::Painter, pos: Pos2, text: &str, p: &Palette) {
    painter.text(
        pos,
        Align2::LEFT_TOP,
        text,
        font_semibold(14.0),
        p.ink,
    );
}

fn label(painter: &eframe::egui::Painter, pos: Pos2, text: &str, p: &Palette) {
    painter.text(pos, Align2::LEFT_TOP, text, font_caption(), p.muted);
}

fn ok_cancel(
    ui: &mut Ui,
    left: f32,
    top: f32,
    p: &Palette,
    with_cancel: bool,
) -> Option<DialogEvent> {
    let mut event = None;
    let ok_rect = Rect::from_min_size(Pos2::new(left, top), Vec2::new(152.0, 42.0));
    if widgets::button(ui, ok_rect, "dlg-ok", "Ok", BtnStyle::Primary, None, 0.0, None, true, p)
        .clicked()
    {
        event = Some(DialogEvent::Ok);
    }

    if with_cancel {
        let cancel_rect =
            Rect::from_min_size(Pos2::new(left + 172.0, top), Vec2::new(152.0, 42.0));
        if widgets::button(
            ui,
            cancel_rect,
            "dlg-cancel",
            "Cancel",
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
    /// Title of the window while the dialog is shown on the Advanced screen.
    pub fn title(&self) -> &'static str {
        match self {
            Dialog::Info { .. } => "Simple Auto Clicker Advanced",
            Dialog::Hotkey { .. } => "Hotkey Setting",
            Dialog::AdvSettings { .. } => "View Setting",
            Dialog::AdvHotkeys { .. } => "Hotkeys Setting",
            Dialog::Pick { .. } => "Pick location",
        }
    }

    /// Size the dialog needs; used to grow the viewport on the small Advanced
    /// screen so the dialog fits with a margin around it.
    pub fn size(&self, ctx: &Context) -> Vec2 {
        match self {
            Dialog::Info { text, .. } => {
                let galley = ctx.fonts_mut(|f| f.layout(text.clone(), font_body(), Color32::WHITE, 352.0));
                Vec2::new(404.0, 28.0 + galley.size().y.max(20.0) + 22.0 + 42.0 + 24.0)
            }
            Dialog::Hotkey { .. } => Vec2::new(380.0, 210.0),
            Dialog::AdvSettings { .. } => Vec2::new(380.0, 220.0),
            Dialog::AdvHotkeys { .. } => Vec2::new(380.0, 250.0),
            Dialog::Pick { .. } => Vec2::new(360.0, 130.0),
        }
    }

    /// Draws the dimmed backdrop and the dialog itself. Returns the user's
    /// decision for this frame, if any.
    pub fn show(&mut self, ctx: &Context, p: &Palette) -> Option<DialogEvent> {
        let screen = ctx.viewport_rect();
        let dim = ctx.layer_painter(LayerId::new(Order::Middle, Id::unique("dialog-dim")));
        dim.rect_filled(screen, CornerRadius::ZERO, Color32::from_black_alpha(96));

        match self {
            Dialog::Info { text, warning } => {
                let text = text.clone();
                let warning = *warning;
                let width = 404.0;
                let galley =
                    ctx.fonts_mut(|f| f.layout(text, font_body(), p.ink, width - 52.0));
                let text_height = galley.size().y.max(20.0);
                let height = 28.0 + text_height + 22.0 + 42.0 + 24.0;
                let rect = Rect::from_center_size(screen.center(), Vec2::new(width, height));

                let mut event = None;
                if ctx.input(|i| i.key_pressed(Key::Escape) || i.key_pressed(Key::Enter)) {
                    event = Some(DialogEvent::Ok);
                }

                Area::new(Id::unique("dialog"))
                    .order(Order::Foreground)
                    .fixed_pos(rect.min)
                    .show(ctx, |ui| {
                        let painter = ui.painter().clone();
                        frame(
                            &painter,
                            rect,
                            p,
                            if warning { p.danger } else { p.accent },
                        );
                        painter.galley(
                            Pos2::new(rect.left() + 26.0, rect.top() + 28.0),
                            galley.clone(),
                            p.ink,
                        );

                        let ok_rect = Rect::from_min_size(
                            Pos2::new(rect.right() - 26.0 - 140.0, rect.top() + 28.0 + text_height + 22.0),
                            Vec2::new(140.0, 42.0),
                        );
                        if widgets::button(
                            ui, ok_rect, "dlg-ok", "Ok", BtnStyle::Primary, None, 0.0, None, true, p,
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

                let rect = Rect::from_center_size(screen.center(), Vec2::new(380.0, 210.0));
                let mut event = None;
                Area::new(Id::unique("dialog"))
                    .order(Order::Foreground)
                    .fixed_pos(rect.min)
                    .show(ctx, |ui| {
                        let painter = ui.painter().clone();
                        frame(&painter, rect, p, p.accent);
                        title(&painter, Pos2::new(rect.left() + 28.0, rect.top() + 18.0), "Hotkey Setting", p);
                        label(
                            &painter,
                            Pos2::new(rect.left() + 28.0, rect.top() + 50.0),
                            "Start / Stop",
                            p,
                        );

                        let field = Rect::from_min_size(
                            Pos2::new(rect.left() + 28.0, rect.top() + 68.0),
                            Vec2::new(324.0, 38.0),
                        );
                        widgets::key_field(ui, field, "start-stop-key", &key_name(*value), true, p);

                        if let Some(result) =
                            ok_cancel(ui, rect.left() + 28.0, rect.top() + 142.0, p, true)
                        {
                            event = Some(result);
                        }
                    });
                event
            }

            Dialog::AdvSettings {
                minimize_play,
                minimize_record,
            } => {
                let rect = Rect::from_center_size(screen.center(), Vec2::new(380.0, 220.0));
                let mut event = None;
                Area::new(Id::unique("dialog"))
                    .order(Order::Foreground)
                    .fixed_pos(rect.min)
                    .show(ctx, |ui| {
                        let painter = ui.painter().clone();
                        frame(&painter, rect, p, p.accent);
                        title(&painter, Pos2::new(rect.left() + 28.0, rect.top() + 18.0), "View Setting", p);

                        let play_rect = Rect::from_min_size(
                            Pos2::new(rect.left() + 28.0, rect.top() + 58.0),
                            Vec2::new(324.0, 24.0),
                        );
                        if widgets::checkbox(
                            ui,
                            play_rect,
                            "min-play",
                            "Minimized when playing",
                            *minimize_play,
                            true,
                            p,
                        )
                        .clicked()
                        {
                            *minimize_play = !*minimize_play;
                        }

                        let record_rect = Rect::from_min_size(
                            Pos2::new(rect.left() + 28.0, rect.top() + 94.0),
                            Vec2::new(324.0, 24.0),
                        );
                        if widgets::checkbox(
                            ui,
                            record_rect,
                            "min-record",
                            "Minimized when recording",
                            *minimize_record,
                            true,
                            p,
                        )
                        .clicked()
                        {
                            *minimize_record = !*minimize_record;
                        }

                        if let Some(result) =
                            ok_cancel(ui, rect.left() + 28.0, rect.top() + 150.0, p, true)
                        {
                            event = Some(result);
                        }
                    });
                event
            }

            Dialog::AdvHotkeys { play, record, active } => {
                if let Some(event) = capture_double_key(ctx, play, record, active) {
                    return Some(event);
                }

                let rect = Rect::from_center_size(screen.center(), Vec2::new(380.0, 250.0));
                let mut event = None;
                Area::new(Id::unique("dialog"))
                    .order(Order::Foreground)
                    .fixed_pos(rect.min)
                    .show(ctx, |ui| {
                        let painter = ui.painter().clone();
                        frame(&painter, rect, p, p.accent);
                        title(&painter, Pos2::new(rect.left() + 28.0, rect.top() + 18.0), "Hotkeys Setting", p);

                        label(
                            &painter,
                            Pos2::new(rect.left() + 28.0, rect.top() + 46.0),
                            "Playback / Stop",
                            p,
                        );
                        let play_rect = Rect::from_min_size(
                            Pos2::new(rect.left() + 28.0, rect.top() + 64.0),
                            Vec2::new(324.0, 38.0),
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
                            Pos2::new(rect.left() + 28.0, rect.top() + 110.0),
                            "Record / Stop",
                            p,
                        );
                        let record_rect = Rect::from_min_size(
                            Pos2::new(rect.left() + 28.0, rect.top() + 128.0),
                            Vec2::new(324.0, 38.0),
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

                        if let Some(result) =
                            ok_cancel(ui, rect.left() + 28.0, rect.top() + 182.0, p, true)
                        {
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

                let rect = Rect::from_center_size(screen.center(), Vec2::new(360.0, 130.0));
                let left = deadline
                    .saturating_duration_since(Instant::now())
                    .as_secs_f32()
                    .ceil() as i32;
                let seconds = left.max(1);

                Area::new(Id::unique("dialog"))
                    .order(Order::Foreground)
                    .fixed_pos(rect.min)
                    .show(ctx, |ui| {
                        let painter = ui.painter().clone();
                        frame(&painter, rect, p, p.accent);
                        painter.text(
                            Pos2::new(rect.center().x, rect.top() + 18.0),
                            Align2::CENTER_TOP,
                            "Point the cursor at the spot you need",
                            font_body(),
                            p.muted,
                        );
                        painter.text(
                            Pos2::new(rect.center().x, rect.top() + 76.0),
                            Align2::CENTER_CENTER,
                            seconds.to_string(),
                            font_semibold(34.0),
                            p.accent,
                        );
                    });

                None
            }
        }
    }
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
