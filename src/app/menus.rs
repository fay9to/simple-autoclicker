//! Popup menus, their items and the actions behind them.

use eframe::egui::{Context, Id, Rect};

use crate::dialogs::Dialog;
use crate::i18n::{self, Key, Lang};
use crate::settings::AppTheme;
use crate::theme::ease_out;
use crate::widgets::{self, MenuItem};

use super::{App, Menu};

impl App {
    pub(super) fn toggle_menu(&mut self, menu: Menu) {
        let same = matches!(
            (&self.menu, &menu),
            (Some(Menu::Theme(_)), Menu::Theme(_))
                | (Some(Menu::Language(_)), Menu::Language(_))
                | (Some(Menu::Button(_)), Menu::Button(_))
                | (Some(Menu::Type(_)), Menu::Type(_))
                | (Some(Menu::More(_)), Menu::More(_))
        );

        if same {
            self.menu = None;
        } else {
            self.menu = Some(menu);
            self.menu_since = std::time::Instant::now();
        }
    }

    pub(super) fn draw_menu(&mut self, ctx: &Context) {
        let Some(menu) = self.menu else {
            return;
        };
        let palette = self.palette;
        let appear = ease_out(self.menu_since.elapsed().as_secs_f32() / 0.14);

        let (anchor, items, width, id): (Rect, Vec<MenuItem>, f32, &str) = match menu {
            Menu::Theme(rect) => (
                rect,
                vec![
                    MenuItem::check(self.tr(Key::ThemeSystem), self.settings.theme == AppTheme::System),
                    MenuItem::check(self.tr(Key::ThemeLight), self.settings.theme == AppTheme::Light),
                    MenuItem::check(self.tr(Key::ThemeDark), self.settings.theme == AppTheme::Dark),
                ],
                178.0,
                "menu-theme",
            ),
            Menu::Language(rect) => (
                rect,
                Lang::ALL
                    .iter()
                    .map(|lang| MenuItem::check(lang.native(), self.lang == *lang))
                    .collect(),
                186.0,
                "menu-language",
            ),
            Menu::Button(rect) => (
                rect,
                vec![
                    MenuItem::check(self.tr(Key::Left), self.button_idx == 0),
                    MenuItem::check(self.tr(Key::Right), self.button_idx == 1),
                    MenuItem::check(self.tr(Key::Middle), self.button_idx == 2),
                ],
                180.0,
                "menu-button",
            ),
            Menu::Type(rect) => (
                rect,
                vec![
                    MenuItem::check(self.tr(Key::Single), self.type_idx == 0),
                    MenuItem::check(self.tr(Key::Double), self.type_idx == 1),
                ],
                180.0,
                "menu-type",
            ),
            Menu::More(rect) => (
                rect,
                vec![
                    MenuItem::action(self.tr(Key::ClearRecording)),
                    MenuItem::separator(),
                    MenuItem::action(self.tr(Key::Settings)),
                    MenuItem::action(self.tr(Key::Hotkeys)),
                    MenuItem::separator(),
                    MenuItem::action(self.tr(Key::HowToRecord)),
                    MenuItem::action(self.tr(Key::About)),
                ],
                250.0,
                "menu-more",
            ),
        };

        let (clicked, popup_rect) =
            widgets::popup_menu(ctx, Id::unique(id), anchor, width, &items, appear, &palette);

        if ctx.input(|i| i.pointer.any_pressed()) {
            if let Some(position) = ctx.input(|i| i.pointer.interact_pos()) {
                if !popup_rect.contains(position) && !anchor.contains(position) {
                    self.menu = None;
                    return;
                }
            }
        }

        let Some(index) = clicked else {
            return;
        };

        match menu {
            Menu::Theme(_) => {
                let mode = match index {
                    1 => AppTheme::Light,
                    2 => AppTheme::Dark,
                    _ => AppTheme::System,
                };
                self.set_theme(ctx, mode);
            }
            Menu::Language(_) => {
                if let Some(lang) = Lang::ALL.get(index) {
                    self.set_language(*lang);
                }
            }
            Menu::Button(_) => {
                self.button_idx = index.min(2);
            }
            Menu::Type(_) => {
                self.type_idx = index.min(1);
            }
            Menu::More(_) => match index {
                0 => {
                    if !self.rec.recording && !self.rec.playing {
                        self.rec.events.clear();
                    }
                }
                2 => self.open_dialog(
                    ctx,
                    Dialog::AdvSettings {
                        minimize_play: self.settings.minimize_when_playing,
                        minimize_record: self.settings.minimize_when_recording,
                    },
                ),
                3 => self.open_advanced_hotkeys(ctx),
                5 => self.open_dialog(
                    ctx,
                    Dialog::Info {
                        text: self.tr(Key::HowToText).to_string(),
                        warning: false,
                    },
                ),
                6 => self.open_dialog(
                    ctx,
                    Dialog::Info {
                        text: i18n::about_text(self.lang),
                        warning: false,
                    },
                ),
                _ => {}
            },
        }

        self.menu = None;
    }

    fn open_advanced_hotkeys(&mut self, ctx: &Context) {
        self.open_dialog(
            ctx,
            Dialog::AdvHotkeys {
                play: self.settings.playback_hotkey,
                record: self.settings.record_hotkey,
                active: 0,
            },
        );
    }
}
