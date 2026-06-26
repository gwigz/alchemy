use eframe::egui::{self, Align2, CornerRadius, FontId, Sense, Stroke, StrokeKind};
use twill::tokens::{BorderWidth, FontSize, Spacing};

use crate::theme;

pub fn bar<T: PartialEq + Copy>(ui: &mut egui::Ui, selected: &mut T, tabs: &[(T, &str)]) {
    let p = theme::active(ui.ctx());
    let radius = theme::corner(ui.ctx()).nw;
    let border = theme::border_width(BorderWidth::S1);
    let height = theme::field_h();
    let gap = theme::space(Spacing::S0_5);
    let font = FontId::proportional(theme::font(FontSize::Base));

    #[allow(clippy::cast_precision_loss)]
    let count = tabs.len() as f32;
    let full_w = ui.available_width();
    let tab_w = ((full_w - gap * (count - 1.0)) / count).floor();

    let (strip_rect, _) = ui.allocate_exact_size(egui::vec2(full_w, height), Sense::hover());
    let strip = ui.interact(
        strip_rect,
        ui.id().with("tab_strip"),
        Sense::click_and_drag(),
    );

    // Press-and-drag across the strip switches tabs live (and a plain click selects). Sensing the
    // drag here also stops it falling through to the parent window's move interaction.
    if !tabs.is_empty() && strip.is_pointer_button_down_on() {
        if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
            let rel = (pos.x - strip_rect.left()).clamp(0.0, strip_rect.width());
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let idx = ((rel / (tab_w + gap)) as usize).min(tabs.len() - 1);
            *selected = tabs[idx].0;
        }
    }

    let baseline_y = strip_rect.bottom();
    ui.painter().hline(
        strip_rect.x_range(),
        baseline_y,
        Stroke::new(border, p.border),
    );

    let corner = CornerRadius {
        nw: radius,
        ne: radius,
        sw: 0,
        se: 0,
    };

    let mut x = strip_rect.left();
    for (value, label) in tabs {
        let rect =
            egui::Rect::from_min_size(egui::pos2(x, strip_rect.top()), egui::vec2(tab_w, height));
        x += tab_w + gap;

        let active = *selected == *value;
        let hovered = ui.rect_contains_pointer(rect);

        let fill = if active {
            theme::mix(p.secondary, p.primary, 0.5)
        } else if hovered {
            theme::mix(p.secondary, p.foreground, 0.08)
        } else {
            p.secondary
        };
        let text_color = if active {
            p.foreground
        } else {
            p.muted_foreground
        };

        ui.painter().rect_filled(rect, corner, fill);
        if !active {
            ui.painter().rect_stroke(
                rect,
                corner,
                Stroke::new(border, p.border),
                StrokeKind::Inside,
            );
        }
        ui.painter().text(
            rect.center(),
            Align2::CENTER_CENTER,
            *label,
            font.clone(),
            text_color,
        );
    }
}

pub fn column<T: PartialEq + Copy>(
    ui: &mut egui::Ui,
    selected: &mut T,
    width: f32,
    tabs: &[(T, &str)],
) {
    let p = theme::active(ui.ctx());
    let radius = theme::corner(ui.ctx()).nw;
    let border = theme::border_width(BorderWidth::S1);
    let row_h = theme::field_h();
    let pad_x = theme::field_pad_x();
    let font = FontId::proportional(theme::font(FontSize::Base));

    #[allow(clippy::cast_precision_loss)]
    let count = tabs.len() as f32;
    let (strip_rect, _) = ui.allocate_exact_size(egui::vec2(width, row_h * count), Sense::hover());
    let strip = ui.interact(
        strip_rect,
        ui.id().with("vtab_strip"),
        Sense::click_and_drag(),
    );

    // Press-and-drag down the strip switches tabs live (and a plain click selects). Sensing the
    // drag here also stops it falling through to the parent window's move interaction.
    if !tabs.is_empty() && strip.is_pointer_button_down_on() {
        if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
            let rel = (pos.y - strip_rect.top()).clamp(0.0, strip_rect.height());
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let idx = ((rel / row_h) as usize).min(tabs.len() - 1);
            *selected = tabs[idx].0;
        }
    }

    ui.painter().vline(
        strip_rect.right(),
        strip_rect.y_range(),
        Stroke::new(border, p.border),
    );

    let corner = CornerRadius {
        nw: radius,
        sw: radius,
        ne: 0,
        se: 0,
    };

    let mut y = strip_rect.top();
    for (value, label) in tabs {
        let rect =
            egui::Rect::from_min_size(egui::pos2(strip_rect.left(), y), egui::vec2(width, row_h));
        y += row_h;

        let active = *selected == *value;
        let hovered = ui.rect_contains_pointer(rect);

        let fill = if active {
            theme::mix(p.secondary, p.primary, 0.5)
        } else if hovered {
            theme::mix(p.secondary, p.foreground, 0.08)
        } else {
            p.secondary
        };
        let text_color = if active {
            p.foreground
        } else {
            p.muted_foreground
        };

        ui.painter().rect_filled(rect, corner, fill);
        if !active {
            ui.painter().rect_stroke(
                rect,
                corner,
                Stroke::new(border, p.border),
                StrokeKind::Inside,
            );
        }
        ui.painter().text(
            rect.left_center() + egui::vec2(pad_x, 0.0),
            Align2::LEFT_CENTER,
            *label,
            font.clone(),
            text_color,
        );
    }
}
