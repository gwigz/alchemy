//! Shared helpers for the preferences panels: search highlighting, hints, combo option resolution,
//! and the per-panel render context.

use eframe::egui::{self, Color32, RichText};
use twill::tokens::{FontSize, Spacing};
use unic_langid::LanguageIdentifier;

use crate::context::Services;
use crate::i18n::tr;
use crate::theme;

// Read-only context handed to each panel's `show`, bundling the active locale/selectors and the
// current search query so sections don't thread `(q, lang)` everywhere.
pub(super) struct PanelCtx<'a> {
    pub services: &'a Services<'a>,
    // Trimmed + lowercased search query, for label highlighting.
    pub q: &'a str,
}

impl PanelCtx<'_> {
    pub fn lang(&self) -> &LanguageIdentifier {
        self.services.lang
    }
}

// A slightly shorter field height for the compact selects and search box.
pub(super) fn compact_h() -> f32 {
    theme::space(Spacing::S6)
}

// Resolve a list of ftl keys to display strings in the active language.
pub(super) fn options(lang: &LanguageIdentifier, keys: &[&str]) -> Vec<String> {
    keys.iter().map(|k| tr(lang, k)).collect()
}

// A dimmed parenthetical hint under a control.
pub(super) fn hint(ui: &mut egui::Ui, text: &str) {
    let p = theme::active(ui.ctx());
    ui.label(
        RichText::new(text)
            .color(theme::mix(p.muted_foreground, p.popover, 0.45))
            .size(theme::font(FontSize::Base)),
    );
}

fn matches(text: &str, q: &str) -> bool {
    !q.is_empty() && text.to_lowercase().contains(q)
}

fn highlight_color(p: &theme::Palette) -> Color32 {
    theme::mix(p.popover, p.destructive, 0.55)
}

// Inline-highlight a control's own label (checkbox/radio text) when it matches the query.
pub(super) fn hl(text: &str, q: &str, p: &theme::Palette) -> RichText {
    let rt = RichText::new(text);
    if matches(text, q) {
        rt.background_color(highlight_color(p))
    } else {
        rt
    }
}

// A section label that highlights (and brightens) when it matches the query.
pub(super) fn hl_label(ui: &mut egui::Ui, text: &str, q: &str) {
    let p = theme::active(ui.ctx());
    let mut rt = RichText::new(text).size(theme::font(FontSize::Base));
    if matches(text, q) {
        rt = rt.color(p.foreground).background_color(highlight_color(&p));
    } else {
        rt = rt.color(p.muted_foreground);
    }
    ui.label(rt);
    ui.add_space(theme::space(Spacing::S0_5));
}
