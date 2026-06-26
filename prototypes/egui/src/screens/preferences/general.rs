//! The General preferences panel.

use eframe::egui::{self, Align2, Color32, FontId, Sense};
use twill::tokens::{FontSize, Spacing};

use super::common::{compact_h, hint, hl, hl_label, options, PanelCtx};
use crate::context::{self, Command};
use crate::i18n::tr;
use crate::{theme, widgets};

#[derive(Clone, Copy, PartialEq, Eq, Default)]
enum NameTags {
    Off,
    #[default]
    On,
    ShowBriefly,
}

#[derive(Clone, Copy, PartialEq, Eq, Default)]
enum LetterKeys {
    #[default]
    StartsChat,
    AffectsMovement,
}

// Combo options as harvested ftl keys (ascending, mildest first). Resolved per-locale at render.
const TIME_FORMAT_KEYS: &[&str] = &["pref-time-12h", "pref-time-24h"];
const CONTENT_RATING_KEYS: &[&str] = &["pref-rating-pg", "pref-rating-mature", "pref-rating-adult"];
const GROUP_TAG_KEYS: &[&str] = &["pref-tags-none", "pref-tags-mine", "pref-tags-all"];
const AWAY_TIMEOUT_KEYS: &[&str] = &[
    "pref-afk-2",
    "pref-afk-5",
    "pref-afk-10",
    "pref-afk-30",
    "pref-afk-never",
];

const DND_DEFAULT: &str =
    "This resident has turned on 'Do Not Disturb' and will see your message later.";

// i18n keys for every label this panel renders through `hl` / `hl_label`. Search resolves these in
// the active language. Keep in sync with the `hl_label(...)` / `hl(...)` calls below.
pub(super) const SEARCH_KEYS: &[&str] = &[
    "pref-language",
    "pref-time-format",
    "pref-content-rated",
    "pref-show-favorites",
    "pref-name-tags",
    "pref-tag-off",
    "pref-tag-on",
    "pref-tag-brief",
    "pref-my-name",
    "pref-usernames",
    "pref-distance",
    "pref-display-names",
    "pref-highlight-friends",
    "pref-letter-keys",
    "pref-keys-chat",
    "pref-keys-move",
    "pref-away-timeout",
    "pref-dnd-response",
];

#[allow(clippy::struct_excessive_bools)]
pub(super) struct State {
    time_format: usize,
    content_rating: usize,
    group_tags: usize,
    away_timeout: usize,
    show_favorites: bool,
    name_tags: NameTags,
    show_my_name: bool,
    show_usernames: bool,
    show_distance: bool,
    show_display_names: bool,
    highlight_friends: bool,
    letter_keys: LetterKeys,
    dnd_response: String,
}

impl Default for State {
    fn default() -> Self {
        Self {
            time_format: 0,
            content_rating: 2,
            group_tags: 2,
            away_timeout: 1,
            show_favorites: true,
            name_tags: NameTags::On,
            show_my_name: true,
            show_usernames: false,
            show_distance: false,
            show_display_names: true,
            highlight_friends: true,
            letter_keys: LetterKeys::StartsChat,
            dnd_response: DND_DEFAULT.to_owned(),
        }
    }
}

pub(super) fn show(ui: &mut egui::Ui, state: &mut State, cx: &PanelCtx) {
    egui::ScrollArea::vertical()
        .id_salt("prefs_general")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            // Reset to top-down: the parent `horizontal_top` layout would otherwise
            // flow these rows left-to-right.
            ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                let p = theme::active(ui.ctx());
                ui.set_min_width(ui.available_width());

                language_row(ui, state, cx);
                ui.add_space(theme::space(Spacing::S2));
                content_rating_row(ui, state, cx);
                ui.add_space(theme::space(Spacing::S2));

                ui.checkbox(
                    &mut state.show_favorites,
                    hl(&tr(cx.lang(), "pref-show-favorites"), cx.q, &p),
                );
                hint(ui, &tr(cx.lang(), "pref-favorites-hint"));
                ui.add_space(theme::space(Spacing::S2));

                name_tags_section(ui, state, cx);
                ui.add_space(theme::space(Spacing::S2));
                letter_keys_section(ui, state, cx);
                ui.add_space(theme::space(Spacing::S2));
                away_timeout_row(ui, state, cx);
                ui.add_space(theme::space(Spacing::S2));
                dnd_section(ui, state, cx);
            });
        });
}

fn language_row(ui: &mut egui::Ui, state: &mut State, cx: &PanelCtx) {
    let lang = cx.lang();
    let langs = crate::i18n::available();
    let mut idx = langs.iter().position(|(_, id)| id == lang).unwrap_or(0);

    ui.horizontal_top(|ui| {
        ui.vertical(|ui| {
            hl_label(ui, &tr(lang, "pref-language"), cx.q);
            widgets::combo::select_sized(
                ui,
                "prefs_lang",
                &mut idx,
                langs.iter().map(|(name, _)| *name),
                egui::vec2(theme::field_w(), compact_h()),
            );
        });
        ui.add_space(theme::space(Spacing::S2));
        ui.vertical(|ui| {
            hl_label(ui, &tr(lang, "pref-time-format"), cx.q);
            let opts = options(lang, TIME_FORMAT_KEYS);
            widgets::combo::select_sized(
                ui,
                "prefs_time",
                &mut state.time_format,
                opts.iter().map(String::as_str),
                egui::vec2(theme::field_w() / 2.0, compact_h()),
            );
        });
    });

    if let Some((_, id)) = langs.get(idx) {
        if id != lang {
            context::post(ui.ctx(), Command::SetLanguage(id.clone()));
        }
    }
}

fn content_rating_row(ui: &mut egui::Ui, state: &mut State, cx: &PanelCtx) {
    hl_label(ui, &tr(cx.lang(), "pref-content-rated"), cx.q);
    ui.horizontal(|ui| {
        let opts = options(cx.lang(), CONTENT_RATING_KEYS);
        widgets::combo::select_sized(
            ui,
            "prefs_rating",
            &mut state.content_rating,
            opts.iter().map(String::as_str),
            egui::vec2(theme::field_w(), compact_h()),
        );
        ui.add_space(theme::space(Spacing::S2));
        // Only the ratings the selection grants access to are shown (General -> G, +Moderate -> M,
        // +Adult -> A), mirroring the cumulative content-rating combo.
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = theme::space(Spacing::S0_5);
            let badges = [
                ("G", Color32::from_rgb(176, 206, 226)),
                ("M", Color32::from_rgb(203, 203, 204)),
                ("A", Color32::from_rgb(218, 169, 170)),
            ];
            for (letter, fill) in badges.iter().take(state.content_rating + 1) {
                rating_badge(ui, letter, *fill);
            }
        });
    });
}

fn name_tags_section(ui: &mut egui::Ui, state: &mut State, cx: &PanelCtx) {
    let lang = cx.lang();
    let q = cx.q;
    let p = theme::active(ui.ctx());
    hl_label(ui, &tr(lang, "pref-name-tags"), q);

    // Lay the options out in three aligned columns, like the viewer's name-tag block, rather than
    // packing them left-to-right.
    ui.columns(3, |c| {
        c[0].radio_value(
            &mut state.name_tags,
            NameTags::Off,
            hl(&tr(lang, "pref-tag-off"), q, &p),
        );
        c[1].radio_value(
            &mut state.name_tags,
            NameTags::On,
            hl(&tr(lang, "pref-tag-on"), q, &p),
        );
        c[2].radio_value(
            &mut state.name_tags,
            NameTags::ShowBriefly,
            hl(&tr(lang, "pref-tag-brief"), q, &p),
        );
    });
    ui.add_space(theme::space(Spacing::S1));
    ui.columns(3, |c| {
        c[0].checkbox(
            &mut state.show_my_name,
            hl(&tr(lang, "pref-my-name"), q, &p),
        );
        c[1].checkbox(
            &mut state.show_usernames,
            hl(&tr(lang, "pref-usernames"), q, &p),
        );
        c[2].checkbox(
            &mut state.show_distance,
            hl(&tr(lang, "pref-distance"), q, &p),
        );
    });
    ui.columns(3, |c| {
        c[0].checkbox(
            &mut state.show_display_names,
            hl(&tr(lang, "pref-display-names"), q, &p),
        );
        c[1].checkbox(
            &mut state.highlight_friends,
            hl(&tr(lang, "pref-highlight-friends"), q, &p),
        );
    });
    ui.add_space(theme::space(Spacing::S1));
    let opts = options(lang, GROUP_TAG_KEYS);
    widgets::combo::select_sized(
        ui,
        "prefs_grouptags",
        &mut state.group_tags,
        opts.iter().map(String::as_str),
        egui::vec2(theme::field_w(), compact_h()),
    );
}

fn letter_keys_section(ui: &mut egui::Ui, state: &mut State, cx: &PanelCtx) {
    let lang = cx.lang();
    let p = theme::active(ui.ctx());
    hl_label(ui, &tr(lang, "pref-letter-keys"), cx.q);
    ui.radio_value(
        &mut state.letter_keys,
        LetterKeys::StartsChat,
        hl(&tr(lang, "pref-keys-chat"), cx.q, &p),
    );
    ui.radio_value(
        &mut state.letter_keys,
        LetterKeys::AffectsMovement,
        hl(&tr(lang, "pref-keys-move"), cx.q, &p),
    );
}

fn away_timeout_row(ui: &mut egui::Ui, state: &mut State, cx: &PanelCtx) {
    hl_label(ui, &tr(cx.lang(), "pref-away-timeout"), cx.q);
    let opts = options(cx.lang(), AWAY_TIMEOUT_KEYS);
    widgets::combo::select_sized(
        ui,
        "prefs_away",
        &mut state.away_timeout,
        opts.iter().map(String::as_str),
        egui::vec2(theme::field_w(), compact_h()),
    );
}

fn dnd_section(ui: &mut egui::Ui, state: &mut State, cx: &PanelCtx) {
    hl_label(ui, &tr(cx.lang(), "pref-dnd-response"), cx.q);

    let two_lines = ui.text_style_height(&egui::TextStyle::Body) * 2.0;
    widgets::field::multiline(
        ui,
        "prefs_dnd",
        &mut state.dnd_response,
        widgets::field::MultilineOpts {
            rows: 2,
            readonly: false,
            scroll_height: Some(two_lines),
        },
    );
}

// Maturity badge, colored to match SL's `Parcel_PG/M/R_Dark` icons (sampled from the textures).
fn rating_badge(ui: &mut egui::Ui, letter: &str, fill: Color32) {
    let side = theme::space(Spacing::S5);
    let (rect, _) = ui.allocate_exact_size(egui::vec2(side, side), Sense::hover());
    let corner = theme::corner(ui.ctx());

    ui.painter().rect_filled(rect, corner, fill);
    ui.painter().text(
        rect.center(),
        Align2::CENTER_CENTER,
        letter,
        FontId::proportional(theme::font(FontSize::Sm)),
        theme::mix(fill, Color32::BLACK, 0.55),
    );
}
