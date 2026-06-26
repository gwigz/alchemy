use eframe::egui;
use egui_notify::Toasts;
use unic_langid::LanguageIdentifier;

use crate::context::{self, Command, Floater, FloaterKind, Services};
use crate::data::MockData;
use crate::screens::{
    self, about::AboutState, confirm_quit::ConfirmQuitState, debug_settings::DebugSettingsState,
    login::LoginState, preferences::PreferencesState, Screen,
};
use crate::theme::fonts::{MonoFont, UiFont};
use crate::theme::{RadiusPref, Theme};
use crate::{i18n, notify, theme};

// localStorage / RON key for the persisted dev-loop snapshot.
const SESSION_KEY: &str = "mock_session";

// A minimal snapshot persisted across hot reloads. Selectors are stored by their registry name
// strings so the underlying enum/lang types stay plain (no serde derives leak onto them). Only the
// dev-loop annoyances are captured here; floater positions persist separately via egui memory.
#[derive(serde::Serialize, serde::Deserialize, Default)]
struct Session {
    screen: usize,
    // FloaterKind::key() of each open floater (ConfirmQuit excluded).
    open_floaters: Vec<String>,
    lang: String,
    theme: String,
    radius: String,
    ui_font: String,
    mono_font: String,
}

pub struct MockApp {
    data: MockData,
    screen: Screen,
    login: LoginState,
    // The floater registry. Adding a floater is a new `impl Floater` plus one `Box` in `new`.
    floaters: Vec<Box<dyn Floater>>,
    lang: LanguageIdentifier,
    theme: Theme,
    radius: RadiusPref,
    ui_font: UiFont,
    mono_font: MonoFont,
    applied_fonts: (UiFont, MonoFont),
    toasts: Toasts,
}

impl MockApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        egui_extras::install_image_loaders(&cc.egui_ctx);

        // Restore the previous session if one was persisted (survives trunk hot reloads); otherwise
        // fall back to the same hardcoded defaults as before.
        let session = cc
            .storage
            .and_then(|s| eframe::get_value::<Session>(s, SESSION_KEY))
            .unwrap_or_default();

        let theme = value_of(theme::themes(), &session.theme, Theme::Default);
        let radius = value_of(theme::radii(), &session.radius, RadiusPref::Small);
        let ui_font = value_of(theme::fonts::ui_fonts(), &session.ui_font, UiFont::Inter);
        let mono_font = value_of(
            theme::fonts::mono_fonts(),
            &session.mono_font,
            MonoFont::CascadiaCode,
        );

        let lang = lang_value(&session.lang);
        let screen = Screen::ALL
            .get(session.screen)
            .copied()
            .unwrap_or(Screen::Login);

        theme::install(&cc.egui_ctx, theme, radius);
        theme::fonts::install_fonts(&cc.egui_ctx, ui_font, mono_font);

        let mut floaters: Vec<Box<dyn Floater>> = vec![
            Box::new(AboutState::default()),
            Box::new(DebugSettingsState::default()),
            Box::new(ConfirmQuitState::default()),
            Box::new(PreferencesState::default()),
        ];

        for f in &mut floaters {
            f.set_open(session.open_floaters.iter().any(|k| k == f.kind().key()));
        }

        Self {
            data: MockData::load(),
            screen,
            login: LoginState::default(),
            floaters,
            lang,
            theme,
            radius,
            ui_font,
            mono_font,
            applied_fonts: (ui_font, mono_font),
            toasts: Toasts::default(),
        }
    }
}

impl eframe::App for MockApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        theme::install(ctx, self.theme, self.radius);

        // Fonts rebuild the glyph atlas; only reinstall when the choice changes.
        if self.applied_fonts != (self.ui_font, self.mono_font) {
            theme::fonts::install_fonts(ctx, self.ui_font, self.mono_font);
            self.applied_fonts = (self.ui_font, self.mono_font);
        }

        egui::TopBottomPanel::top("screen_picker").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Prototype:");

                for &screen in Screen::ALL {
                    ui.selectable_value(&mut self.screen, screen, screen.title());
                }

                ui.separator();

                let langs = i18n::available();
                let current = langs
                    .iter()
                    .find(|(_, id)| *id == self.lang)
                    .map_or("English", |(name, _)| *name);

                egui::ComboBox::from_id_salt("locale")
                    .selected_text(current)
                    .show_ui(ui, |ui| {
                        for (name, id) in &langs {
                            ui.selectable_value(&mut self.lang, id.clone(), *name);
                        }
                    });

                ui.separator();
                registry_combo(ui, "theme", theme::themes(), &mut self.theme);
                registry_combo(ui, "radius", theme::radii(), &mut self.radius);
                registry_combo(ui, "ui_font", theme::fonts::ui_fonts(), &mut self.ui_font);
                registry_combo(
                    ui,
                    "mono_font",
                    theme::fonts::mono_fonts(),
                    &mut self.mono_font,
                );
            });
        });

        // Apply deferred actions posted by surfaces last frame (floater opens, language picks).
        for cmd in context::drain(ctx) {
            match cmd {
                Command::OpenFloater(kind) => {
                    if let Some(f) = self.floaters.iter_mut().find(|f| f.kind() == kind) {
                        f.set_open(true);
                    }
                }
                Command::SetLanguage(id) => self.lang = id,
                Command::SetTheme(t) => self.theme = t,
                Command::SetRadius(r) => self.radius = r,
                Command::SetUiFont(f) => self.ui_font = f,
                Command::SetMonoFont(f) => self.mono_font = f,
            }
        }

        // Screens own their full bleed-to-edge layout, so drop the frame margin.
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ctx, |ui| {
                ui.spacing_mut().interact_size.y = theme::field_h();

                let services = Services {
                    model: &self.data,
                    lang: &self.lang,
                    theme: self.theme,
                    radius: self.radius,
                    ui_font: self.ui_font,
                    mono_font: self.mono_font,
                };
                match self.screen {
                    Screen::Login => screens::login::show(ui, &mut self.login, &services),
                }
            });

        let services = Services {
            model: &self.data,
            lang: &self.lang,
            theme: self.theme,
            radius: self.radius,
            ui_font: self.ui_font,
            mono_font: self.mono_font,
        };
        for f in &mut self.floaters {
            f.show(ctx, &services);
        }

        for notice in notify::drain(ctx) {
            match notice.level {
                notify::Level::Info => self.toasts.info(notice.text),
                notify::Level::Success => self.toasts.success(notice.text),
                notify::Level::Error => self.toasts.error(notice.text),
            };
        }

        self.toasts.show(ctx);
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        // Persist open floaters except the quit modal (don't re-pop it on reload).
        let open_floaters = self
            .floaters
            .iter()
            .filter(|f| f.open() && f.kind() != FloaterKind::ConfirmQuit)
            .map(|f| f.kind().key().to_owned())
            .collect();

        let session = Session {
            screen: Screen::ALL
                .iter()
                .position(|s| *s == self.screen)
                .unwrap_or(0),
            open_floaters,
            lang: lang_name(&self.lang),
            theme: name_of(theme::themes(), self.theme),
            radius: name_of(theme::radii(), self.radius),
            ui_font: name_of(theme::fonts::ui_fonts(), self.ui_font),
            mono_font: name_of(theme::fonts::mono_fonts(), self.mono_font),
        };
        eframe::set_value(storage, SESSION_KEY, &session);
    }
}

// Look up a registry entry's name from its value, and vice versa. Shared by the session snapshot
// (save reads names off the live selectors, restore resolves them back to values).
fn name_of<T: Copy + PartialEq>(options: &[(&'static str, T)], value: T) -> String {
    options
        .iter()
        .find(|(_, v)| *v == value)
        .map_or_else(String::new, |(name, _)| (*name).to_owned())
}

fn value_of<T: Copy>(options: &[(&'static str, T)], name: &str, default: T) -> T {
    options
        .iter()
        .find(|(n, _)| *n == name)
        .map_or(default, |(_, v)| *v)
}

// Language equivalents of `name_of`/`value_of`; `LanguageIdentifier` isn't `Copy`, so these clone.
fn lang_name(lang: &LanguageIdentifier) -> String {
    i18n::available()
        .into_iter()
        .find(|(_, id)| id == lang)
        .map_or_else(String::new, |(name, _)| name.to_owned())
}

fn lang_value(name: &str) -> LanguageIdentifier {
    let langs = i18n::available();
    langs
        .iter()
        .find(|(n, _)| *n == name)
        .map_or_else(|| langs[0].1.clone(), |(_, id)| id.clone())
}

fn registry_combo<T: Copy + PartialEq>(
    ui: &mut egui::Ui,
    id: &str,
    options: &[(&'static str, T)],
    selected: &mut T,
) {
    let current = options
        .iter()
        .find(|(_, value)| value == selected)
        .map_or("", |(name, _)| *name);

    egui::ComboBox::from_id_salt(id)
        .selected_text(current)
        .show_ui(ui, |ui| {
            for (name, value) in options {
                ui.selectable_value(selected, *value, *name);
            }
        });
}
