//! Palette and small drawing helpers. Colours are copied from the C# build so
//! both versions look the same; the palette is resolved every time the theme
//! changes, which makes live switching possible.

use eframe::egui::{
    Color32, Context, CornerRadius, FontFamily, FontId, Painter, Pos2, Rect, Shape, Stroke,
    TextStyle, Vec2, Visuals,
};
use windows::core::w;
use windows::Win32::Foundation::ERROR_SUCCESS;
use windows::Win32::System::Registry::{
    RegGetValueW, HKEY_CURRENT_USER, RRF_RT_REG_DWORD,
};

use crate::settings::AppTheme;

#[derive(Clone, Copy, Debug)]
pub struct Palette {
    pub bg: Color32,
    pub card: Color32,
    pub field: Color32,
    pub hover_card: Color32,
    pub ink: Color32,
    pub muted: Color32,
    pub disabled_ink: Color32,
    pub border: Color32,
    pub control_border: Color32,
    pub border_focus: Color32,
    pub accent: Color32,
    pub accent_hover: Color32,
    pub accent_down: Color32,
    pub soft: Color32,
    pub soft_hover: Color32,
    pub disabled_surface: Color32,
    pub danger: Color32,
    pub danger_soft: Color32,
    pub danger_hover: Color32,
    pub danger_down: Color32,
    pub green: Color32,
    pub on_accent: Color32,
}

fn rgb(r: u8, g: u8, b: u8) -> Color32 {
    Color32::from_rgb(r, g, b)
}

impl Palette {
    pub fn light() -> Self {
        Self {
            bg: rgb(245, 247, 251),
            card: rgb(255, 255, 255),
            field: rgb(255, 255, 255),
            hover_card: rgb(250, 251, 254),
            ink: rgb(30, 35, 48),
            muted: rgb(110, 118, 136),
            disabled_ink: rgb(170, 176, 190),
            border: rgb(225, 230, 240),
            control_border: rgb(180, 187, 200),
            border_focus: rgb(79, 107, 255),
            accent: rgb(79, 107, 255),
            accent_hover: rgb(101, 126, 255),
            accent_down: rgb(61, 86, 226),
            soft: rgb(242, 244, 250),
            soft_hover: rgb(233, 237, 246),
            disabled_surface: rgb(196, 206, 244),
            danger: rgb(226, 68, 75),
            danger_soft: rgb(253, 236, 237),
            danger_hover: rgb(253, 240, 241),
            danger_down: rgb(248, 219, 221),
            green: rgb(79, 168, 96),
            on_accent: rgb(255, 255, 255),
        }
    }

    pub fn dark() -> Self {
        Self {
            bg: rgb(22, 25, 32),
            card: rgb(30, 34, 43),
            field: rgb(24, 28, 36),
            hover_card: rgb(38, 43, 54),
            ink: rgb(231, 235, 243),
            muted: rgb(150, 159, 176),
            disabled_ink: rgb(108, 116, 132),
            border: rgb(48, 54, 67),
            control_border: rgb(74, 82, 99),
            border_focus: rgb(124, 140, 255),
            accent: rgb(108, 124, 255),
            accent_hover: rgb(127, 142, 255),
            accent_down: rgb(90, 105, 230),
            soft: rgb(36, 41, 51),
            soft_hover: rgb(46, 52, 64),
            disabled_surface: rgb(56, 63, 104),
            danger: rgb(255, 107, 112),
            danger_soft: rgb(58, 32, 38),
            danger_hover: rgb(72, 39, 46),
            danger_down: rgb(84, 44, 52),
            green: rgb(78, 208, 125),
            on_accent: rgb(255, 255, 255),
        }
    }
}

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
    FontId::new(11.0, FontFamily::Proportional)
}

/// 9 pt text: hints and status lines.
pub fn font_small() -> FontId {
    FontId::new(12.0, FontFamily::Proportional)
}

pub fn font_body() -> FontId {
    FontId::new(12.5, FontFamily::Proportional)
}

pub fn font_field() -> FontId {
    FontId::new(13.0, FontFamily::Proportional)
}

pub fn font_semibold(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name("semibold".into()))
}

pub const RADIUS_CARD: u8 = 12;
pub const RADIUS_CONTROL: u8 = 9;
pub const RADIUS_BUTTON: u8 = 10;

/// Loads Segoe UI (and its semibold weight) so the app looks native.
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

/// Installs the fonts and the egui style used by the whole app.
pub fn apply_style(ctx: &Context, palette: &Palette, dark: bool) {
    let mut style = (*ctx.global_style()).clone();

    let mut visuals = if dark {
        Visuals::dark()
    } else {
        Visuals::light()
    };
    visuals.override_text_color = Some(palette.ink);
    visuals.panel_fill = palette.bg;
    visuals.window_fill = palette.card;
    visuals.extreme_bg_color = palette.field;
    visuals.faint_bg_color = palette.soft;
    visuals.selection.bg_fill = Color32::from_rgba_unmultiplied(
        palette.accent.r(),
        palette.accent.g(),
        palette.accent.b(),
        90,
    );
    visuals.selection.stroke = Stroke::new(1.0, palette.on_accent);
    visuals.widgets.noninteractive.bg_fill = palette.card;
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, palette.ink);
    visuals.widgets.inactive.bg_fill = palette.field;
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, palette.ink);
    visuals.widgets.hovered.bg_fill = palette.soft_hover;
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, palette.ink);
    visuals.widgets.active.bg_fill = palette.soft_hover;
    visuals.widgets.active.fg_stroke = Stroke::new(1.0, palette.ink);
    visuals.text_cursor.stroke = Stroke::new(1.5, palette.ink);
    style.visuals = visuals;

    style.spacing.item_spacing = Vec2::new(8.0, 8.0);
    style.spacing.button_padding = Vec2::new(8.0, 4.0);

    style.text_styles = [
        (TextStyle::Small, FontId::new(11.0, FontFamily::Proportional)),
        (TextStyle::Body, font_body()),
        (TextStyle::Button, font_semibold(12.5)),
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

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum IconKind {
    Play,
    Record,
    Stop,
    More,
    Pick,
    Sun,
    Moon,
}

/// Draws one of the small vector icons. `erase` is the colour behind the icon;
/// it is only used by the crescent moon, which is a circle with a bite taken
/// out of it.
pub fn draw_icon(painter: &Painter, rect: Rect, kind: IconKind, color: Color32, erase: Color32) {
    let w = rect.width();
    let h = rect.height();
    let (l, t) = (rect.left(), rect.top());

    match kind {
        IconKind::Play => {
            painter.add(Shape::convex_polygon(
                vec![
                    Pos2::new(l + w * 0.16, t + h * 0.06),
                    Pos2::new(l + w * 0.94, t + h * 0.50),
                    Pos2::new(l + w * 0.16, t + h * 0.94),
                ],
                color,
                Stroke::NONE,
            ));
        }
        IconKind::Record => {
            painter.circle_filled(rect.center(), w * 0.5, color);
        }
        IconKind::Stop => {
            let s = w * 0.62;
            let r = (s * 0.16).max(1.0).round() as u8;
            painter.rect_filled(
                Rect::from_center_size(rect.center(), Vec2::splat(s)),
                CornerRadius::same(r),
                color,
            );
        }
        IconKind::More => {
            let d = w * 0.22;
            let gap = d * 0.55;
            let total = d * 3.0 + gap * 2.0;
            let mut x = rect.center().x - total * 0.5;
            let y = rect.center().y;
            for _ in 0..3 {
                painter.circle_filled(Pos2::new(x + d * 0.5, y), d * 0.5, color);
                x += d + gap;
            }
        }
        IconKind::Pick => {
            let stroke = Stroke::new((w * 0.16).max(1.5), color);
            let c = rect.center();
            let r = w * 0.30;
            painter.circle_stroke(c, r, stroke);
            painter.line_segment([Pos2::new(c.x, t), Pos2::new(c.x, t + h * 0.18)], stroke);
            painter.line_segment(
                [Pos2::new(c.x, rect.bottom() - h * 0.18), Pos2::new(c.x, rect.bottom())],
                stroke,
            );
            painter.line_segment([Pos2::new(l, c.y), Pos2::new(l + w * 0.18, c.y)], stroke);
            painter.line_segment(
                [Pos2::new(rect.right() - w * 0.18, c.y), Pos2::new(rect.right(), c.y)],
                stroke,
            );
        }
        IconKind::Sun => {
            let size = w.min(h);
            let c = rect.center();
            let core = size * 0.24;
            painter.circle_filled(c, core, color);
            let stroke = Stroke::new((size * 0.09).max(1.4), color);
            for i in 0..8 {
                let angle = i as f32 * std::f32::consts::PI / 4.0;
                let r0 = core * 1.75;
                let r1 = size * 0.47;
                let dir = Vec2::new(angle.cos(), angle.sin());
                painter.line_segment([c + dir * r0, c + dir * r1], stroke);
            }
        }
        IconKind::Moon => {
            let size = w.min(h);
            let outer = Rect::from_center_size(rect.center(), Vec2::splat(size));
            painter.circle_filled(outer.center(), size * 0.5, color);
            painter.circle_filled(
                outer.center() + Vec2::new(size * 0.34, -size * 0.10),
                size * 0.43,
                erase,
            );
        }
    }
}
