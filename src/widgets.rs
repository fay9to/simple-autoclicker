//! Hand-drawn controls: glass cards, animated buttons, radios, checkboxes,
//! rounded fields and the popup menus.

use eframe::egui::{
    Align2, Area, Color32, Context, CornerRadius, Id, Margin, Order, Pos2, Rect, Response, Sense,
    Shape, Stroke, StrokeKind, TextEdit, Ui, Vec2,
};

use crate::theme::{
    draw_icon, fade, font_body, font_caption, font_field, font_small, font_semibold, glow, mix, soft_shadow,
    vertical_gradient, IconKind, Palette, RADIUS_BUTTON, RADIUS_CARD, RADIUS_CONTROL,
};

// ------------------------------------------------------------------- surfaces

pub fn card(painter: &eframe::egui::Painter, rect: Rect, title: &str, p: &Palette) {
    soft_shadow(painter, rect, RADIUS_CARD, p.shadow, 9.0, 4.0);
    painter.rect_filled(rect, CornerRadius::same(RADIUS_CARD), p.card);
    painter.rect_stroke(
        rect,
        CornerRadius::same(RADIUS_CARD),
        Stroke::new(1.0, p.card_border),
        StrokeKind::Inside,
    );
    painter.line_segment(
        [
            Pos2::new(rect.left() + 18.0, rect.top() + 0.5),
            Pos2::new(rect.right() - 18.0, rect.top() + 0.5),
        ],
        Stroke::new(1.0, fade(p.card_highlight, 0.30)),
    );

    if !title.is_empty() {
        painter.text(
            Pos2::new(rect.left() + 18.0, rect.top() + 21.0),
            Align2::LEFT_CENTER,
            title,
            font_semibold(13.0),
            p.ink,
        );
    }
}

pub fn caption(painter: &eframe::egui::Painter, center: Pos2, text: &str, p: &Palette) {
    painter.text(center, Align2::CENTER_CENTER, text, font_caption(), p.muted);
}

pub fn caption_left(painter: &eframe::egui::Painter, left_center: Pos2, text: &str, p: &Palette) {
    painter.text(left_center, Align2::LEFT_CENTER, text, font_caption(), p.muted);
}

// -------------------------------------------------------------------- buttons

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BtnStyle {
    /// Gradient accent button used for the main action.
    Hero,
    /// Solid accent button.
    Primary,
    /// Glass surface with a border.
    Secondary,
    /// Soft red surface.
    Danger,
    /// Tinted surface without a border.
    Ghost,
    /// Transparent until hovered; used for the title-bar icons.
    Bare,
}

struct Look {
    top: Color32,
    bottom: Color32,
    fore: Color32,
    border: Option<Color32>,
    gradient: bool,
}

fn look(style: BtnStyle, enabled: bool, hover: f32, down: f32, p: &Palette) -> Look {
    if !enabled {
        return match style {
            BtnStyle::Hero | BtnStyle::Primary => Look {
                top: p.disabled_surface,
                bottom: p.disabled_surface,
                fore: p.disabled_ink,
                border: None,
                gradient: false,
            },
            BtnStyle::Bare => Look {
                top: Color32::TRANSPARENT,
                bottom: Color32::TRANSPARENT,
                fore: p.disabled_ink,
                border: None,
                gradient: false,
            },
            BtnStyle::Ghost => Look {
                top: p.soft,
                bottom: p.soft,
                fore: p.disabled_ink,
                border: None,
                gradient: false,
            },
            _ => Look {
                top: p.card,
                bottom: p.card,
                fore: p.disabled_ink,
                border: Some(p.card_border),
                gradient: false,
            },
        };
    }

    match style {
        BtnStyle::Hero => {
            let base = mix(p.accent, Color32::WHITE, 0.10 + 0.10 * hover);
            let bottom = mix(p.accent, p.accent2, 0.45 + 0.25 * hover);
            let top = mix(base, Color32::WHITE, 0.10);
            let (top, bottom) = if down > 0.5 {
                (mix(top, Color32::BLACK, 0.06), mix(bottom, Color32::BLACK, 0.06))
            } else {
                (top, bottom)
            };
            Look {
                top,
                bottom,
                fore: p.on_accent,
                border: None,
                gradient: true,
            }
        }
        BtnStyle::Primary => {
            let base = mix(p.accent, p.accent2, 0.30);
            let base = if hover > 0.0 { mix(base, Color32::WHITE, 0.14 * hover) } else { base };
            let base = if down > 0.5 { mix(base, Color32::BLACK, 0.08) } else { base };
            Look {
                top: mix(base, Color32::WHITE, 0.12),
                bottom: base,
                fore: p.on_accent,
                border: None,
                gradient: true,
            }
        }
        BtnStyle::Danger => Look {
            top: mix(p.danger_soft, p.danger_hover, hover),
            bottom: mix(p.danger_soft, p.danger_hover, hover),
            fore: p.danger,
            border: Some(fade(p.danger, 0.30)),
            gradient: false,
        },
        BtnStyle::Ghost => Look {
            top: mix(p.soft, p.soft_hover, hover),
            bottom: mix(p.soft, p.soft_hover, hover),
            fore: p.ink,
            border: None,
            gradient: false,
        },
        BtnStyle::Bare => Look {
            top: fade(p.soft_hover, hover),
            bottom: fade(p.soft_hover, hover),
            fore: if hover > 0.5 { p.ink } else { p.ink },
            border: None,
            gradient: false,
        },
        BtnStyle::Secondary => Look {
            top: mix(p.card, p.soft_hover, hover.max(0.15)),
            bottom: mix(p.card, p.soft_hover, hover.max(0.15)),
            fore: p.ink,
            border: Some(mix(p.card_border, p.accent, hover * 0.35)),
            gradient: false,
        },
    }
}

#[allow(clippy::too_many_arguments)]
pub fn button(
    ui: &mut Ui,
    rect: Rect,
    key: &str,
    text: &str,
    style: BtnStyle,
    icon: Option<IconKind>,
    icon_size: f32,
    icon_color: Option<Color32>,
    enabled: bool,
    p: &Palette,
) -> Response {
    let id = Id::unique(("button", key));
    let sense = if enabled { Sense::click() } else { Sense::hover() };
    let response = ui.interact(rect, id, sense);
    let ctx = ui.ctx();

    let hover = if enabled {
        ctx.animate_bool_with_time(id.with("hover"), response.hovered(), 0.14)
    } else {
        0.0
    };
    let down = if enabled {
        ctx.animate_bool_with_time(id.with("down"), response.is_pointer_button_down_on(), 0.08)
    } else {
        0.0
    };

    let look = look(style, enabled, hover, down, p);
    let scale = 1.0 - 0.015 * down;
    let body = Rect::from_center_size(rect.center(), rect.size() * scale);

    let painter = ui.painter();
    if style == BtnStyle::Hero {
        glow(painter, body, RADIUS_BUTTON, p.accent, 0.10 + 0.32 * hover);
    }

    if look.gradient {
        vertical_gradient(painter, body, RADIUS_BUTTON, look.top, look.bottom);
    } else if look.top.a() > 0 {
        painter.rect_filled(body, CornerRadius::same(RADIUS_BUTTON), look.top);
    }
    if let Some(border) = look.border {
        painter.rect_stroke(
            body,
            CornerRadius::same(RADIUS_BUTTON),
            Stroke::new(1.0, border),
            StrokeKind::Inside,
        );
    }

    let shift = Vec2::new(0.0, down * 0.8);
    let color = icon_color.unwrap_or(look.fore);
    // The crescent moon is a circle with a bite taken out, so the bite must be
    // filled with the surface behind the icon rather than transparency.
    let erase = if look.top.a() > 200 {
        look.top
    } else {
        crate::theme::opaque(p.bg_top)
    };
    let font = font_semibold(if style == BtnStyle::Hero { 13.4 } else { 12.6 });
    let galley = painter.layout_no_wrap(text.to_owned(), font, look.fore);

    match icon {
        Some(kind) => {
            if text.is_empty() {
                let icon_rect = Rect::from_center_size(body.center() + shift, Vec2::splat(icon_size));
                draw_icon(painter, icon_rect, kind, color, erase);
            } else {
                let gap = 9.0;
                let total = icon_size + gap + galley.size().x;
                let start = body.center().x - total * 0.5;
                let icon_rect = Rect::from_center_size(
                    Pos2::new(start + icon_size * 0.5, body.center().y) + shift,
                    Vec2::splat(icon_size),
                );
                draw_icon(painter, icon_rect, kind, color, erase);
                painter.galley(
                    Pos2::new(
                        start + icon_size + gap,
                        body.center().y - galley.size().y * 0.5,
                    ) + shift,
                    galley,
                    look.fore,
                );
            }
        }
        None => {
            painter.galley(
                body.center() - galley.size() * 0.5 + shift,
                galley,
                look.fore,
            );
        }
    }

    response
}

/// Square icon button used in the title bar.
pub fn icon_button(
    ui: &mut Ui,
    rect: Rect,
    key: &str,
    icon: IconKind,
    size: f32,
    enabled: bool,
    p: &Palette,
) -> Response {
    button(
        ui,
        rect,
        key,
        "",
        BtnStyle::Bare,
        Some(icon),
        size,
        None,
        enabled,
        p,
    )
}

// --------------------------------------------------------------------- inputs

pub fn radio(
    ui: &mut Ui,
    rect: Rect,
    key: &str,
    text: &str,
    selected: bool,
    enabled: bool,
    p: &Palette,
) -> Response {
    let id = Id::unique(("radio", key));
    let sense = if enabled { Sense::click() } else { Sense::hover() };
    let response = ui.interact(rect, id, sense);
    let ctx = ui.ctx();

    let sel = ctx.animate_bool_with_time(id.with("sel"), selected, 0.16);
    let hover = if enabled {
        ctx.animate_bool_with_time(id.with("hover"), response.hovered(), 0.12)
    } else {
        0.0
    };

    let painter = ui.painter();
    if hover > 0.01 && enabled {
        painter.rect_filled(
            rect.expand2(Vec2::new(7.0, 2.0)),
            CornerRadius::same(8),
            fade(p.soft_hover, hover),
        );
    }

    let diameter = 16.0;
    let circle = Rect::from_center_size(
        Pos2::new(rect.left() + 1.0 + diameter * 0.5, rect.center().y),
        Vec2::splat(diameter),
    );

    painter.circle_filled(
        circle.center(),
        diameter * 0.5,
        if enabled { p.field } else { p.soft },
    );
    painter.circle_stroke(
        circle.center(),
        diameter * 0.5 - 0.8,
        Stroke::new(1.6, mix(p.field_border, p.accent, sel)),
    );
    if sel > 0.01 {
        painter.circle_filled(circle.center(), 4.4 * sel, fade(p.accent, 0.35 + 0.65 * sel));
    }

    if !text.is_empty() {
        let color = if enabled { p.ink } else { p.muted };
        painter.text(
            Pos2::new(circle.right() + 9.0, rect.center().y),
            Align2::LEFT_CENTER,
            text,
            font_body(),
            color,
        );
    }

    response
}

pub fn checkbox(
    ui: &mut Ui,
    rect: Rect,
    key: &str,
    text: &str,
    checked: bool,
    enabled: bool,
    p: &Palette,
) -> Response {
    let id = Id::unique(("checkbox", key));
    let sense = if enabled { Sense::click() } else { Sense::hover() };
    let response = ui.interact(rect, id, sense);
    let ctx = ui.ctx();

    let t = ctx.animate_bool_with_time(id.with("check"), checked, 0.16);
    let hover = if enabled {
        ctx.animate_bool_with_time(id.with("hover"), response.hovered(), 0.12)
    } else {
        0.0
    };

    let painter = ui.painter();
    if hover > 0.01 && enabled {
        painter.rect_filled(
            rect.expand2(Vec2::new(7.0, 2.0)),
            CornerRadius::same(8),
            fade(p.soft_hover, hover),
        );
    }

    let size = 18.0;
    let box_rect = Rect::from_center_size(
        Pos2::new(rect.left() + 1.0 + size * 0.5, rect.center().y),
        Vec2::splat(size),
    );

    painter.rect_filled(
        box_rect,
        CornerRadius::same(5),
        if enabled {
            mix(p.field, p.accent, t)
        } else {
            p.soft
        },
    );
    painter.rect_stroke(
        box_rect,
        CornerRadius::same(5),
        Stroke::new(
            1.6,
            mix(if enabled { p.field_border } else { p.card_border }, p.accent, t),
        ),
        StrokeKind::Inside,
    );

    if t > 0.05 {
        let points = vec![
            Pos2::new(box_rect.left() + size * 0.24, box_rect.top() + size * 0.52),
            Pos2::new(box_rect.left() + size * 0.44, box_rect.top() + size * 0.72),
            Pos2::new(box_rect.left() + size * 0.78, box_rect.top() + size * 0.30),
        ];
        painter.add(Shape::line(
            points,
            Stroke::new(2.2, fade(p.on_accent, t)),
        ));
    }

    let color = if enabled { p.ink } else { p.muted };
    painter.text(
        Pos2::new(box_rect.right() + 10.0, rect.center().y),
        Align2::LEFT_CENTER,
        text,
        font_body(),
        color,
    );

    response
}

pub fn select_field(
    ui: &mut Ui,
    rect: Rect,
    key: &str,
    text: &str,
    open: bool,
    enabled: bool,
    p: &Palette,
) -> Response {
    let id = Id::unique(("select", key));
    let sense = if enabled { Sense::click() } else { Sense::hover() };
    let response = ui.interact(rect, id, sense);
    let ctx = ui.ctx();

    let hover = if enabled {
        ctx.animate_bool_with_time(id.with("hover"), response.hovered(), 0.14)
    } else {
        0.0
    };
    let open_t = ctx.animate_bool_with_time(id.with("open"), open, 0.18);

    let painter = ui.painter();
    painter.rect_filled(
        rect,
        CornerRadius::same(RADIUS_CONTROL),
        if enabled { p.field } else { p.soft },
    );

    let active = (hover.max(open_t) * 0.75).clamp(0.0, 1.0);
    let border = mix(p.field_border, p.border_focus, active);
    painter.rect_stroke(
        rect,
        CornerRadius::same(RADIUS_CONTROL),
        Stroke::new(1.0 + 0.6 * active, border),
        StrokeKind::Inside,
    );
    if active > 0.05 {
        glow(painter, rect, RADIUS_CONTROL, p.accent, 0.16 * active);
    }

    painter.text(
        Pos2::new(rect.left() + 13.0, rect.center().y),
        Align2::LEFT_CENTER,
        text,
        font_body(),
        if enabled { p.ink } else { p.muted },
    );

    let c = Pos2::new(rect.right() - 17.0, rect.center().y);
    let color = mix(p.muted, p.accent, active);
    let stroke = Stroke::new(1.7, color);
    let angle = std::f32::consts::PI * open_t;
    let (sin, cos) = angle.sin_cos();
    let rotate = |p: Pos2| {
        let d = p - c;
        c + Vec2::new(d.x * cos - d.y * sin, d.x * sin + d.y * cos)
    };
    let a = rotate(c + Vec2::new(-4.5, -2.0));
    let b = rotate(c + Vec2::new(0.0, 2.5));
    let d = rotate(c + Vec2::new(4.5, -2.0));
    painter.add(Shape::line(vec![a, b, d], stroke));

    response
}

pub fn number_field(
    ui: &mut Ui,
    rect: Rect,
    key: &str,
    text: &mut String,
    enabled: bool,
    p: &Palette,
) -> Response {
    let painter = ui.painter().clone();
    painter.rect_filled(
        rect,
        CornerRadius::same(RADIUS_CONTROL),
        if enabled { p.field } else { p.soft },
    );

    let response = if enabled {
        let inner = Rect::from_min_max(
            Pos2::new(rect.left() + 8.0, rect.center().y - 10.0),
            Pos2::new(rect.right() - 8.0, rect.center().y + 10.0),
        );
        let response = ui.put(
            inner,
            TextEdit::singleline(text)
                .frame(eframe::egui::Frame::NONE)
                .align(Align2::CENTER_CENTER)
                .font(font_field())
                .text_color(p.ink)
                .margin(Margin::symmetric(0, 2)),
        );
        text.retain(|c| c.is_ascii_digit());
        if text.len() > 9 {
            text.truncate(9);
        }
        response
    } else {
        painter.text(
            rect.center(),
            Align2::CENTER_CENTER,
            text.as_str(),
            font_field(),
            p.muted,
        );
        ui.interact(rect, Id::unique(("number", key)), Sense::hover())
    };

    let focus = if enabled {
        ui.ctx()
            .animate_bool_with_time(Id::unique(("number", key, "focus")), response.has_focus(), 0.14)
    } else {
        0.0
    };
    let border = mix(p.field_border, p.border_focus, focus);
    painter.rect_stroke(
        rect,
        CornerRadius::same(RADIUS_CONTROL),
        Stroke::new(1.0 + 0.6 * focus, border),
        StrokeKind::Inside,
    );
    if focus > 0.02 {
        glow(&painter, rect, RADIUS_CONTROL, p.accent, 0.18 * focus);
    }

    response
}

/// Numeric field with the little up/down arrows on the right.
/// Returns the response plus a requested +/-1 step.
pub fn spin_field(
    ui: &mut Ui,
    rect: Rect,
    key: &str,
    text: &mut String,
    enabled: bool,
    p: &Palette,
) -> (Response, i32) {
    let painter = ui.painter().clone();
    painter.rect_filled(
        rect,
        CornerRadius::same(RADIUS_CONTROL),
        if enabled { p.field } else { p.soft },
    );

    let mut delta = 0;
    let up_rect = Rect::from_min_max(
        Pos2::new(rect.right() - 30.0, rect.top()),
        Pos2::new(rect.right(), rect.center().y),
    );
    let down_rect = Rect::from_min_max(
        Pos2::new(rect.right() - 30.0, rect.center().y),
        Pos2::new(rect.right(), rect.bottom()),
    );

    let up_resp = ui.interact(
        up_rect,
        Id::unique(("spin-up", key)),
        if enabled { Sense::click() } else { Sense::hover() },
    );
    let down_resp = ui.interact(
        down_rect,
        Id::unique(("spin-down", key)),
        if enabled { Sense::click() } else { Sense::hover() },
    );

    if enabled && up_resp.clicked() {
        delta += 1;
    }
    if enabled && down_resp.clicked() {
        delta -= 1;
    }

    let response = if enabled {
        let inner = Rect::from_min_max(
            Pos2::new(rect.left() + 10.0, rect.center().y - 10.0),
            Pos2::new(rect.right() - 34.0, rect.center().y + 10.0),
        );
        let response = ui.put(
            inner,
            TextEdit::singleline(text)
                .frame(eframe::egui::Frame::NONE)
                .align(Align2::RIGHT_CENTER)
                .font(font_field())
                .text_color(p.ink)
                .margin(Margin::symmetric(0, 2)),
        );
        text.retain(|c| c.is_ascii_digit());
        if text.len() > 9 {
            text.truncate(9);
        }
        if response.has_focus() {
            if ui.input(|i| i.key_pressed(eframe::egui::Key::ArrowUp)) {
                delta += 1;
            }
            if ui.input(|i| i.key_pressed(eframe::egui::Key::ArrowDown)) {
                delta -= 1;
            }
        }
        response
    } else {
        painter.text(
            Pos2::new(rect.right() - 36.0, rect.center().y),
            Align2::RIGHT_CENTER,
            text.as_str(),
            font_field(),
            p.muted,
        );
        ui.interact(rect, Id::unique(("spin-field", key)), Sense::hover())
    };

    let focus = if enabled {
        ui.ctx()
            .animate_bool_with_time(Id::unique(("spin", key, "focus")), response.has_focus(), 0.14)
    } else {
        0.0
    };

    let cx = rect.right() - 15.0;
    let w = 5.0;
    let h = 3.2;
    let up_color = if !enabled {
        p.disabled_ink
    } else if up_resp.hovered() {
        p.accent
    } else {
        p.muted
    };
    let down_color = if !enabled {
        p.disabled_ink
    } else if down_resp.hovered() {
        p.accent
    } else {
        p.muted
    };

    let up_y = rect.center().y - 6.0;
    painter.add(Shape::convex_polygon(
        vec![
            Pos2::new(cx, up_y - h * 0.5),
            Pos2::new(cx - w, up_y + h * 0.5),
            Pos2::new(cx + w, up_y + h * 0.5),
        ],
        up_color,
        Stroke::NONE,
    ));
    let down_y = rect.center().y + 6.0;
    painter.add(Shape::convex_polygon(
        vec![
            Pos2::new(cx, down_y + h * 0.5),
            Pos2::new(cx - w, down_y - h * 0.5),
            Pos2::new(cx + w, down_y - h * 0.5),
        ],
        down_color,
        Stroke::NONE,
    ));

    let border = mix(p.field_border, p.border_focus, focus);
    painter.rect_stroke(
        rect,
        CornerRadius::same(RADIUS_CONTROL),
        Stroke::new(1.0 + 0.6 * focus, border),
        StrokeKind::Inside,
    );
    if focus > 0.02 {
        glow(&painter, rect, RADIUS_CONTROL, p.accent, 0.18 * focus);
    }

    (response, delta)
}

/// Key capture field for the hotkey dialogs.
pub fn key_field(
    ui: &mut Ui,
    rect: Rect,
    key: &str,
    text: &str,
    focused: bool,
    p: &Palette,
) -> Response {
    let id = Id::unique(("key-field", key));
    let response = ui.interact(rect, id, Sense::click());
    let ctx = ui.ctx();

    let focus = ctx.animate_bool_with_time(id.with("focus"), focused, 0.18);
    let hover = ctx.animate_bool_with_time(id.with("hover"), response.hovered(), 0.12);
    let pulse = if focused {
        0.5 + 0.5 * (ctx.input(|i| i.time) as f32 * 4.0).sin()
    } else {
        0.0
    };

    let painter = ui.painter();
    painter.rect_filled(rect, CornerRadius::same(RADIUS_CONTROL), p.field);
    painter.rect_stroke(
        rect,
        CornerRadius::same(RADIUS_CONTROL),
        Stroke::new(
            1.0 + 0.7 * focus,
            mix(p.field_border, p.border_focus, focus.max(hover * 0.5)),
        ),
        StrokeKind::Inside,
    );
    if focus > 0.02 {
        glow(painter, rect, RADIUS_CONTROL, p.accent, 0.10 + 0.18 * focus * pulse);
    }
    painter.text(
        rect.center(),
        Align2::CENTER_CENTER,
        text,
        font_semibold(14.0),
        p.ink,
    );

    response
}

// ---------------------------------------------------------------------- tiles

/// Large record/play tile with a round icon badge and a caption.
#[allow(clippy::too_many_arguments)]
pub fn tile(
    ui: &mut Ui,
    rect: Rect,
    key: &str,
    icon: IconKind,
    title: &str,
    subtitle: &str,
    tint: Color32,
    active: bool,
    enabled: bool,
    p: &Palette,
) -> Response {
    let id = Id::unique(("tile", key));
    let sense = if enabled { Sense::click() } else { Sense::hover() };
    let response = ui.interact(rect, id, sense);
    let ctx = ui.ctx();

    let hover = if enabled {
        ctx.animate_bool_with_time(id.with("hover"), response.hovered(), 0.16)
    } else {
        0.0
    };
    let down = if enabled {
        ctx.animate_bool_with_time(id.with("down"), response.is_pointer_button_down_on(), 0.08)
    } else {
        0.0
    };
    let active_t = ctx.animate_bool_with_time(id.with("active"), active, 0.25);

    let lift = -2.0 * hover + 1.0 * down;
    let body = rect.translate(Vec2::new(0.0, lift));
    let pulse = 0.5 + 0.5 * (ctx.input(|i| i.time) as f32 * 3.2).sin();
    let tint_strength = (0.10 * active_t + 0.06 * hover).clamp(0.0, 0.2);

    let painter = ui.painter();
    soft_shadow(painter, body, RADIUS_CARD, p.shadow, 12.0, 5.0);
    if active_t > 0.01 {
        glow(painter, body, RADIUS_CARD, tint, active_t * (0.20 + 0.14 * pulse));
    }
    painter.rect_filled(
        body,
        CornerRadius::same(RADIUS_CARD),
        mix(p.card, mix(p.card, tint, 0.18), active_t.max(hover * 0.6)),
    );
    painter.rect_stroke(
        body,
        CornerRadius::same(RADIUS_CARD),
        Stroke::new(1.0, mix(p.card_border, fade(tint, 0.55), active_t.max(hover * 0.4))),
        StrokeKind::Inside,
    );
    painter.line_segment(
        [
            Pos2::new(body.left() + 18.0, body.top() + 0.5),
            Pos2::new(body.right() - 18.0, body.top() + 0.5),
        ],
        Stroke::new(1.0, fade(p.card_highlight, 0.30)),
    );

    let badge_r = 30.0;
    let badge = Rect::from_center_size(
        Pos2::new(body.center().x, body.top() + 66.0),
        Vec2::splat(badge_r * 2.0),
    );
    painter.circle_filled(
        badge.center(),
        badge_r,
        mix(fade(tint, 0.13), fade(tint, 0.22), active_t.max(hover * 0.5)),
    );
    painter.circle_stroke(
        badge.center(),
        badge_r,
        Stroke::new(1.0, fade(tint, 0.30 + 0.25 * active_t)),
    );
    let icon_rect = Rect::from_center_size(
        badge.center() - Vec2::new(0.0, tint_strength * 0.0),
        Vec2::splat(24.0),
    );
    draw_icon(painter, icon_rect, icon, tint, mix(p.card, tint, 0.13));

    painter.text(
        Pos2::new(body.center().x, body.top() + 116.0),
        Align2::CENTER_CENTER,
        title,
        font_semibold(15.0),
        if enabled { p.ink } else { p.muted },
    );

    let galley = painter.layout(subtitle.to_owned(), font_small(), p.muted, body.width() - 34.0);
    painter.galley(
        Pos2::new(
            body.center().x - galley.size().x * 0.5,
            body.top() + 132.0,
        ),
        galley,
        p.muted,
    );

    response
}

// ---------------------------------------------------------------------- menus

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ItemKind {
    Action,
    Check(bool),
    Separator,
}

pub struct MenuItem {
    pub label: &'static str,
    pub kind: ItemKind,
}

impl MenuItem {
    pub fn action(label: &'static str) -> Self {
        Self {
            label,
            kind: ItemKind::Action,
        }
    }

    pub fn check(label: &'static str, checked: bool) -> Self {
        Self {
            label,
            kind: ItemKind::Check(checked),
        }
    }

    pub fn separator() -> Self {
        Self {
            label: "",
            kind: ItemKind::Separator,
        }
    }

    fn height(&self) -> f32 {
        match self.kind {
            ItemKind::Separator => 10.0,
            _ => 32.0,
        }
    }
}

/// Draws a themed popup menu below (or above) the anchor.
/// `appear` fades and slides the menu in (0..1).
/// Returns the clicked item index and the popup rectangle.
pub fn popup_menu(
    ctx: &Context,
    id: Id,
    anchor: Rect,
    width: f32,
    items: &[MenuItem],
    appear: f32,
    p: &Palette,
) -> (Option<usize>, Rect) {
    let screen = ctx.viewport_rect();
    let pad = 6.0;
    let mut height = pad * 2.0;
    for item in items {
        height += item.height();
    }

    let mut position = Pos2::new(anchor.left(), anchor.bottom() + 6.0);
    if position.y + height > screen.bottom() - 8.0 {
        position.y = (anchor.top() - 6.0 - height).max(screen.top() + 8.0);
    }
    position.x = position
        .x
        .min(screen.right() - width - 8.0)
        .max(screen.left() + 8.0);

    let slide = (1.0 - appear) * 8.0;
    let rect = Rect::from_min_size(position + Vec2::new(0.0, slide), Vec2::new(width, height));
    let mut clicked = None;

    Area::new(id)
        .order(Order::Foreground)
        .fixed_pos(position)
        .interactable(true)
        .show(ctx, |ui| {
            ui.set_opacity(appear);
            let painter = ui.painter();
            soft_shadow(painter, rect, 12, p.shadow, 16.0, 7.0);
            painter.rect_filled(rect, CornerRadius::same(12), p.popup);
            painter.rect_stroke(
                rect,
                CornerRadius::same(12),
                Stroke::new(1.0, p.card_border),
                StrokeKind::Inside,
            );
            painter.line_segment(
                [
                    Pos2::new(rect.left() + 14.0, rect.top() + 0.5),
                    Pos2::new(rect.right() - 14.0, rect.top() + 0.5),
                ],
                Stroke::new(1.0, fade(p.card_highlight, 0.28)),
            );

            let mut y = rect.top() + pad;
            for (index, item) in items.iter().enumerate() {
                let item_height = item.height();
                match item.kind {
                    ItemKind::Separator => {
                        painter.line_segment(
                            [
                                Pos2::new(rect.left() + 12.0, y + 5.0),
                                Pos2::new(rect.right() - 12.0, y + 5.0),
                            ],
                            Stroke::new(1.0, p.card_border),
                        );
                    }
                    ItemKind::Action | ItemKind::Check(_) => {
                        let row = Rect::from_min_size(
                            Pos2::new(rect.left() + pad, y),
                            Vec2::new(width - pad * 2.0, item_height),
                        );
                        let response =
                            ui.interact(row, Id::unique((id, "row", index)), Sense::click());
                        let hover = ctx.animate_bool_with_time(
                            Id::unique((id, "row-hover", index)),
                            response.hovered(),
                            0.10,
                        );
                        if hover > 0.01 {
                            painter.rect_filled(
                                row,
                                CornerRadius::same(8),
                                fade(p.soft_hover, hover.max(0.15)),
                            );
                        }

                        let (text_left, color) = match item.kind {
                            ItemKind::Check(true) => {
                                let size = 14.0;
                                let check_rect = Rect::from_center_size(
                                    Pos2::new(row.left() + 17.0, row.center().y),
                                    Vec2::splat(size),
                                );
                                painter.add(Shape::line(
                                    vec![
                                        Pos2::new(
                                            check_rect.left() + size * 0.12,
                                            check_rect.top() + size * 0.52,
                                        ),
                                        Pos2::new(
                                            check_rect.left() + size * 0.40,
                                            check_rect.top() + size * 0.78,
                                        ),
                                        Pos2::new(
                                            check_rect.left() + size * 0.90,
                                            check_rect.top() + size * 0.24,
                                        ),
                                    ],
                                    Stroke::new(1.9, p.accent),
                                ));
                                (row.left() + 33.0, p.ink)
                            }
                            ItemKind::Check(false) => (row.left() + 33.0, p.ink),
                            _ => (row.left() + 12.0, p.ink),
                        };

                        painter.text(
                            Pos2::new(text_left, row.center().y),
                            Align2::LEFT_CENTER,
                            item.label,
                            font_body(),
                            color,
                        );

                        if response.clicked() {
                            clicked = Some(index);
                        }
                    }
                }

                y += item_height;
            }
        });

    (clicked, rect)
}
