//! Hand-drawn controls that mirror the C# build: cards, buttons, radios,
//! checkboxes, rounded text fields, a spin field and the popup menus.

use eframe::egui::{
    Align2, Area, Color32, Context, CornerRadius, Id, Margin, Order, Pos2, Rect, Response, Sense,
    Shape, Stroke, StrokeKind, TextEdit, Ui, Vec2,
};

use crate::theme::{
    draw_icon, font_body, font_caption, font_field, font_semibold, IconKind, Palette, RADIUS_BUTTON,
    RADIUS_CARD, RADIUS_CONTROL,
};

pub fn card(painter: &eframe::egui::Painter, rect: Rect, title: &str, p: &Palette) {
    let body = rect.shrink(1.0);
    painter.rect_filled(body, CornerRadius::same(RADIUS_CARD), p.card);
    painter.rect_stroke(
        body,
        CornerRadius::same(RADIUS_CARD),
        Stroke::new(1.0, p.border),
        StrokeKind::Inside,
    );

    if !title.is_empty() {
        painter.text(
            rect.min + Vec2::new(16.0, 11.0),
            Align2::LEFT_TOP,
            title,
            font_semibold(12.5),
            p.ink,
        );
    }
}

pub fn caption(painter: &eframe::egui::Painter, center: Pos2, text: &str, p: &Palette) {
    painter.text(center, Align2::CENTER_CENTER, text, font_caption(), p.muted);
}

/// Left-aligned caption, as used above the "Click options" dropdowns.
pub fn caption_left(painter: &eframe::egui::Painter, left_center: Pos2, text: &str, p: &Palette) {
    painter.text(left_center, Align2::LEFT_CENTER, text, font_caption(), p.muted);
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BtnStyle {
    Primary,
    Secondary,
    Danger,
    Ghost,
}

fn button_colors(style: BtnStyle, enabled: bool, hover: bool, down: bool, p: &Palette) -> (Color32, Color32, Option<Color32>) {
    if !enabled {
        return match style {
            BtnStyle::Primary => (p.disabled_surface, p.on_accent, None),
            BtnStyle::Ghost => (p.soft, p.disabled_ink, None),
            _ => (p.card, p.disabled_ink, Some(p.border)),
        };
    }

    match style {
        BtnStyle::Primary => (
            if down {
                p.accent_down
            } else if hover {
                p.accent_hover
            } else {
                p.accent
            },
            p.on_accent,
            None,
        ),
        BtnStyle::Danger => (
            if down {
                p.danger_down
            } else if hover {
                p.danger_hover
            } else {
                p.danger_soft
            },
            p.danger,
            None,
        ),
        BtnStyle::Ghost => (
            if hover || down { p.soft_hover } else { p.soft },
            p.ink,
            None,
        ),
        BtnStyle::Secondary => (
            if down {
                p.soft
            } else if hover {
                p.hover_card
            } else {
                p.card
            },
            p.ink,
            Some(p.border),
        ),
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

    let hover = enabled && response.hovered();
    let down = enabled && response.is_pointer_button_down_on();
    let (back, fore, border) = button_colors(style, enabled, hover, down, p);

    let painter = ui.painter();
    painter.rect_filled(rect, CornerRadius::same(RADIUS_BUTTON), back);
    if let Some(border) = border {
        painter.rect_stroke(
            rect,
            CornerRadius::same(RADIUS_BUTTON),
            Stroke::new(1.0, border),
            StrokeKind::Inside,
        );
    }

    let text_font = font_semibold(12.5);
    let galley = painter.layout_no_wrap(text.to_owned(), text_font, fore);

    match icon {
        Some(kind) => {
            let icon_color = icon_color.unwrap_or(fore);
            if text.is_empty() {
                let icon_rect = Rect::from_center_size(rect.center(), Vec2::splat(icon_size));
                draw_icon(painter, icon_rect, kind, icon_color, back);
            } else {
                let gap = 10.0;
                let total = icon_size + gap + galley.size().x;
                let start = rect.center().x - total * 0.5;
                let icon_rect = Rect::from_center_size(
                    Pos2::new(start + icon_size * 0.5, rect.center().y),
                    Vec2::splat(icon_size),
                );
                draw_icon(painter, icon_rect, kind, icon_color, back);
                painter.galley(
                    Pos2::new(start + icon_size + gap, rect.center().y - galley.size().y * 0.5),
                    galley,
                    fore,
                );
            }
        }
        None => {
            painter.galley(
                rect.center() - galley.size() * 0.5,
                galley,
                fore,
            );
        }
    }

    response
}

/// Flat text link used by the Advanced header (Options / Help).
pub fn text_button(ui: &mut Ui, rect: Rect, key: &str, text: &str, enabled: bool, p: &Palette) -> Response {
    let id = Id::unique(("text-button", key));
    let sense = if enabled { Sense::click() } else { Sense::hover() };
    let response = ui.interact(rect, id, sense);
    let hover = enabled && response.hovered();
    let color = if !enabled {
        p.disabled_ink
    } else if hover {
        p.ink
    } else {
        p.muted
    };
    let painter = ui.painter();
    let galley = painter.layout_no_wrap(text.to_owned(), font_body(), color);
    painter.galley(rect.center() - galley.size() * 0.5, galley, color);
    response
}

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

    let painter = ui.painter();
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
        Stroke::new(1.6, if selected { p.accent } else { p.control_border }),
    );
    if selected {
        painter.circle_filled(circle.center(), 4.0, p.accent);
    }

    if !text.is_empty() {
        let color = if enabled { p.ink } else { p.muted };
        painter.text(
            Pos2::new(circle.right() + 8.0, rect.center().y),
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

    let painter = ui.painter();
    let size = 18.0;
    let box_rect = Rect::from_center_size(
        Pos2::new(rect.left() + 1.0 + size * 0.5, rect.center().y),
        Vec2::splat(size),
    );

    painter.rect_filled(
        box_rect,
        CornerRadius::same(5),
        if checked {
            p.accent
        } else if enabled {
            p.field
        } else {
            p.soft
        },
    );
    painter.rect_stroke(
        box_rect,
        CornerRadius::same(5),
        Stroke::new(1.6, if checked { p.accent } else { p.control_border }),
        StrokeKind::Inside,
    );

    if checked {
        let points = vec![
            Pos2::new(box_rect.left() + size * 0.24, box_rect.top() + size * 0.52),
            Pos2::new(box_rect.left() + size * 0.44, box_rect.top() + size * 0.72),
            Pos2::new(box_rect.left() + size * 0.78, box_rect.top() + size * 0.30),
        ];
        painter.add(Shape::line(points, Stroke::new(2.2, p.on_accent)));
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
    let hover = enabled && response.hovered();

    let painter = ui.painter();
    painter.rect_filled(
        rect,
        CornerRadius::same(RADIUS_CONTROL),
        if enabled { p.field } else { p.soft },
    );

    let border = if open || (enabled && hover) {
        Stroke::new(if open { 1.6 } else { 1.0 }, p.border_focus)
    } else {
        Stroke::new(1.0, p.border)
    };
    painter.rect_stroke(rect, CornerRadius::same(RADIUS_CONTROL), border, StrokeKind::Inside);

    painter.text(
        Pos2::new(rect.left() + 12.0, rect.center().y),
        Align2::LEFT_CENTER,
        text,
        font_body(),
        if enabled { p.ink } else { p.muted },
    );

    let cx = rect.right() - 16.0;
    let cy = rect.center().y;
    let color = if open || hover { p.accent } else { p.muted };
    let stroke = Stroke::new(1.6, color);
    painter.line_segment(
        [Pos2::new(cx - 4.0, cy - 2.5), Pos2::new(cx, cy + 2.5)],
        stroke,
    );
    painter.line_segment(
        [Pos2::new(cx, cy + 2.5), Pos2::new(cx + 4.0, cy - 2.5)],
        stroke,
    );

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

    let border = if response.has_focus() {
        Stroke::new(1.6, p.border_focus)
    } else {
        Stroke::new(1.0, p.border)
    };
    painter.rect_stroke(rect, CornerRadius::same(RADIUS_CONTROL), border, StrokeKind::Inside);

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
            Pos2::new(rect.left() + 8.0, rect.center().y - 10.0),
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

    let cx = rect.right() - 15.0;
    let w = 5.0;
    let h = 3.0;
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

    let border = if response.has_focus() {
        Stroke::new(1.6, p.border_focus)
    } else {
        Stroke::new(1.0, p.border)
    };
    painter.rect_stroke(rect, CornerRadius::same(RADIUS_CONTROL), border, StrokeKind::Inside);

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

    let painter = ui.painter();
    painter.rect_filled(rect, CornerRadius::same(RADIUS_CONTROL), p.field);
    painter.rect_stroke(
        rect,
        CornerRadius::same(RADIUS_CONTROL),
        Stroke::new(if focused { 1.6 } else { 1.0 }, if focused { p.border_focus } else { p.border }),
        StrokeKind::Inside,
    );
    painter.text(
        rect.center(),
        Align2::CENTER_CENTER,
        text,
        font_semibold(14.0),
        p.ink,
    );

    response
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ItemKind {
    Action,
    Check(bool),
    Separator,
    Caption,
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

    pub fn caption(label: &'static str) -> Self {
        Self {
            label,
            kind: ItemKind::Caption,
        }
    }

    fn height(&self) -> f32 {
        match self.kind {
            ItemKind::Separator => 9.0,
            ItemKind::Caption => 22.0,
            _ => 28.0,
        }
    }
}

/// Draws a themed popup menu below (or above) the anchor.
/// Returns the clicked item index and the popup rectangle.
pub fn popup_menu(
    ctx: &Context,
    id: Id,
    anchor: Rect,
    width: f32,
    items: &[MenuItem],
    p: &Palette,
) -> (Option<usize>, Rect) {
    let screen = ctx.viewport_rect();
    let pad = 5.0;
    let mut height = pad * 2.0;
    for item in items {
        height += item.height();
    }

    let mut position = Pos2::new(anchor.left(), anchor.bottom() + 4.0);
    if position.y + height > screen.bottom() - 6.0 {
        position.y = (anchor.top() - 4.0 - height).max(screen.top() + 6.0);
    }
    position.x = position
        .x
        .min(screen.right() - width - 6.0)
        .max(screen.left() + 6.0);

    let rect = Rect::from_min_size(position, Vec2::new(width, height));
    let mut clicked = None;

    Area::new(id)
        .order(Order::Foreground)
        .fixed_pos(position)
        .show(ctx, |ui| {
            let painter = ui.painter();
            painter.rect_filled(
                rect.translate(Vec2::new(0.0, 4.0)).expand(1.0),
                CornerRadius::same(11),
                Color32::from_black_alpha(30),
            );
            painter.rect_filled(rect, CornerRadius::same(10), p.card);
            painter.rect_stroke(
                rect,
                CornerRadius::same(10),
                Stroke::new(1.0, p.border),
                StrokeKind::Inside,
            );

            let mut y = rect.top() + pad;
            for (index, item) in items.iter().enumerate() {
                let item_height = item.height();
                match item.kind {
                    ItemKind::Separator => {
                        painter.line_segment(
                            [
                                Pos2::new(rect.left() + 10.0, y + 4.5),
                                Pos2::new(rect.right() - 10.0, y + 4.5),
                            ],
                            Stroke::new(1.0, p.border),
                        );
                    }
                    ItemKind::Caption => {
                        painter.text(
                            Pos2::new(rect.left() + 16.0, y + item_height * 0.5),
                            Align2::LEFT_CENTER,
                            item.label,
                            font_caption(),
                            p.muted,
                        );
                    }
                    ItemKind::Action | ItemKind::Check(_) => {
                        let row = Rect::from_min_size(
                            Pos2::new(rect.left() + pad, y),
                            Vec2::new(width - pad * 2.0, item_height),
                        );
                        let response = ui.interact(
                            row,
                            Id::unique((id, "row", index)),
                            Sense::click(),
                        );
                        if response.hovered() {
                            painter.rect_filled(
                                row,
                                CornerRadius::same(7),
                                p.soft_hover,
                            );
                        }

                        let (text_left, color) = match item.kind {
                            ItemKind::Check(true) => {
                                let size = 14.0;
                                let check_rect = Rect::from_center_size(
                                    Pos2::new(row.left() + 16.0, row.center().y),
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
                                    Stroke::new(1.8, p.accent),
                                ));
                                (row.left() + 32.0, p.ink)
                            }
                            ItemKind::Check(false) => (row.left() + 32.0, p.ink),
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
