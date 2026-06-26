use std::hash::Hash;

use eframe::egui;
use twill::tokens::{BorderWidth, FontSize, Spacing};

use crate::theme;
use crate::widgets::icon;

// Carves the arrow column off the right of `field_rect`, fills the background, and returns the
// (text, arrow) sub-rects. Shared by both the read-only and editable dropdowns.
fn begin_field(
    ui: &mut egui::Ui,
    field_rect: egui::Rect,
    fill: egui::Color32,
) -> (egui::Rect, egui::Rect) {
    let arrow_w = theme::space(Spacing::S6);

    let arrow_rect = egui::Rect::from_min_size(
        egui::pos2(field_rect.right() - arrow_w, field_rect.top()),
        egui::vec2(arrow_w, field_rect.height()),
    );
    let text_rect = egui::Rect::from_min_max(
        egui::pos2(field_rect.left() + theme::field_pad_x(), field_rect.top()),
        egui::pos2(arrow_rect.left(), field_rect.bottom()),
    );

    ui.painter()
        .rect_filled(field_rect, theme::active_radius(ui.ctx()).px(), fill);

    (text_rect, arrow_rect)
}

// Draws the caret (the same glyph the number spinner uses) and the field border. `ring` swaps the
// border for the focus ring.
fn finish_field(
    ui: &mut egui::Ui,
    field_rect: egui::Rect,
    arrow_rect: egui::Rect,
    arrow_col: egui::Color32,
    ring: bool,
) {
    let p = theme::active(ui.ctx());
    let border = theme::border_width(BorderWidth::S1);

    let glyph = egui::Rect::from_center_size(
        arrow_rect.center(),
        egui::vec2(theme::space(Spacing::S6), theme::font(FontSize::Sm)),
    );
    icon::paint(ui.painter(), glyph, icon::caret_down(), arrow_col);

    let stroke_col = if ring { p.ring } else { p.border };
    ui.painter().rect_stroke(
        field_rect,
        theme::active_radius(ui.ctx()).px(),
        egui::Stroke::new(border, stroke_col),
        egui::StrokeKind::Inside,
    );
}

// The dropdown popup frame plus the inner margin used to inset its min width.
fn popup_frame(ui: &egui::Ui) -> (egui::Frame, f32) {
    let margin = theme::space(Spacing::S1);
    let frame =
        egui::Frame::popup(ui.style()).inner_margin(egui::Margin::same(theme::round_i8(margin)));
    (frame, margin)
}

pub fn select<'a>(
    ui: &mut egui::Ui,
    id: &str,
    selected: &mut usize,
    items: impl Iterator<Item = &'a str> + Clone,
) {
    select_sized(
        ui,
        id,
        selected,
        items,
        egui::vec2(theme::field_w(), theme::field_h()),
    );
}

pub fn select_sized<'a>(
    ui: &mut egui::Ui,
    id: &str,
    selected: &mut usize,
    items: impl Iterator<Item = &'a str> + Clone,
    size: egui::Vec2,
) {
    let p = theme::active(ui.ctx());
    let id = ui.make_persistent_id(id);
    let popup_id = id.with("popup");

    let current = items.clone().nth(*selected).unwrap_or("").to_owned();

    let (field_rect, resp) = ui.allocate_exact_size(size, egui::Sense::click());

    let open = egui::Popup::is_id_open(ui.ctx(), popup_id);

    // Dropdowns read as buttons, not text inputs, so they use the secondary fill.
    let fill = if resp.hovered() || open {
        theme::mix(p.secondary, p.foreground, 0.12)
    } else {
        p.secondary
    };

    let (text_rect, arrow_rect) = begin_field(ui, field_rect, fill);

    ui.painter().with_clip_rect(text_rect).text(
        text_rect.left_center(),
        egui::Align2::LEFT_CENTER,
        current,
        egui::FontId::proportional(theme::font(FontSize::Base)),
        p.foreground,
    );

    let arrow_col = if resp.hovered() || open {
        p.foreground
    } else {
        p.muted_foreground
    };
    finish_field(ui, field_rect, arrow_rect, arrow_col, open);

    let (frame, margin) = popup_frame(ui);
    egui::Popup::from_toggle_button_response(&resp)
        .id(popup_id)
        .frame(frame)
        .layout(egui::Layout::top_down_justified(egui::Align::Min))
        .show(|ui| {
            ui.set_min_width(size.x - 2.0 * margin);

            for (index, label) in items.enumerate() {
                if ui.selectable_label(index == *selected, label).clicked() {
                    *selected = index;
                    ui.close();
                }
            }
        });
}

pub fn editable(
    ui: &mut egui::Ui,
    id_source: impl Hash,
    text: &mut String,
    hint: &str,
    options: &[(&str, &str)],
) {
    let p = theme::active(ui.ctx());
    let id = ui.make_persistent_id(id_source);

    let (field_rect, _) = ui.allocate_exact_size(
        egui::vec2(theme::field_w(), theme::field_h()),
        egui::Sense::hover(),
    );

    let focused = ui.memory(|m| m.has_focus(id.with("edit")));
    let fill = if focused { p.input_focus } else { p.input };

    let (text_rect, arrow_rect) = begin_field(ui, field_rect, fill);

    // Drive the typed text and the gamma-muted placeholder from the input color, then restore
    // so the arrow and popup keep the normal foreground.
    let prev_text_color = ui.visuals().override_text_color;
    ui.visuals_mut().override_text_color = Some(p.input_foreground);
    let text_resp = ui.put(
        text_rect,
        egui::TextEdit::singleline(text)
            .id(id.with("edit"))
            .frame(false)
            // `text_rect` already insets by `field_pad_x`; zero the default margin to avoid double padding.
            .margin(egui::Margin::ZERO)
            .vertical_align(egui::Align::Center)
            .hint_text(hint),
    );
    ui.visuals_mut().override_text_color = prev_text_color;

    let arrow_resp = ui.interact(arrow_rect, id.with("arrow"), egui::Sense::click());

    // Track the input text color so the arrow stays visible on both dark and light fields.
    let arrow_col = if arrow_resp.hovered() {
        p.input_foreground
    } else {
        theme::mix(p.input_foreground, p.input, 0.45)
    };

    finish_field(ui, field_rect, arrow_rect, arrow_col, text_resp.has_focus());

    let (frame, margin) = popup_frame(ui);
    egui::Popup::from_toggle_button_response(&arrow_resp)
        .id(id.with("popup"))
        .anchor(field_rect)
        .frame(frame)
        .layout(egui::Layout::top_down_justified(egui::Align::Min))
        .show(|ui| {
            ui.set_min_width(theme::field_w() - 2.0 * margin);

            for (display, value) in options {
                if ui.selectable_label(false, *display).clicked() {
                    (*value).clone_into(text);
                }
            }
        });
}
