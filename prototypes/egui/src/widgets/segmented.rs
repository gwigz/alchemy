//! A shadcn-style segmented control: a rounded track with a raised, highlighted active segment.
//! Sizes to its labels and writes the chosen value.

use eframe::egui::{self, Align2, FontId, Sense, Stroke, StrokeKind};
use twill::tokens::{BorderWidth, FontSize, Spacing};

use crate::theme;

pub fn bar<T: PartialEq + Copy>(ui: &mut egui::Ui, selected: &mut T, items: &[(T, &str)]) {
    let p = theme::active(ui.ctx());
    let font = FontId::proportional(theme::font(FontSize::Base));
    let border = theme::border_width(BorderWidth::S1);
    let pad = theme::space(Spacing::S0_5);
    let seg_pad_x = theme::field_pad_x();
    let height = theme::space(Spacing::S6);

    let mut label_w = 0.0_f32;
    for (_, label) in items {
        let galley = ui
            .painter()
            .layout_no_wrap((*label).to_owned(), font.clone(), p.foreground);
        label_w = label_w.max(galley.size().x);
    }
    let seg_w = (label_w + 2.0 * seg_pad_x).ceil();

    #[allow(clippy::cast_precision_loss)]
    let count = items.len() as f32;
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(seg_w * count + 2.0 * pad, height),
        Sense::hover(),
    );

    let track = theme::corner(ui.ctx());
    ui.painter()
        .rect_filled(rect, track, theme::mix(p.secondary, p.background, 0.5));

    let seg_corner = egui::CornerRadius::same(theme::round_u8(
        (f32::from(track.nw) - pad).max(0.0),
    ));
    let mut x = rect.left() + pad;
    for (value, label) in items {
        let seg = egui::Rect::from_min_size(
            egui::pos2(x, rect.top() + pad),
            egui::vec2(seg_w, height - 2.0 * pad),
        );
        x += seg_w;

        let resp = ui.interact(seg, ui.id().with(("seg", *label)), Sense::click());
        if resp.clicked() {
            *selected = *value;
        }
        let active = *selected == *value;

        if active {
            ui.painter().rect_filled(seg, seg_corner, p.secondary);
            ui.painter()
                .rect_stroke(seg, seg_corner, Stroke::new(border, p.border), StrokeKind::Inside);
        } else if resp.hovered() {
            ui.painter()
                .rect_filled(seg, seg_corner, theme::mix(p.secondary, p.foreground, 0.06));
        }

        let text_color = if active {
            p.foreground
        } else {
            p.muted_foreground
        };
        ui.painter()
            .text(seg.center(), Align2::CENTER_CENTER, *label, font.clone(), text_color);
    }
}
