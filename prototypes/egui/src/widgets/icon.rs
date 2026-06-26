//! Icons from iconflow, rendered from their own font families (registered in `theme::fonts`).

use eframe::egui::text::{LayoutJob, TextFormat};
use eframe::egui::{self, Align, Color32, FontFamily, FontId, Painter, Rect};
use iconflow::packs::phosphor;
use iconflow::{IconRef, Size, Style};

pub fn copy() -> IconRef {
    phosphor::Icon::Copy.icon(Style::Regular, Size::Regular)
}

pub fn caret_up() -> IconRef {
    phosphor::Icon::CaretUp.icon(Style::Regular, Size::Regular)
}

pub fn caret_down() -> IconRef {
    phosphor::Icon::CaretDown.icon(Style::Regular, Size::Regular)
}

// Icon and label live in different font families, so a `LayoutJob` is needed to mix them.
pub fn labeled(icon: IconRef, text: &str, size: f32, color: Color32) -> LayoutJob {
    let mut job = LayoutJob::default();

    if let Some(glyph) = char::from_u32(icon.codepoint) {
        job.append(
            glyph.encode_utf8(&mut [0u8; 4]),
            0.0,
            TextFormat {
                font_id: FontId::new(size, FontFamily::Name(icon.family.into())),
                color,
                valign: Align::Center,
                ..Default::default()
            },
        );
    }

    job.append(
        &format!("  {text}"),
        0.0,
        TextFormat {
            font_id: FontId::new(size, FontFamily::Proportional),
            color,
            valign: Align::Center,
            ..Default::default()
        },
    );

    job
}

#[allow(dead_code)]
pub fn paint(painter: &Painter, rect: Rect, icon: IconRef, color: Color32) {
    let Some(glyph) = char::from_u32(icon.codepoint) else {
        return;
    };

    painter.text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        glyph,
        FontId::new(rect.height(), FontFamily::Name(icon.family.into())),
        color,
    );
}
