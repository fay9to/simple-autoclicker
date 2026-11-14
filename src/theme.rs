//! Palette, typography, drawing helpers and the small vector icons.

use eframe::egui::{
    Color32, Context, CornerRadius, FontFamily, FontId, Mesh, Painter, Pos2, Rect, Shape, Stroke,
    TextStyle, Vec2, Visuals,
};
use windows::core::w;
use windows::Win32::Foundation::ERROR_SUCCESS;
use windows::Win32::System::Registry::{RegGetValueW, HKEY_CURRENT_USER, RRF_RT_REG_DWORD};

use crate::settings::AppTheme;

#[derive(Clone, Copy, Debug)]
pub struct Palette {
    pub bg_top: Color32,
    pub bg_bottom: Color32,
    pub card: Color32,
    pub card_border: Color32,
    pub card_highlight: Color32,
    /// Opaque surface for menus and dialogs, so text stays readable.
    pub popup: Color32,
    pub field: Color32,
    pub field_border: Color32,
    pub ink: Color32,
    pub muted: Color32,
    pub disabled_ink: Color32,
    pub accent: Color32,
    pub accent2: Color32,
    pub on_accent: Color32,
    pub soft: Color32,
    pub soft_hover: Color32,
    pub danger: Color32,
    pub danger_soft: Color32,
    pub danger_hover: Color32,
    pub green: Color32,
    pub shadow: Color32,
    pub disabled_surface: Color32,
    pub border_focus: Color32,
}

fn rgba(r: u8, g: u8, b: u8, a: u8) -> Color32 {
    Color32::from_rgba_unmultiplied(r, g, b, a)
}

fn rgb(r: u8, g: u8, b: u8) -> Color32 {
    Color32::from_rgb(r, g, b)
}

impl Palette {
    pub fn light() -> Self {
        Self {
            bg_top: rgba(255, 255, 255, 250),
            bg_bottom: rgba(236, 240, 249, 250),
            card: rgba(255, 255, 255, 232),
            card_border: rgba(24, 32, 56, 18),
            card_highlight: rgba(255, 255, 255, 220),
            popup: rgba(255, 255, 255, 251),
            field: rgba(246, 248, 253, 240),
            field_border: rgba(24, 32, 56, 30),
            ink: rgb(24, 28, 38),
            muted: rgb(108, 116, 134),
            disabled_ink: rgb(172, 178, 192),
            accent: rgb(91, 108, 255),
            accent2: rgb(150, 92, 255),
            on_accent: rgb(255, 255, 255),
            soft: rgba(24, 32, 56, 9),
            soft_hover: rgba(24, 32, 56, 16),
            danger: rgb(224, 68, 78),
            danger_soft: rgba(224, 68, 78, 26),
            danger_hover: rgba(224, 68, 78, 44),
            green: rgb(30, 158, 92),
            shadow: Color32::from_black_alpha(38),
            disabled_surface: rgba(24, 32, 56, 22),
            border_focus: rgb(91, 108, 255),
        }
    }

    pub fn dark() -> Self {
        Self {
            bg_top: rgba(19, 22, 30, 244),
            bg_bottom: rgba(11, 13, 19, 244),
            card: rgba(255, 255, 255, 11),
            card_border: rgba(255, 255, 255, 20),
            card_highlight: rgba(255, 255, 255, 26),
            popup: rgba(32, 36, 46, 251),
            field: rgba(255, 255, 255, 8),
            field_border: rgba(255, 255, 255, 22),
            ink: rgb(235, 238, 246),
            muted: rgb(150, 159, 178),
            disabled_ink: rgb(105, 113, 130),
            accent: rgb(111, 124, 255),
            accent2: rgb(166, 107, 255),
            on_accent: rgb(255, 255, 255),
            soft: rgba(255, 255, 255, 8),
            soft_hover: rgba(255, 255, 255, 15),
            danger: rgb(255, 108, 118),
            danger_soft: rgba(255, 108, 118, 30),
            danger_hover: rgba(255, 108, 118, 52),
            green: rgb(74, 222, 128),
            shadow: Color32::from_black_alpha(70),
            disabled_surface: rgba(255, 255, 255, 14),
            border_focus: rgb(132, 143, 255),
        }
    }
}

// ------------------------------------------------------------------- helpers

pub fn mix(a: Color32, b: Color32, t: f32) -> Color32 {
    a.lerp_to_gamma(b, t.clamp(0.0, 1.0))
}

pub fn fade(color: Color32, t: f32) -> Color32 {
    let a = (color.a() as f32 * t.clamp(0.0, 1.0)).round() as u8;
    Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), a)
}

pub fn ease_out(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    1.0 - (1.0 - t) * (1.0 - t) * (1.0 - t)
}

pub const RADIUS_CARD: u8 = 16;
pub const RADIUS_CONTROL: u8 = 10;
pub const RADIUS_BUTTON: u8 = 12;

/// Vertical gradient with rounded top and bottom caps. egui meshes are not
/// clipped by a corner radius, so the caps are drawn as separate rounded rects.
pub fn vertical_gradient(painter: &Painter, rect: Rect, radius: u8, top: Color32, bottom: Color32) {
    let r = radius as f32;
    if rect.height() <= r * 2.0 + 1.0 {
        painter.rect_filled(rect, CornerRadius::same(radius), mix(top, bottom, 0.5));
        return;
    }

    painter.rect_filled(rect, CornerRadius::same(radius), top);

    let middle = Rect::from_min_max(
        Pos2::new(rect.left(), rect.top() + r),
        Pos2::new(rect.right(), rect.bottom() - r),
    );
    let mut mesh = Mesh::default();
    let start = mesh.vertices.len() as u32;
    mesh.colored_vertex(middle.left_top(), top);
    mesh.colored_vertex(middle.right_top(), top);
    mesh.colored_vertex(middle.left_bottom(), bottom);
    mesh.colored_vertex(middle.right_bottom(), bottom);
    mesh.add_triangle(start, start + 1, start + 2);
    mesh.add_triangle(start + 1, start + 3, start + 2);
    painter.add(Shape::mesh(mesh));

    painter.rect_filled(
        Rect::from_min_max(rect.min, Pos2::new(rect.right(), rect.top() + r)),
        CornerRadius {
            nw: radius,
            ne: radius,
            sw: 0,
            se: 0,
        },
        top,
    );
    painter.rect_filled(
        Rect::from_min_max(Pos2::new(rect.left(), rect.bottom() - r), rect.max),
        CornerRadius {
            nw: 0,
            ne: 0,
            sw: radius,
            se: radius,
        },
        bottom,
    );
}

/// Soft shadow made of a few translucent rounded rects, from wide and faint to
/// tight and stronger.
pub fn soft_shadow(
    painter: &Painter,
    rect: Rect,
    radius: u8,
    color: Color32,
    spread: f32,
    offset_y: f32,
) {
    let layers = 7;
    for i in (1..=layers).rev() {
        let t = i as f32 / layers as f32;
        let grow = spread * t;
        let alpha = color.a() as f32 * (1.0 - t) * (1.0 - t);
        let shifted = rect.translate(Vec2::new(0.0, offset_y * (1.0 - t * 0.6)));
        painter.rect_filled(
            shifted.expand(grow),
            CornerRadius::same((radius as f32 + grow).round() as u8),
            Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), alpha as u8),
        );
    }
}

/// Accent glow behind an element; `strength` is 0..1.
pub fn glow(painter: &Painter, rect: Rect, radius: u8, color: Color32, strength: f32) {
    if strength <= 0.001 {
        return;
    }
    soft_shadow(
        painter,
        rect,
        radius,
        fade(color, 0.55 * strength.clamp(0.0, 1.0)),
        12.0,
        2.0,
    );
}

/// Soft radial blob: a mesh fan with interpolated alpha, so there are no
/// visible rings or banding.
pub fn soft_circle(painter: &Painter, center: Pos2, radius: f32, color: Color32) {
    const SEGMENTS: usize = 72;
    let mid = Color32::from_rgba_unmultiplied(
        color.r(),
        color.g(),
        color.b(),
        (color.a() as f32 * 0.55) as u8,
    );
    let edge = Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 0);

    let mut mesh = Mesh::default();
    mesh.colored_vertex(center, color);

    let mid_start = mesh.vertices.len() as u32;
    for i in 0..=SEGMENTS {
        let angle = i as f32 * std::f32::consts::TAU / SEGMENTS as f32;
        mesh.colored_vertex(center + Vec2::new(angle.cos(), angle.sin()) * radius * 0.45, mid);
    }

    let edge_start = mesh.vertices.len() as u32;
    for i in 0..=SEGMENTS {
        let angle = i as f32 * std::f32::consts::TAU / SEGMENTS as f32;
        mesh.colored_vertex(center + Vec2::new(angle.cos(), angle.sin()) * radius, edge);
    }

    for i in 0..SEGMENTS as u32 {
        mesh.add_triangle(0, mid_start + i, mid_start + i + 1);
        mesh.add_triangle(mid_start + i, edge_start + i, mid_start + i + 1);
        mesh.add_triangle(mid_start + i + 1, edge_start + i, edge_start + i + 1);
    }

    painter.add(Shape::mesh(mesh));
}

/// Drops the alpha channel, for shapes that are used as an eraser.
pub fn opaque(color: Color32) -> Color32 {
    Color32::from_rgb(color.r(), color.g(), color.b())
}

// --------------------------------------------------------------------- fonts

/// Reads the Windows "dark mode for apps" preference.
pub fn system_prefers_dark() -> bool {
    let mut value: u32 = 0;
    let mut size = std::mem::size_of::<u32>() as u32;
    let status = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            w!("Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize"),
            w!("AppsUseLightTheme"),
            RRF_RT_REG_DWORD,
            None,
            Some(&mut value as *mut u32 as *mut core::ffi::c_void),
            Some(&mut size),
        )
    };
    status == ERROR_SUCCESS && value == 0
}

pub fn resolve(mode: AppTheme) -> bool {
    match mode {
        AppTheme::Dark => true,
        AppTheme::Light => false,
        AppTheme::System => system_prefers_dark(),
    }
}

pub fn font_caption() -> FontId {
    FontId::new(10.8, FontFamily::Proportional)
}

pub fn font_small() -> FontId {
    FontId::new(11.8, FontFamily::Proportional)
}

pub fn font_body() -> FontId {
    FontId::new(12.6, FontFamily::Proportional)
}

pub fn font_field() -> FontId {
    FontId::new(13.6, FontFamily::Proportional)
}

pub fn font_semibold(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name("semibold".into()))
}

/// Loads Segoe UI (and its semibold weight) so the app looks native and has
/// Cyrillic plus Western European glyphs.
pub fn install_fonts(ctx: &Context) {
    use eframe::egui::FontDefinitions;
    use std::sync::Arc;

    let mut fonts = FontDefinitions::default();
    let Ok(regular) = std::fs::read(r"C:\Windows\Fonts\segoeui.ttf") else {
        return;
    };

    fonts.font_data.insert(
        "segoe".to_owned(),
        Arc::new(eframe::egui::FontData::from_owned(regular)),
    );
    fonts
        .families
        .entry(FontFamily::Proportional)
        .or_default()
        .insert(0, "segoe".to_owned());

    let semibold = std::fs::read(r"C:\Windows\Fonts\seguisb.ttf")
        .or_else(|_| std::fs::read(r"C:\Windows\Fonts\segoeuib.ttf"));
    let semibold_key = match semibold {
        Ok(data) => {
            fonts.font_data.insert(
                "segoe-semibold".to_owned(),
                Arc::new(eframe::egui::FontData::from_owned(data)),
            );
            "segoe-semibold"
        }
        Err(_) => "segoe",
    };

    fonts.families.insert(
        FontFamily::Name("semibold".into()),
        vec![semibold_key.to_owned(), "segoe".to_owned()],
    );

    ctx.set_fonts(fonts);
}

/// Installs the egui style used by the whole app.
pub fn apply_style(ctx: &Context, palette: &Palette, dark: bool) {
    let mut style = (*ctx.global_style()).clone();

    let mut visuals = if dark {
        Visuals::dark()
    } else {
        Visuals::light()
    };
    visuals.override_text_color = Some(palette.ink);
    visuals.panel_fill = Color32::TRANSPARENT;
    visuals.window_fill = palette.popup;
    visuals.window_stroke = Stroke::new(1.0, palette.card_border);
    visuals.window_corner_radius = CornerRadius::same(14);
    visuals.menu_corner_radius = CornerRadius::same(12);
    visuals.extreme_bg_color = palette.field;
    visuals.faint_bg_color = palette.soft;
    visuals.selection.bg_fill = fade(palette.accent, 0.35);
    visuals.selection.stroke = Stroke::new(1.0, palette.ink);
    visuals.widgets.noninteractive.bg_fill = palette.card;
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, palette.ink);
    visuals.widgets.inactive.bg_fill = palette.field;
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, palette.ink);
    visuals.widgets.hovered.bg_fill = palette.soft_hover;
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, palette.ink);
    visuals.widgets.active.bg_fill = palette.soft_hover;
    visuals.widgets.active.fg_stroke = Stroke::new(1.0, palette.ink);
    visuals.widgets.inactive.corner_radius = CornerRadius::same(8);
    visuals.widgets.hovered.corner_radius = CornerRadius::same(8);
    visuals.widgets.active.corner_radius = CornerRadius::same(8);
    visuals.text_cursor.stroke = Stroke::new(1.6, palette.ink);
    style.visuals = visuals;

    style.spacing.item_spacing = Vec2::new(8.0, 8.0);
    style.spacing.button_padding = Vec2::new(8.0, 4.0);
    style.spacing.menu_margin = eframe::egui::Margin::same(6);

    style.text_styles = [
        (TextStyle::Small, FontId::new(11.0, FontFamily::Proportional)),
        (TextStyle::Body, font_body()),
        (TextStyle::Button, font_semibold(12.6)),
        (TextStyle::Heading, font_semibold(18.0)),
        (TextStyle::Monospace, FontId::new(12.0, FontFamily::Monospace)),
    ]
    .into();

    let style = std::sync::Arc::new(style);
    ctx.set_theme(if dark {
        eframe::egui::Theme::Dark
    } else {
        eframe::egui::Theme::Light
    });
    ctx.set_style_of(eframe::egui::Theme::Dark, style.clone());
    ctx.set_style_of(eframe::egui::Theme::Light, style);
}

// --------------------------------------------------------------------- icons

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum IconKind {
    Play,
    Record,
    Stop,
    More,
    Pick,
    Sun,
    Moon,
    Globe,
    Minus,
    Close,
    ChevronLeft,
    Cursor,
}

/// Draws one of the vector icons. `erase` is the colour behind the icon; it is
/// only used by the crescent moon, which is a circle with a bite taken out.
pub fn draw_icon(painter: &Painter, rect: Rect, kind: IconKind, color: Color32, erase: Color32) {
    let w = rect.width();
    let h = rect.height();
    let (l, t) = (rect.left(), rect.top());

    match kind {
        IconKind::Play => {
            painter.add(Shape::convex_polygon(
                vec![
                    Pos2::new(l + w * 0.18, t + h * 0.08),
                    Pos2::new(l + w * 0.92, t + h * 0.50),
                    Pos2::new(l + w * 0.18, t + h * 0.92),
                ],
                color,
                Stroke::NONE,
            ));
        }
        IconKind::Record => {
            painter.circle_filled(rect.center(), w * 0.46, color);
        }
        IconKind::Stop => {
            let s = w * 0.60;
            painter.rect_filled(
                Rect::from_center_size(rect.center(), Vec2::splat(s)),
                CornerRadius::same((s * 0.22).round() as u8),
                color,
            );
        }
        IconKind::More => {
            let d = w * 0.21;
            let gap = d * 0.6;
            let total = d * 3.0 + gap * 2.0;
            let mut x = rect.center().x - total * 0.5;
            let y = rect.center().y;
            for _ in 0..3 {
                painter.circle_filled(Pos2::new(x + d * 0.5, y), d * 0.5, color);
                x += d + gap;
            }
        }
        IconKind::Pick => {
            let stroke = Stroke::new((w * 0.15).max(1.5), color);
            let c = rect.center();
            let r = w * 0.28;
            painter.circle_stroke(c, r, stroke);
            painter.line_segment([Pos2::new(c.x, t), Pos2::new(c.x, t + h * 0.16)], stroke);
            painter.line_segment(
                [Pos2::new(c.x, rect.bottom() - h * 0.16), Pos2::new(c.x, rect.bottom())],
                stroke,
            );
            painter.line_segment([Pos2::new(l, c.y), Pos2::new(l + w * 0.16, c.y)], stroke);
            painter.line_segment(
                [Pos2::new(rect.right() - w * 0.16, c.y), Pos2::new(rect.right(), c.y)],
                stroke,
            );
            painter.circle_filled(c, (r * 0.22).max(1.0), color);
        }
        IconKind::Sun => {
            let size = w.min(h);
            let c = rect.center();
            let core = size * 0.22;
            painter.circle_filled(c, core, color);
            let stroke = Stroke::new((size * 0.09).max(1.4), color);
            for i in 0..8 {
                let angle = i as f32 * std::f32::consts::PI / 4.0;
                let r0 = core * 1.9;
                let r1 = size * 0.46;
                let dir = Vec2::new(angle.cos(), angle.sin());
                painter.line_segment([c + dir * r0, c + dir * r1], stroke);
            }
        }
        IconKind::Moon => {
            let size = w.min(h);
            painter.circle_filled(rect.center(), size * 0.48, color);
            painter.circle_filled(
                rect.center() + Vec2::new(size * 0.32, -size * 0.10),
                size * 0.41,
                erase,
            );
        }
        IconKind::Globe => {
            let c = rect.center();
            let r = w * 0.44;
            let stroke = Stroke::new((w * 0.10).max(1.2), color);
            painter.circle_stroke(c, r, stroke);
            painter.line_segment(
                [Pos2::new(c.x - r, c.y), Pos2::new(c.x + r, c.y)],
                stroke,
            );
            let mut left = Vec::new();
            let mut right = Vec::new();
            for i in 0..=24 {
                let a = -std::f32::consts::FRAC_PI_2
                    + i as f32 * std::f32::consts::PI / 24.0;
                let dx = r * 0.52 * a.sin();
                let dy = r * a.cos();
                left.push(Pos2::new(c.x - dx.abs(), c.y + dy));
                right.push(Pos2::new(c.x + dx.abs(), c.y + dy));
            }
            painter.add(Shape::line(left, stroke));
            painter.add(Shape::line(right, stroke));
        }
        IconKind::Minus => {
            let stroke = Stroke::new((w * 0.11).max(1.5), color);
            painter.line_segment(
                [
                    Pos2::new(l + w * 0.18, rect.center().y),
                    Pos2::new(rect.right() - w * 0.18, rect.center().y),
                ],
                stroke,
            );
        }
        IconKind::Close => {
            let stroke = Stroke::new((w * 0.11).max(1.5), color);
            let m = w * 0.24;
            painter.line_segment(
                [
                    Pos2::new(l + m, t + m),
                    Pos2::new(rect.right() - m, rect.bottom() - m),
                ],
                stroke,
            );
            painter.line_segment(
                [
                    Pos2::new(rect.right() - m, t + m),
                    Pos2::new(l + m, rect.bottom() - m),
                ],
                stroke,
            );
        }
        IconKind::ChevronLeft => {
            let stroke = Stroke::new((w * 0.12).max(1.6), color);
            let c = rect.center();
            painter.line_segment(
                [Pos2::new(c.x + w * 0.14, c.y - h * 0.24), Pos2::new(c.x - w * 0.14, c.y)],
                stroke,
            );
            painter.line_segment(
                [Pos2::new(c.x - w * 0.14, c.y), Pos2::new(c.x + w * 0.14, c.y + h * 0.24)],
                stroke,
            );
        }
        IconKind::Cursor => {
            let points = vec![
                Pos2::new(l + w * 0.28, t + h * 0.10),
                Pos2::new(l + w * 0.28, t + h * 0.86),
                Pos2::new(l + w * 0.44, t + h * 0.69),
                Pos2::new(l + w * 0.56, t + h * 0.95),
                Pos2::new(l + w * 0.69, t + h * 0.89),
                Pos2::new(l + w * 0.57, t + h * 0.64),
                Pos2::new(l + w * 0.77, t + h * 0.61),
            ];
            painter.add(eframe::egui::epaint::PathShape {
                points,
                closed: true,
                fill: color,
                stroke: Default::default(),
            });
        }
    }
}
