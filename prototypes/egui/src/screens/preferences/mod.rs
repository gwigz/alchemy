//! Preferences floater, modeled on SL's `floater_preferences` (mock: edits aren't persisted).
//!
//! This module is the shell: window chrome, search box, sidebar, actions, and category dispatch.
//! Each category's panel lives in its own submodule and owns its state, constants, and search keys.

mod common;
mod general;
mod themes;

use eframe::egui::{self, RichText};
use twill::tokens::Spacing;
use unic_langid::LanguageIdentifier;

use common::{compact_h, PanelCtx};

use crate::context::{self, FloaterKind, Services};
use crate::i18n::tr;
use crate::{floater, theme, widgets};

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum Category {
    #[default]
    General,
    Graphics,
    SoundMedia,
    Chat,
    MoveView,
    Notifications,
    Colors,
    Privacy,
    Setup,
    Advanced,
    Uploads,
    Controls,
    Themes,
    Interface,
}

impl Category {
    const ALL: &'static [Category] = &[
        Self::General,
        Self::Graphics,
        Self::SoundMedia,
        Self::Chat,
        Self::MoveView,
        Self::Notifications,
        Self::Colors,
        Self::Privacy,
        Self::Setup,
        Self::Advanced,
        Self::Uploads,
        Self::Controls,
        Self::Themes,
        Self::Interface,
    ];

    // ftl key for the category's display name (used by the sidebar and by search).
    const fn key(self) -> &'static str {
        match self {
            Self::General => "pref-cat-general",
            Self::Graphics => "pref-cat-graphics",
            Self::SoundMedia => "pref-cat-sound-media",
            Self::Chat => "pref-cat-chat",
            Self::MoveView => "pref-cat-move-view",
            Self::Notifications => "pref-cat-notifications",
            Self::Colors => "pref-cat-colors",
            Self::Privacy => "pref-cat-privacy",
            Self::Setup => "pref-cat-setup",
            Self::Advanced => "pref-cat-advanced",
            Self::Uploads => "pref-cat-uploads",
            Self::Controls => "pref-cat-controls",
            Self::Themes => "pref-cat-themes",
            Self::Interface => "pref-cat-interface",
        }
    }

    // i18n label keys a panel renders; search resolves them in the active language. Owned by the
    // panel module so a new panel brings its own keys.
    const fn search_keys(self) -> &'static [&'static str] {
        match self {
            Self::General => general::SEARCH_KEYS,
            _ => &[],
        }
    }

    // Literal (un-harvested) labels a panel renders. The Themes panel is Alchemy/mock-specific with
    // no SL XUI source, so its labels are matched directly rather than via `tr`.
    const fn search_literals(self) -> &'static [&'static str] {
        match self {
            Self::Themes => themes::SEARCH_LITERALS,
            _ => &[],
        }
    }

    // Match against the localized category name plus its rendered labels, so search works in any
    // locale. `q` arrives already trimmed and lowercased.
    fn matches(self, q: &str, lang: &LanguageIdentifier) -> bool {
        if q.is_empty() {
            return true;
        }
        let hit = |s: &str| s.to_lowercase().contains(q);
        hit(&tr(lang, self.key()))
            || self.search_keys().iter().any(|k| hit(&tr(lang, k)))
            || self.search_literals().iter().any(|s| hit(s))
    }
}

#[derive(Default)]
pub struct PreferencesState {
    pub open: bool,
    search: String,
    category: Category,
    general: general::State,
    themes: themes::State,
}

impl context::Floater for PreferencesState {
    fn kind(&self) -> FloaterKind {
        FloaterKind::Preferences
    }

    fn open(&self) -> bool {
        self.open
    }

    fn set_open(&mut self, open: bool) {
        self.open = open;
    }

    fn show(&mut self, ctx: &egui::Context, services: &Services) {
        if !self.open {
            return;
        }

        let mut open = true;
        let mut close = false;
        let size = egui::vec2(620.0, 540.0);

        floater::window(
            ctx,
            FloaterKind::Preferences.key(),
            tr(services.lang, "pref-title"),
            floater::WindowOpts {
                default_size: size,
                // Floor the height at the General panel's content so it never needs the central scroll.
                min_size: Some(egui::vec2(560.0, 540.0)),
                fixed_size: None,
                collapsible: true,
            },
            &mut open,
            |ui| contents(ui, self, &mut close, services),
        );

        // The language combo posts `SetLanguage` rather than mutating here, so reading `services.lang`
        // stays sound; the pick applies next frame.
        self.open = open && !close;
    }
}

fn contents(
    ui: &mut egui::Ui,
    state: &mut PreferencesState,
    close: &mut bool,
    services: &Services,
) {
    let lang = services.lang;

    egui::TopBottomPanel::top("prefs_search")
        .frame(egui::Frame::NONE)
        .show_separator_line(false)
        .show_inside(ui, |ui| {
            ui.add_space(theme::space(Spacing::S1));
            widgets::field::text_full_sized(
                ui,
                &mut state.search,
                &tr(lang, "pref-search"),
                compact_h(),
            );
            ui.add_space(theme::space(Spacing::S1));
        });

    egui::TopBottomPanel::bottom("prefs_actions")
        .frame(egui::Frame::NONE)
        .show_separator_line(false)
        .show_inside(ui, |ui| {
            let size = egui::vec2(theme::field_w() / 2.0, theme::field_h());
            let action = |ui: &mut egui::Ui, text: &str| {
                widgets::field::button(ui, widgets::field::ButtonVariant::Secondary, size, text)
            };
            ui.add_space(theme::space(Spacing::S1));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if action(ui, &tr(lang, "pref-cancel")).clicked() {
                    *close = true;
                }
                ui.add_space(theme::space(Spacing::S2));
                if action(ui, &tr(lang, "pref-ok")).clicked() {
                    *close = true;
                }
            });
        });

    egui::CentralPanel::default()
        .frame(egui::Frame::NONE)
        .show_inside(ui, |ui| {
            let q = state.search.trim().to_lowercase();
            let filtered: Vec<Category> = Category::ALL
                .iter()
                .copied()
                .filter(|c| c.matches(&q, lang))
                .collect();

            if !filtered.is_empty() && !filtered.contains(&state.category) {
                state.category = filtered[0];
            }

            ui.horizontal_top(|ui| {
                sidebar(ui, state, &filtered, lang);
                ui.add_space(theme::space(Spacing::S2));
                if !filtered.is_empty() {
                    let cx = PanelCtx { services, q: &q };
                    match state.category {
                        Category::General => general::show(ui, &mut state.general, &cx),
                        Category::Themes => themes::show(ui, &mut state.themes, &cx),
                        _ => placeholder(ui),
                    }
                }
            });
        });
}

fn sidebar(
    ui: &mut egui::Ui,
    state: &mut PreferencesState,
    filtered: &[Category],
    lang: &LanguageIdentifier,
) {
    let width = 140.0;

    let labels: Vec<(Category, String)> =
        filtered.iter().map(|c| (*c, tr(lang, c.key()))).collect();
    let tabs: Vec<(Category, &str)> = labels.iter().map(|(c, s)| (*c, s.as_str())).collect();

    widgets::tabs::column(ui, &mut state.category, width, &tabs);
}

fn placeholder(ui: &mut egui::Ui) {
    let p = theme::active(ui.ctx());
    ui.allocate_ui_with_layout(
        ui.available_size(),
        egui::Layout::centered_and_justified(egui::Direction::TopDown),
        |ui| {
            ui.label(
                RichText::new("This panel isn't implemented in this mock.")
                    .color(p.muted_foreground),
            );
        },
    );
}
