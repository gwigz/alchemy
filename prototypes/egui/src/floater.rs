//! Floater window chrome. The registry/command wiring lives in `context`.

use eframe::egui::{self, RichText};
use twill::tokens::FontSize;

use crate::theme;

#[derive(Clone, Copy)]
pub struct WindowOpts {
    pub default_size: egui::Vec2,
    // Resizable with this floor when Some; ignored when `fixed_size` is set.
    pub min_size: Option<egui::Vec2>,
    // Non-resizable at this size when Some (e.g. the About floater).
    pub fixed_size: Option<egui::Vec2>,
    pub collapsible: bool,
}

// Shared floater chrome: derives the window id, tracks active/inactive state, and keeps the
// frame and content opacity in sync so a backgrounded floater dims consistently. Returns the
// window's inner response so callers can read its rect (e.g. the debug-settings resize pin).
pub fn window(
    ctx: &egui::Context,
    id_str: &str,
    title: impl Into<String>,
    opts: WindowOpts,
    open: &mut bool,
    add: impl FnOnce(&mut egui::Ui),
) -> Option<egui::InnerResponse<Option<()>>> {
    let id = egui::Id::new(id_str);
    let inactive = theme::window_inactive(ctx, id);
    let anchor = opts.fixed_size.unwrap_or(opts.default_size);

    let title = RichText::new(title.into()).size(theme::font(FontSize::Base));

    let mut win = egui::Window::new(title)
        .id(id)
        .open(open)
        .collapsible(opts.collapsible)
        .default_pos(ctx.content_rect().center() - anchor * 0.5)
        .frame(theme::window_frame(ctx, inactive));

    if let Some(fixed) = opts.fixed_size {
        win = win.resizable(false).fixed_size(fixed);
    } else {
        win = win.resizable(true).default_size(opts.default_size);
        if let Some(min) = opts.min_size {
            win = win.min_size(min);
        }
    }

    win.show(ctx, |ui| {
        ui.multiply_opacity(theme::window_content_opacity(inactive));
        add(ui);
    })
}
