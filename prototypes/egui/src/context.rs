//! App wiring: the read-only `Services` handed to every surface, the floater registry trait, and a
//! small command bus that carries floater-open and set-language events back to `MockApp`.

use eframe::egui;
use unic_langid::LanguageIdentifier;

use crate::data;
use crate::theme::fonts::{MonoFont, UiFont};
use crate::theme::{RadiusPref, Theme};

// Read-only context threaded into every screen and floater, in place of per-call `data`/`lang`
// plumbing. Surfaces emit changes back via `post`, not by mutating through here.
pub struct Services<'a> {
    pub model: &'a dyn data::Model,
    pub lang: &'a LanguageIdentifier,
    pub theme: Theme,
    pub radius: RadiusPref,
    pub ui_font: UiFont,
    pub mono_font: MonoFont,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FloaterKind {
    About,
    DebugSettings,
    ConfirmQuit,
    Preferences,
}

impl FloaterKind {
    // Stable identifier used as the egui window id and the persisted open-floater key.
    pub fn key(self) -> &'static str {
        match self {
            Self::About => "about",
            Self::DebugSettings => "debug_settings",
            Self::ConfirmQuit => "confirm_quit",
            Self::Preferences => "preferences",
        }
    }
}

// A floating window/modal in the registry. `MockApp` iterates these uniformly; adding one is just a
// new `impl` plus one `Box` in `MockApp::new`.
pub trait Floater {
    fn kind(&self) -> FloaterKind;
    fn open(&self) -> bool;
    fn set_open(&mut self, open: bool);
    fn show(&mut self, ctx: &egui::Context, services: &Services);
}

// Deferred app actions: surfaces `post` during their `show`, `MockApp::update` drains them next
// frame. Generalizes the old floater open-queue so language changes flow through the same channel.
#[derive(Clone)]
pub enum Command {
    OpenFloater(FloaterKind),
    SetLanguage(LanguageIdentifier),
    SetTheme(Theme),
    SetRadius(RadiusPref),
    SetUiFont(UiFont),
    SetMonoFont(MonoFont),
}

fn queue_id() -> egui::Id {
    egui::Id::new("command_queue")
}

pub fn post(ctx: &egui::Context, command: Command) {
    ctx.data_mut(|d| {
        d.get_temp_mut_or_default::<Vec<Command>>(queue_id())
            .push(command);
    });
}

pub fn drain(ctx: &egui::Context) -> Vec<Command> {
    ctx.data_mut(|d| std::mem::take(d.get_temp_mut_or_default::<Vec<Command>>(queue_id())))
}
