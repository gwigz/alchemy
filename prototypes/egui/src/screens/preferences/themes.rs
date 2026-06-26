//! The Themes preferences panel (Alchemy/mock-specific: skin, corner radius, fonts).

use eframe::egui::{self, Align2, Color32, FontId, RichText, Sense};
use twill::tokens::{BorderWidth, FontSize, Spacing};

use super::common::{compact_h, hl_label, PanelCtx};
use crate::context::{self, Command, Services};
use crate::{theme, widgets};

// Themes panel labels. Mock-only (no SL XUI source to harvest), so these are plain English literals
// used both for rendering and for search; they stay English in every locale.
const THEME_LABEL_COLOR: &str = "Color theme:";
const THEME_LABEL_RADIUS: &str = "Corner radius:";
const THEME_LABEL_UI_FONT: &str = "Interface font:";
const THEME_LABEL_MONO_FONT: &str = "Monospace font:";

pub(super) const SEARCH_LITERALS: &[&str] = &[
    THEME_LABEL_COLOR,
    THEME_LABEL_RADIUS,
    THEME_LABEL_UI_FONT,
    THEME_LABEL_MONO_FONT,
];

#[derive(Clone, Copy, PartialEq, Eq, Default)]
enum ThemeFilter {
    #[default]
    All,
    Dark,
    Light,
}

#[derive(Default)]
pub(super) struct State {
    query: String,
    filter: ThemeFilter,
}

pub(super) fn show(ui: &mut egui::Ui, state: &mut State, cx: &PanelCtx) {
    let services = cx.services;
    let q = cx.q;

    // The parent lays panels out horizontally; switch to top-down so our sections stack and
    // `available_height` is meaningful for sizing the theme list.
    ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
        ui.set_min_width(ui.available_width());
        hl_label(ui, THEME_LABEL_COLOR, q);
        // The list is a fixed-height scroll region; radius/fonts stay visible below it.
        color_theme_section(ui, state, services);
        ui.add_space(theme::space(Spacing::S2));
        theme_controls(ui, q, services);
    });
}

fn theme_controls(ui: &mut egui::Ui, q: &str, services: &Services) {
    let p = theme::active(ui.ctx());
    let gap = theme::space(Spacing::S2);

    hl_label(ui, THEME_LABEL_RADIUS, q);
    let radius_name = theme::radii()
        .iter()
        .find(|(_, r)| *r == services.radius)
        .map_or("", |(name, _)| *name);
    ui.horizontal(|ui| {
        if let Some(r) = radius_slider(ui, services.radius) {
            context::post(ui.ctx(), Command::SetRadius(r));
        }
        ui.add_space(gap);
        ui.label(
            RichText::new(radius_name)
                .color(p.muted_foreground)
                .size(theme::font(FontSize::Base)),
        );
    });
    ui.add_space(gap);

    hl_label(ui, THEME_LABEL_UI_FONT, q);
    font_combo(
        ui,
        "prefs_ui_font",
        theme::fonts::ui_fonts(),
        services.ui_font,
        Command::SetUiFont,
    );
    ui.add_space(gap);

    hl_label(ui, THEME_LABEL_MONO_FONT, q);
    font_combo(
        ui,
        "prefs_mono_font",
        theme::fonts::mono_fonts(),
        services.mono_font,
        Command::SetMonoFont,
    );
}

// Filter control + name filter, then a fixed-height bordered scroll box of theme rows by family.
fn color_theme_section(ui: &mut egui::Ui, state: &mut State, services: &Services) {
    let p = theme::active(ui.ctx());
    let border = theme::border_width(BorderWidth::S1);

    ui.horizontal(|ui| {
        widgets::segmented::bar(
            ui,
            &mut state.filter,
            &[
                (ThemeFilter::All, "All"),
                (ThemeFilter::Dark, "Dark"),
                (ThemeFilter::Light, "Light"),
            ],
        );
        ui.add_space(theme::space(Spacing::S1));
        widgets::field::text_full_sized(ui, &mut state.query, "Filter themes", compact_h());
    });
    ui.add_space(theme::space(Spacing::S1));

    let needle = state.query.trim().to_lowercase();
    let filter = state.filter;
    let active_theme = services.theme;
    // A short, fixed-height box (the controls below it always stay on screen). It scrolls past ~6.
    let row_h = compact_h() + theme::space(Spacing::S1);
    let pad = theme::space(Spacing::S1_5);
    let (list_rect, _) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), row_h * 6.0),
        egui::Sense::hover(),
    );
    let corner = theme::corner(ui.ctx());
    ui.painter()
        .rect_filled(list_rect, corner, theme::mix(p.background, Color32::BLACK, 0.12));
    ui.painter().rect_stroke(
        list_rect,
        corner,
        egui::Stroke::new(border, p.border),
        egui::StrokeKind::Inside,
    );
    // Pad top/left/right; keep the bottom near-flush (just clear the border) so content fills the
    // box without clipping short, but doesn't draw over the bottom border.
    let inner = egui::Rect::from_min_max(
        list_rect.min + egui::vec2(pad, pad),
        egui::pos2(list_rect.max.x - pad, list_rect.max.y - border),
    );
    let mut list_ui = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(inner)
            .layout(egui::Layout::top_down(egui::Align::Min)),
    );
    list_ui.set_clip_rect(inner);
    egui::ScrollArea::vertical()
        .id_salt("prefs_theme_list")
        .auto_shrink([false, false])
        .show(&mut list_ui, |ui| {
            ui.set_min_width(ui.available_width());
            let mut last_group = "";
            for (name, t) in theme::themes() {
                if !theme_filter_matches(filter, t.dark()) {
                    continue;
                }
                if !needle.is_empty() && !name.to_lowercase().contains(&needle) {
                    continue;
                }
                if t.group() != last_group {
                    last_group = t.group();
                    theme_group_header(ui, last_group);
                }
                if theme_row(ui, name, *t, *t == active_theme) && *t != active_theme {
                    context::post(ui.ctx(), Command::SetTheme(*t));
                }
            }
        });
}

fn theme_filter_matches(filter: ThemeFilter, dark: bool) -> bool {
    match filter {
        ThemeFilter::All => true,
        ThemeFilter::Dark => dark,
        ThemeFilter::Light => !dark,
    }
}

fn theme_group_header(ui: &mut egui::Ui, name: &str) {
    let p = theme::active(ui.ctx());
    ui.add_space(theme::space(Spacing::S0_5));
    ui.label(
        RichText::new(name)
            .color(p.muted_foreground)
            .size(theme::font(FontSize::Sm)),
    );
}

// One theme as a compact row: an inline strip of palette swatches sampled from the theme (without
// installing it), the name, and an active dot. Returns whether it was clicked.
fn theme_row(ui: &mut egui::Ui, name: &str, theme_val: theme::Theme, active: bool) -> bool {
    let p = theme::active(ui.ctx());
    let pal = theme_val.palette();
    let pad = theme::space(Spacing::S1);
    let h = compact_h() + theme::space(Spacing::S1);

    let (rect, resp) = ui.allocate_exact_size(egui::vec2(ui.available_width(), h), Sense::click());
    let corner = theme::corner(ui.ctx());

    let fill = if active {
        theme::mix(p.background, p.primary, 0.2)
    } else if resp.hovered() {
        theme::mix(p.background, p.foreground, 0.1)
    } else {
        Color32::TRANSPARENT
    };

    let painter = ui.painter();
    painter.rect_filled(rect, corner, fill);

    let swatches = [
        pal.background,
        pal.primary,
        pal.accent,
        pal.secondary,
        pal.destructive,
    ];
    let sw = h - 2.0 * pad;
    let sgap = theme::space(Spacing::S0_5);
    let mut x = rect.left() + pad;
    for c in swatches {
        painter.rect_filled(
            egui::Rect::from_min_size(egui::pos2(x, rect.top() + pad), egui::vec2(sw, sw)),
            egui::CornerRadius::same(2),
            c,
        );
        x += sw + sgap;
    }

    painter.text(
        egui::pos2(x + pad, rect.center().y),
        Align2::LEFT_CENTER,
        name,
        FontId::proportional(theme::font(FontSize::Base)),
        p.foreground,
    );

    if active {
        painter.circle_filled(
            egui::pos2(rect.right() - pad - 3.0, rect.center().y),
            3.0,
            p.primary,
        );
    }

    resp.clicked()
}

fn font_combo<T: Copy + PartialEq>(
    ui: &mut egui::Ui,
    id: &str,
    options: &[(&'static str, T)],
    current: T,
    make: impl Fn(T) -> Command,
) {
    let mut idx = options.iter().position(|(_, v)| *v == current).unwrap_or(0);
    widgets::combo::select_sized(
        ui,
        id,
        &mut idx,
        options.iter().map(|(name, _)| *name),
        egui::vec2(theme::field_w(), compact_h()),
    );

    if let Some((_, v)) = options.get(idx) {
        if *v != current {
            context::post(ui.ctx(), make(*v));
        }
    }
}

// A discrete slider over the radius presets (None..Large). Drag/click snaps to the nearest stop;
// returns the new preset when it changes. Live preview: every corner in the app follows it.
#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
fn radius_slider(ui: &mut egui::Ui, current: theme::RadiusPref) -> Option<theme::RadiusPref> {
    let radii = theme::radii();
    let n = radii.len();
    let cur = radii.iter().position(|(_, r)| *r == current).unwrap_or(0);
    let p = theme::active(ui.ctx());

    let (rect, resp) = ui.allocate_exact_size(
        egui::vec2(theme::field_w(), compact_h()),
        egui::Sense::click_and_drag(),
    );

    let handle_r = rect.height() * 0.3;
    let x0 = rect.left() + handle_r;
    let x1 = rect.right() - handle_r;
    let cy = rect.center().y;
    let pos = |i: usize| x0 + (x1 - x0) * (i as f32 / (n - 1) as f32);

    let painter = ui.painter();
    let track = |a: f32, b: f32, col: Color32| {
        painter.rect_filled(
            egui::Rect::from_min_max(egui::pos2(a, cy - 1.5), egui::pos2(b, cy + 1.5)),
            egui::CornerRadius::same(2),
            col,
        );
    };
    track(x0, x1, p.secondary);
    track(x0, pos(cur), p.primary);
    for i in 0..n {
        painter.circle_filled(
            egui::pos2(pos(i), cy),
            2.0,
            theme::mix(p.secondary, p.foreground, 0.3),
        );
    }
    painter.circle_filled(egui::pos2(pos(cur), cy), handle_r, p.primary);
    painter.circle_stroke(
        egui::pos2(pos(cur), cy),
        handle_r,
        egui::Stroke::new(1.0, p.primary_foreground),
    );

    if (resp.clicked() || resp.dragged()) && x1 > x0 {
        if let Some(ptr) = resp.interact_pointer_pos() {
            let frac = ((ptr.x - x0) / (x1 - x0)).clamp(0.0, 1.0);
            let idx = (frac * (n - 1) as f32).round() as usize;
            if idx != cur {
                return radii.get(idx).map(|(_, r)| *r);
            }
        }
    }
    None
}
