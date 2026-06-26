use std::hash::Hash;

use eframe::egui::{self, RichText};
use twill::tokens::{BorderWidth, FontSize, Spacing};

use crate::theme;
use crate::widgets::icon;

pub fn label(ui: &mut egui::Ui, text: &str) {
    ui.label(
        RichText::new(text)
            .color(theme::active(ui.ctx()).muted_foreground)
            .size(theme::font(FontSize::Base)),
    );

    ui.add_space(theme::space(Spacing::S0_5));
}

fn field<'a>(
    ui: &mut egui::Ui,
    value: &'a mut String,
    hint: &str,
    size: egui::Vec2,
    configure: impl FnOnce(egui::TextEdit<'a>) -> egui::TextEdit<'a>,
) -> egui::Response {
    let p = theme::active(ui.ctx());

    // Reserve the id so we can read last frame's focus state before drawing.
    let id = ui.next_auto_id();
    ui.skip_ahead_auto_ids(1);
    let focused = ui.memory(|m| m.has_focus(id));

    let margin = egui::Margin::symmetric(
        theme::round_i8(theme::field_pad_x()),
        theme::round_i8(theme::space(Spacing::S0_5)),
    );

    let edit = egui::TextEdit::singleline(value)
        .id(id)
        .vertical_align(egui::Align::Center)
        .margin(margin)
        .hint_text(hint.to_owned())
        .background_color(if focused { p.input_focus } else { p.input });

    // Drive both the typed text and the gamma-muted placeholder from the field's own input
    // color (egui derives the hint from `override_text_color`), so the hint tone matches the
    // eventual text on every skin, notably the light default field.
    ui.scope(|ui| {
        ui.visuals_mut().override_text_color = Some(p.input_foreground);
        // Placeholder = text color gamma-multiplied by weak_text_alpha (premultiplied, so this
        // reads as opacity). Keep it well below the 0.6 default so the hint stays faint.
        ui.visuals_mut().weak_text_alpha = 0.3;
        ui.add_sized(size, configure(edit))
    })
    .inner
}

#[allow(dead_code)]
pub fn text(ui: &mut egui::Ui, value: &mut String, hint: &str) -> egui::Response {
    field(
        ui,
        value,
        hint,
        egui::vec2(theme::field_w(), theme::field_h()),
        |edit| edit,
    )
}

pub fn text_full(ui: &mut egui::Ui, value: &mut String, hint: &str) -> egui::Response {
    let size = egui::vec2(ui.available_width(), theme::field_h());
    field(ui, value, hint, size, |edit| edit)
}

pub fn text_full_sized(
    ui: &mut egui::Ui,
    value: &mut String,
    hint: &str,
    height: f32,
) -> egui::Response {
    let size = egui::vec2(ui.available_width(), height);
    field(ui, value, hint, size, |edit| edit)
}

pub fn password(ui: &mut egui::Ui, value: &mut String, hint: &str) -> egui::Response {
    field(
        ui,
        value,
        hint,
        egui::vec2(theme::field_w(), theme::field_h()),
        |edit| edit.password(true),
    )
}

#[allow(dead_code)]
pub fn number(ui: &mut egui::Ui, value: &mut String, width: f32) -> egui::Response {
    field(ui, value, "", egui::vec2(width, theme::field_h()), |edit| {
        edit.horizontal_align(egui::Align::Max)
    })
}

#[derive(Clone, Copy)]
pub struct NumberOpts {
    pub step: f64,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub precision: usize,
}

const HOLD_DELAY: f64 = 0.4;
const HOLD_INTERVAL: f64 = 0.05;

fn apply_step(value: &mut String, delta: f64, opts: NumberOpts) {
    let Ok(cur) = value.trim().parse::<f64>() else {
        return;
    };

    let mut v = cur + delta;

    if let Some(min) = opts.min {
        v = v.max(min);
    }

    if let Some(max) = opts.max {
        v = v.min(max);
    }

    *value = format!("{:.*}", opts.precision, v);
}

fn step_scale(ui: &egui::Ui) -> f64 {
    ui.input(|i| {
        if i.modifiers.command {
            0.1
        } else if i.modifiers.shift {
            10.0
        } else {
            1.0
        }
    })
}

fn held_fire(ui: &egui::Ui, resp: &egui::Response, key: egui::Id) -> bool {
    if !resp.is_pointer_button_down_on() {
        ui.data_mut(|d| d.remove::<f64>(key));
        return false;
    }

    let now = ui.input(|i| i.time);
    let fire = match ui.data(|d| d.get_temp::<f64>(key)) {
        None => {
            ui.data_mut(|d| d.insert_temp(key, now + HOLD_DELAY));
            true
        }
        Some(next) if now >= next => {
            ui.data_mut(|d| d.insert_temp(key, now + HOLD_INTERVAL));
            true
        }
        _ => false,
    };

    ui.ctx().request_repaint();
    fire
}

#[allow(clippy::cast_precision_loss)]
fn arrow_key_delta(ui: &mut egui::Ui, step: f64) -> f64 {
    ui.input_mut(|i| {
        let net = |m: egui::Modifiers, i: &mut egui::InputState| {
            let up = i.count_and_consume_key(m, egui::Key::ArrowUp);
            let down = i.count_and_consume_key(m, egui::Key::ArrowDown);
            i64::try_from(up).unwrap_or(0) - i64::try_from(down).unwrap_or(0)
        };

        let base = net(egui::Modifiers::NONE, i);
        let coarse = net(egui::Modifiers::SHIFT, i);
        let fine = net(egui::Modifiers::COMMAND, i);
        (base as f64 + coarse as f64 * 10.0 + fine as f64 * 0.1) * step
    })
}

/// A right-aligned numeric field with an up/down spinner column. Steps by
/// `opts.step` (Shift x10, Ctrl/Cmd /10) via the arrows, Up/Down keys (when
/// focused), the mouse wheel (when hovered), and press-and-hold repeat.
pub fn number_spinner(
    ui: &mut egui::Ui,
    id_source: impl std::hash::Hash,
    value: &mut String,
    size: egui::Vec2,
    opts: NumberOpts,
) -> egui::Response {
    let p = theme::active(ui.ctx());
    let id = ui.make_persistent_id(id_source);
    let radius = theme::active_radius(ui.ctx()).px();
    let border = theme::border_width(BorderWidth::S1);
    let arrow_w = theme::space(Spacing::S4);

    let (field_rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());

    let arrow_rect = egui::Rect::from_min_size(
        egui::pos2(field_rect.right() - arrow_w, field_rect.top()),
        egui::vec2(arrow_w, field_rect.height()),
    );

    let mid = arrow_rect.center().y;
    let up_rect = egui::Rect::from_min_max(arrow_rect.min, egui::pos2(arrow_rect.max.x, mid));
    let down_rect = egui::Rect::from_min_max(egui::pos2(arrow_rect.min.x, mid), arrow_rect.max);

    let text_rect = egui::Rect::from_min_max(
        egui::pos2(field_rect.left() + theme::field_pad_x(), field_rect.top()),
        egui::pos2(
            arrow_rect.left() - theme::field_pad_x(),
            field_rect.bottom(),
        ),
    );

    let focused = ui.memory(|m| m.has_focus(id.with("edit")));
    ui.painter().rect_filled(
        field_rect,
        radius,
        if focused { p.input_focus } else { p.input },
    );

    let text_resp = ui.put(
        text_rect,
        egui::TextEdit::singleline(value)
            .id(id.with("edit"))
            .frame(false)
            .margin(egui::Margin::ZERO)
            .vertical_align(egui::Align::Center)
            .horizontal_align(egui::Align::Max)
            .text_color(p.input_foreground),
    );

    let up_resp = ui.interact(up_rect, id.with("up"), egui::Sense::click());
    let down_resp = ui.interact(down_rect, id.with("down"), egui::Sense::click());

    let scale = step_scale(ui);

    if held_fire(ui, &up_resp, id.with("up_t")) {
        apply_step(value, opts.step * scale, opts);
    }

    if held_fire(ui, &down_resp, id.with("down_t")) {
        apply_step(value, -opts.step * scale, opts);
    }

    if text_resp.has_focus() {
        let delta = arrow_key_delta(ui, opts.step);
        if delta != 0.0 {
            apply_step(value, delta, opts);
        }
    }

    if ui.rect_contains_pointer(field_rect) {
        let scroll = ui.input(|i| i.smooth_scroll_delta.y);
        if scroll.abs() > 0.5 {
            apply_step(value, f64::from(scroll.signum()) * opts.step * scale, opts);
        }
    }

    let divider = theme::mix(p.input_foreground, p.input, 0.65);
    ui.painter().vline(
        arrow_rect.left(),
        egui::Rangef::new(field_rect.top() + border, field_rect.bottom() - border),
        egui::Stroke::new(border, divider),
    );

    let glyph = egui::vec2(arrow_w, theme::font(FontSize::Sm));
    let muted = theme::mix(p.input_foreground, p.input, 0.45);
    let up_col = if up_resp.hovered() {
        p.input_foreground
    } else {
        muted
    };
    let down_col = if down_resp.hovered() {
        p.input_foreground
    } else {
        muted
    };

    icon::paint(
        ui.painter(),
        egui::Rect::from_center_size(up_rect.center(), glyph),
        icon::caret_up(),
        up_col,
    );
    icon::paint(
        ui.painter(),
        egui::Rect::from_center_size(down_rect.center(), glyph),
        icon::caret_down(),
        down_col,
    );

    let stroke = if text_resp.has_focus() {
        egui::Stroke::new(border, p.ring)
    } else {
        egui::Stroke::new(border, p.border)
    };
    ui.painter()
        .rect_stroke(field_rect, radius, stroke, egui::StrokeKind::Inside);

    // Re-assert the field; `ui.put` rewound the cursor, which would overlap the next spinner.
    ui.allocate_rect(field_rect, egui::Sense::hover());

    text_resp.union(up_resp).union(down_resp)
}

#[derive(Clone, Copy)]
pub struct MultilineOpts {
    pub rows: usize,
    pub readonly: bool,
    // Some(h) wraps the body in a fixed-height vertical scroll; None lets it size to content.
    pub scroll_height: Option<f32>,
}

pub fn multiline(
    ui: &mut egui::Ui,
    id_source: impl Hash,
    value: &mut String,
    opts: MultilineOpts,
) -> egui::Response {
    let p = theme::active(ui.ctx());
    let id = ui.make_persistent_id(id_source);

    let focused = !opts.readonly && ui.memory(|m| m.has_focus(id));

    let fill = if opts.readonly {
        p.input_readonly
    } else if focused {
        p.input_focus
    } else {
        p.input
    };
    let stroke_col = if focused { p.ring } else { p.border };

    // The read-only label has no margin of its own, so the frame insets it. The editable branch
    // zeroes the frame and pads via the `TextEdit` margin instead (avoids stacking egui's default
    // multiline margin on top), so its insets can be tuned independently.
    let frame_margin = if opts.readonly {
        egui::Margin::symmetric(
            theme::round_i8(theme::field_pad_x()),
            theme::round_i8(theme::space(Spacing::S0_5)),
        )
    } else {
        egui::Margin::ZERO
    };

    theme::bordered_frame(ui.ctx())
        .fill(fill)
        .stroke(egui::Stroke::new(
            theme::border_width(BorderWidth::S1),
            stroke_col,
        ))
        .inner_margin(frame_margin)
        .show(ui, |ui| {
            if opts.readonly {
                let label = RichText::new(value.as_str()).color(p.muted_foreground);
                if let Some(height) = opts.scroll_height {
                    egui::ScrollArea::vertical()
                        .id_salt(id)
                        .auto_shrink([false, false])
                        .max_height(height)
                        .show(ui, |ui| {
                            ui.set_min_height(height);
                            ui.label(label);
                        });
                } else {
                    ui.label(label);
                }
            } else {
                // Even inset on all sides (the frame adds none here).
                let inset = theme::space(Spacing::S1_5);
                let edit = egui::TextEdit::multiline(value)
                    .id(id)
                    .frame(false)
                    .margin(egui::Margin::same(theme::round_i8(inset)))
                    .desired_rows(opts.rows)
                    .desired_width(f32::INFINITY)
                    .text_color(p.input_foreground);

                // Some(h) pins the *text* height so the field doesn't grow with content. A bare
                // `max_height` can't cap a multiline TextEdit (and nesting inside another scroll
                // area hands it unbounded height), so reserve an exact rect, clip a child ui to it,
                // and run a non-shrinking scroll area inside that bounded space. The reserved rect
                // adds the vertical inset on top of `height` so the requested rows show in full.
                if let Some(height) = opts.scroll_height {
                    let (rect, _) = ui.allocate_exact_size(
                        egui::vec2(ui.available_width(), height + 2.0 * inset),
                        egui::Sense::hover(),
                    );
                    let mut child = ui.new_child(
                        egui::UiBuilder::new()
                            .max_rect(rect)
                            .layout(egui::Layout::top_down(egui::Align::Min)),
                    );
                    child.set_clip_rect(rect);
                    egui::ScrollArea::vertical()
                        .id_salt(id.with("scroll"))
                        .auto_shrink([false, false])
                        .show(&mut child, |ui| {
                            ui.add(edit);
                        });
                } else {
                    ui.add(edit);
                }
            }
        })
        .response
}

#[derive(Clone, Copy)]
pub enum ButtonVariant {
    Primary,
    Secondary,
}

impl ButtonVariant {
    // (fill, foreground) for this variant.
    fn colors(self, p: &theme::Palette) -> (egui::Color32, egui::Color32) {
        match self {
            Self::Primary => (p.primary, p.primary_foreground),
            Self::Secondary => (p.secondary, p.secondary_foreground),
        }
    }
}

pub fn button(
    ui: &mut egui::Ui,
    variant: ButtonVariant,
    size: egui::Vec2,
    text: &str,
) -> egui::Response {
    let (fill, fg) = variant.colors(&theme::active(ui.ctx()));
    let button = egui::Button::new(RichText::new(text).strong().color(fg)).fill(fill);
    ui.add_sized(size, button)
}

// `button` with a pre-built (already-colored) label, e.g. an icon + text `LayoutJob`.
pub fn button_labeled(
    ui: &mut egui::Ui,
    variant: ButtonVariant,
    size: egui::Vec2,
    label: impl Into<egui::WidgetText>,
) -> egui::Response {
    let (fill, _) = variant.colors(&theme::active(ui.ctx()));
    ui.add_sized(size, egui::Button::new(label).fill(fill))
}

pub fn primary_button(ui: &mut egui::Ui, text: &str) -> egui::Response {
    button(
        ui,
        ButtonVariant::Primary,
        egui::vec2(theme::field_w(), theme::field_h()),
        text,
    )
}

pub fn secondary_button(ui: &mut egui::Ui, text: &str) -> egui::Response {
    button(
        ui,
        ButtonVariant::Secondary,
        egui::vec2(theme::field_w(), theme::field_h()),
        text,
    )
}
