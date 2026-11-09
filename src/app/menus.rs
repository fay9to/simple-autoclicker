//! Popup menus, their items and the actions behind them.

use eframe::egui::{Context, Id, Rect};

use crate::dialogs::Dialog;
use crate::settings::AppTheme;
use crate::widgets::{self, MenuItem};

use super::{App, Menu};

impl App {
    pub(super) fn toggle_menu(&mut self, menu: Menu) {
        let same = matches!(
            (&self.menu, &menu),
            (Some(Menu::Theme(_)), Menu::Theme(_))
                | (Some(Menu::Button(_)), Menu::Button(_))
                | (Some(Menu::Type(_)), Menu::Type(_))
                | (Some(Menu::Options(_)), Menu::Options(_))
                | (Some(Menu::Help(_)), Menu::Help(_))
                | (Some(Menu::More(_)), Menu::More(_))
        );

        if same {
            self.menu = None;
        } else {
            self.menu = Some(menu);
            self.menu_fresh = true;
        }
    }

    pub(super) fn draw_menu(&mut self, ctx: &Context) {
        let Some(menu) = self.menu else {
            self.menu_fresh = false;
            return;
        };
        let fresh = self.menu_fresh;
        self.menu_fresh = false;
        let palette = self.palette;

        let (anchor, items, width, id): (Rect, Vec<MenuItem>, f32, &str) = match menu {
            Menu::Theme(rect) => (
                rect,
                vec![
                    MenuItem::check("System", self.settings.theme == AppTheme::System),
                    MenuItem::check("Light", self.settings.theme == AppTheme::Light),
                    MenuItem::check("Dark", self.settings.theme == AppTheme::Dark),
                ],
                150.0,
                "menu-theme",
            ),
            Menu::Button(rect) => (
                rect,
                ["Left", "Right", "Middle"]
                    .iter()
                    .enumerate()
                    .map(|(index, label)| MenuItem::check(label, self.button_idx == index))
                    .collect(),
                178.0,
                "menu-button",
            ),
            Menu::Type(rect) => (
                rect,
                ["Single", "Double"]
                    .iter()
                    .enumerate()
                    .map(|(index, label)| MenuItem::check(label, self.type_idx == index))
                    .collect(),
                178.0,
                "menu-type",
            ),
            Menu::Options(rect) => (
                rect,
                vec![
                    MenuItem::action("Settings…"),
                    MenuItem::action("Hotkeys…"),
                    MenuItem::separator(),
                    MenuItem::caption("Theme"),
                    MenuItem::check("System", self.settings.theme == AppTheme::System),
                    MenuItem::check("Light", self.settings.theme == AppTheme::Light),
                    MenuItem::check("Dark", self.settings.theme == AppTheme::Dark),
                ],
                200.0,
                "menu-options",
            ),
            Menu::Help(rect) => (
                rect,
                vec![
                    MenuItem::action("How to record and playback?"),
                    MenuItem::action("About"),
                ],
                250.0,
                "menu-help",
            ),
            Menu::More(rect) => (
                rect,
                vec![
                    MenuItem::action("Clear recording"),
                    MenuItem::separator(),
                    MenuItem::action("Settings…"),
                    MenuItem::action("Hotkeys…"),
                ],
                200.0,
                "menu-more",
            ),
        };

        let (clicked, popup_rect) =
            widgets::popup_menu(ctx, Id::unique(id), anchor, width, &items, &palette);

        if !fresh && ctx.input(|i| i.pointer.any_pressed()) {
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
            Menu::Button(_) => {
                self.button_idx = index.min(2);
            }
            Menu::Type(_) => {
                self.type_idx = index.min(1);
            }
            Menu::Options(_) => match index {
                0 => self.open_dialog(
                    ctx,
                    Dialog::AdvSettings {
                        minimize_play: self.settings.minimize_when_playing,
                        minimize_record: self.settings.minimize_when_recording,
                    },
                ),
                1 => self.open_advanced_hotkeys(ctx),
                4 => self.set_theme(ctx, AppTheme::System),
                5 => self.set_theme(ctx, AppTheme::Light),
                6 => self.set_theme(ctx, AppTheme::Dark),
                _ => {}
            },
            Menu::Help(_) => {
                if index == 0 {
                    self.open_dialog(
                        ctx,
                        Dialog::Info {
                            text: "Press Record, perform the clicks and mouse movements you need, then press Stop.\nPlayback repeats the recording at the same speed."
                                .to_string(),
                            warning: false,
                        },
                    );
                } else {
                    self.open_dialog(
                        ctx,
                        Dialog::Info {
                            text: "Simple Auto Clicker 3.1.1 — record and playback of mouse actions."
                                .to_string(),
                            warning: false,
                        },
                    );
                }
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
