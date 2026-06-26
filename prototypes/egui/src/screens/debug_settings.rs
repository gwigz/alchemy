//! Debug Settings floater, modeled on `floater_settings_debug.xml` (mock: edits aren't persisted).

use eframe::egui::{self, RichText};
use twill::tokens::{FontSize, Spacing};
use unic_langid::LanguageIdentifier;

use crate::context::{self, FloaterKind, Services};
use crate::data::{DebugSetting, SettingKind};
use crate::i18n::tr;
use crate::{floater, theme, widgets};

struct Edit {
    index: usize,
    value: String,
}

#[derive(Default)]
pub struct DebugSettingsState {
    pub open: bool,
    filter: String,
    changed_only: bool,
    selected: Option<usize>,
    edit: Option<Edit>,
    // Last frame's window rect, and the window rect captured when a top-edge resize drag begins.
    // Together they let us pin the bottom edge so the top edge stops at the min height.
    last_rect: Option<egui::Rect>,
    resize_top_anchor: Option<egui::Rect>,
}

impl context::Floater for DebugSettingsState {
    fn kind(&self) -> FloaterKind {
        FloaterKind::DebugSettings
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

        let settings = &services.model.debug_settings().settings;

        if self.selected.is_none() && !settings.is_empty() {
            self.selected = Some(0);
        }

        let mut open = true;
        let size = egui::vec2(640.0, 380.0);
        let min_size = egui::vec2(600.0, 320.0);
        let id = egui::Id::new(FloaterKind::DebugSettings.key());

        // Stop the top edge from sliding the whole window down once it reaches the min height.
        pin_top_resize(ctx, id, self, min_size.y);

        let inner = floater::window(
            ctx,
            FloaterKind::DebugSettings.key(),
            tr(services.lang, "debug-title"),
            floater::WindowOpts {
                default_size: size,
                min_size: Some(min_size),
                fixed_size: None,
                collapsible: true,
            },
            &mut open,
            |ui| contents(ui, self, settings, services.lang),
        );

        if let Some(inner) = inner {
            self.last_rect = Some(inner.response.rect);
        }

        self.open = open;
    }
}

// egui resizes a window's top edge by `cached_drag_start.min.y + total_drag_delta.y`, with no
// clamp at the min height, so once the window can't shrink further it slides the whole thing down
// the screen. We capture the window rect when a top-edge resize begins, then each frame rewrite
// egui's cached drag-start rect so the derived top can't pass `bottom - min_h`. That pins the
// bottom edge while leaving moves, bottom, and side resizes fully native.
fn pin_top_resize(
    ctx: &egui::Context,
    win_id: egui::Id,
    state: &mut DebugSettingsState,
    min_h: f32,
) {
    let (down, pressed, delta, origin) = ctx.input(|i| {
        (
            i.pointer.primary_down(),
            i.pointer.primary_pressed(),
            i.pointer.total_drag_delta(),
            i.pointer.press_origin(),
        )
    });

    if pressed {
        // A top-edge (or top-corner) resize starts with the press inside egui's grab band.
        state.resize_top_anchor = None;
        if let (Some(o), Some(rect)) = (origin, state.last_rect) {
            let band = ctx.style().interaction.resize_grab_radius_side + 2.0;
            let near_top = (o.y - rect.top()).abs() <= band
                && o.x >= rect.left() - band
                && o.x <= rect.right() + band;
            if near_top {
                state.resize_top_anchor = Some(rect);
            }
        }
    }

    if !down {
        state.resize_top_anchor = None;
    }

    if let (Some(anchor), Some(delta)) = (state.resize_top_anchor, delta) {
        let clamp_target = anchor.bottom() - min_h;
        let mut cached = anchor;
        if anchor.top() + delta.y > clamp_target {
            cached.min.y = clamp_target - delta.y;
        }
        // Mirrors egui's internal key: `area_id.with("resize").with("window_rect_at_drag_start")`.
        let key = win_id.with("resize").with("window_rect_at_drag_start");
        ctx.data_mut(|d| d.insert_temp(key, cached));
    }
}

fn contents(
    ui: &mut egui::Ui,
    state: &mut DebugSettingsState,
    settings: &[DebugSetting],
    lang: &LanguageIdentifier,
) {
    egui::TopBottomPanel::top("debug_search")
        .frame(egui::Frame::NONE)
        .show_separator_line(false)
        .show_inside(ui, |ui| {
            ui.add_space(theme::space(Spacing::S1));
            widgets::field::text_full(ui, &mut state.filter, &tr(lang, "debug-search"));
            ui.add_space(theme::space(Spacing::S1));
        });

    egui::TopBottomPanel::bottom("debug_changed")
        .frame(egui::Frame::NONE)
        .show_separator_line(false)
        .show_inside(ui, |ui| {
            ui.add_space(theme::space(Spacing::S1));
            ui.checkbox(&mut state.changed_only, tr(lang, "debug-changed-only"));
        });

    egui::CentralPanel::default()
        .frame(egui::Frame::NONE)
        .show_inside(ui, |ui| {
            let filter = state.filter.to_lowercase();
            let changed_only = state.changed_only;
            let matches = move |s: &DebugSetting| {
                (!changed_only || s.changed()) && s.name.to_lowercase().contains(&filter)
            };

            ui.horizontal_top(|ui| {
                list(ui, state, settings, &matches);
                ui.add_space(theme::space(Spacing::S2));
                detail(ui, state, settings, lang);
            });
        });
}

struct SettingsTable<'a> {
    settings: &'a [DebugSetting],
    rows: &'a [usize],
    selected: &'a mut Option<usize>,
    edit: &'a mut Option<Edit>,
    row_h: f32,
    pad_x: f32,
    font: egui::FontId,
    text: egui::Color32,
    muted: egui::Color32,
    primary: egui::Color32,
    selection_bg: egui::Color32,
    hover_bg: egui::Color32,
    header_bg: egui::Color32,
}

impl egui_table::TableDelegate for SettingsTable<'_> {
    fn header_cell_ui(&mut self, ui: &mut egui::Ui, cell: &egui_table::HeaderCellInfo) {
        let rect = ui.max_rect();
        ui.painter().rect_filled(rect, 0.0, self.header_bg);

        if cell.col_range.start >= 1 {
            ui.painter().text(
                rect.left_center() + egui::vec2(self.pad_x, 0.0),
                egui::Align2::LEFT_CENTER,
                "Setting",
                self.font.clone(),
                self.muted,
            );
        }
    }

    fn row_ui(&mut self, ui: &mut egui::Ui, row_nr: u64) {
        let idx = self.rows[usize::try_from(row_nr).unwrap_or(0)];
        let setting = &self.settings[idx];
        let rect = ui.max_rect();

        let resp = ui.interact(rect, ui.id().with(("row", idx)), egui::Sense::click());

        if *self.selected == Some(idx) {
            ui.painter().rect_filled(rect, 0.0, self.selection_bg);
        } else if resp.hovered() {
            ui.painter().rect_filled(rect, 0.0, self.hover_bg);
        }

        if resp.clicked() {
            *self.selected = Some(idx);
            *self.edit = Some(Edit {
                index: idx,
                value: setting.value.clone(),
            });
        }
    }

    fn cell_ui(&mut self, ui: &mut egui::Ui, cell: &egui_table::CellInfo) {
        let idx = self.rows[usize::try_from(cell.row_nr).unwrap_or(0)];
        let setting = &self.settings[idx];
        let rect = ui.max_rect();

        if cell.col_nr == 0 {
            if setting.changed() {
                ui.painter().text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "●",
                    self.font.clone(),
                    self.primary,
                );
            }
        } else {
            let color = if setting.changed() {
                self.primary
            } else {
                self.text
            };
            ui.painter().text(
                rect.left_center() + egui::vec2(self.pad_x, 0.0),
                egui::Align2::LEFT_CENTER,
                &setting.name,
                self.font.clone(),
                color,
            );
        }
    }

    fn default_row_height(&self) -> f32 {
        self.row_h
    }
}

fn list(
    ui: &mut egui::Ui,
    state: &mut DebugSettingsState,
    settings: &[DebugSetting],
    matches: &impl Fn(&DebugSetting) -> bool,
) {
    let p = theme::active(ui.ctx());
    let width = theme::space(Spacing::S64);
    let indicator_w = theme::space(Spacing::S6);
    let row_h = theme::space(Spacing::S6);

    let rows: Vec<usize> = settings
        .iter()
        .enumerate()
        .filter(|(_, s)| matches(s))
        .map(|(i, _)| i)
        .collect();

    let mut delegate = SettingsTable {
        settings,
        rows: &rows,
        selected: &mut state.selected,
        edit: &mut state.edit,
        row_h,
        pad_x: theme::field_pad_x(),
        font: egui::FontId::proportional(theme::font(FontSize::Base)),
        text: p.foreground,
        muted: p.muted_foreground,
        primary: p.primary,
        selection_bg: theme::mix(p.card, p.primary, 0.35),
        hover_bg: theme::mix(p.card, p.foreground, 0.06),
        header_bg: p.secondary,
    };

    theme::bordered_frame(ui.ctx()).show(ui, |ui| {
        ui.allocate_ui_with_layout(
            egui::vec2(width, ui.available_height()),
            egui::Layout::top_down(egui::Align::Min),
            |ui| {
                egui_table::Table::new()
                    .id_salt("debug_table")
                    .num_rows(rows.len() as u64)
                    .columns(vec![
                        egui_table::Column::new(indicator_w)
                            .range(egui::Rangef::new(indicator_w, indicator_w))
                            .resizable(false),
                        egui_table::Column::new(width - indicator_w).resizable(false),
                    ])
                    .headers(vec![egui_table::HeaderRow::new(row_h)])
                    .auto_size_mode(egui_table::AutoSizeMode::Always)
                    .show(ui, &mut delegate);
            },
        );
    });
}

fn detail(
    ui: &mut egui::Ui,
    state: &mut DebugSettingsState,
    settings: &[DebugSetting],
    lang: &LanguageIdentifier,
) {
    let p = theme::active(ui.ctx());

    let Some(index) = state.selected else {
        ui.label(RichText::new("Select a setting.").color(p.muted_foreground));
        return;
    };

    let setting = &settings[index];

    if state.edit.as_ref().map(|e| e.index) != Some(index) {
        state.edit = Some(Edit {
            index,
            value: setting.value.clone(),
        });
    }

    let edit = state.edit.as_mut().unwrap();

    ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
        ui.set_min_width(ui.available_width());

        ui.label(RichText::new(&setting.name).strong());
        ui.add_space(theme::space(Spacing::S1));

        description(ui, &setting.comment);

        ui.add_space(theme::space(Spacing::S2));
        editor(ui, setting.kind, &mut edit.value);

        ui.add_space(theme::space(Spacing::S2));
        if widgets::field::secondary_button(ui, &tr(lang, "debug-reset")).clicked() {
            setting.default.clone_into(&mut edit.value);
        }
    });
}

fn description(ui: &mut egui::Ui, text: &str) {
    // Read-only display; `multiline` never writes back, so a throwaway owned copy is fine.
    let mut text = text.to_owned();
    widgets::field::multiline(
        ui,
        "debug_desc",
        &mut text,
        widgets::field::MultilineOpts {
            rows: 0,
            readonly: true,
            scroll_height: Some(theme::space(Spacing::S28)),
        },
    );
}

use widgets::field::NumberOpts;

const S32_OPTS: NumberOpts = NumberOpts {
    step: 1.0,
    min: None,
    max: None,
    precision: 0,
};
const F32_OPTS: NumberOpts = NumberOpts {
    step: 0.1,
    min: None,
    max: None,
    precision: 3,
};
const VEC3_OPTS: NumberOpts = NumberOpts {
    step: 0.1,
    min: None,
    max: None,
    precision: 3,
};
const COLOR_OPTS: NumberOpts = NumberOpts {
    step: 0.1,
    min: Some(0.0),
    max: Some(1.0),
    precision: 3,
};

fn editor(ui: &mut egui::Ui, kind: SettingKind, value: &mut String) {
    match kind {
        SettingKind::Boolean => {
            let mut on = value == "true";
            ui.vertical(|ui| {
                ui.radio_value(&mut on, true, "TRUE");
                ui.radio_value(&mut on, false, "FALSE");
            });
            *value = on.to_string();
        }
        SettingKind::Color | SettingKind::Vec3 => {
            let opts = if matches!(kind, SettingKind::Color) {
                COLOR_OPTS
            } else {
                VEC3_OPTS
            };
            let mut parts: Vec<String> = value.split_whitespace().map(str::to_owned).collect();

            let size = egui::vec2(theme::space(Spacing::S16), theme::field_h());
            ui.horizontal(|ui| {
                for (i, part) in parts.iter_mut().enumerate() {
                    widgets::field::number_spinner(ui, i, part, size, opts);
                }
            });

            *value = parts.join(" ");
        }
        SettingKind::S32 | SettingKind::F32 => {
            let opts = if matches!(kind, SettingKind::S32) {
                S32_OPTS
            } else {
                F32_OPTS
            };
            let size = egui::vec2(theme::space(Spacing::S28), theme::field_h());
            widgets::field::number_spinner(ui, 0u8, value, size, opts);
        }
        SettingKind::String => {
            widgets::field::text_full(ui, value, "");
        }
    }
}
