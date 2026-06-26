//! egui doesn't hover-switch top-level menus once open, so open state is driven here.

use eframe::egui;
use twill::tokens::Spacing;

use crate::theme;
use crate::widgets::hotkey;

pub struct Menu<'a> {
    label: String,
    content: Box<dyn FnMut(&mut egui::Ui) + 'a>,
}

impl<'a> Menu<'a> {
    pub fn new(label: impl Into<String>, content: impl FnMut(&mut egui::Ui) + 'a) -> Self {
        Self {
            label: label.into(),
            content: Box::new(content),
        }
    }
}

pub fn bar(ui: &mut egui::Ui, menus: &mut [Menu]) {
    egui::Frame::NONE
        .fill(egui::Color32::BLACK)
        .inner_margin(egui::Margin::symmetric(
            theme::round_i8(theme::field_pad_x()),
            0,
        ))
        .show(ui, |ui| {
            let corner = menu_corner_full(ui.ctx());
            egui::MenuBar::new()
                .style(menu_style(corner))
                .config(egui::containers::menu::MenuConfig::new().style(menu_style(corner)))
                .ui(ui, |ui| buttons(ui, menus));
        });
}

pub fn item(ui: &mut egui::Ui, label: &str, spec: &str) -> egui::Response {
    ui.add(egui::Button::new(label).shortcut_text(hotkey::mac(spec)))
}

pub fn radio_item(ui: &mut egui::Ui, label: &str, selected: bool) -> egui::Response {
    let prefix = if selected { "● " } else { "    " };
    item(ui, &format!("{prefix}{label}"), "")
}

pub fn link(ui: &mut egui::Ui, label: &str, url: &str) {
    if ui.button(label).clicked() {
        ui.ctx().open_url(egui::OpenUrl::new_tab(url));
        ui.close();
    }
}

fn buttons(ui: &mut egui::Ui, menus: &mut [Menu]) {
    let full = menu_corner_full(ui.ctx());
    let flat = menu_corner_flat(ui.ctx());

    let ids: Vec<egui::Id> = (0..menus.len())
        .map(|i| ui.make_persistent_id(("menu_bar", i)))
        .collect();

    let open: Vec<bool> = ids
        .iter()
        .map(|id| egui::Popup::is_id_open(ui.ctx(), *id))
        .collect();

    let bar_open = open.iter().any(|&o| o);

    let resps: Vec<egui::Response> = menus.iter().map(|m| ui.button(&m.label)).collect();

    let mut cmds: Vec<Option<egui::SetOpenCommand>> = resps
        .iter()
        .map(|r| r.clicked().then_some(egui::SetOpenCommand::Toggle))
        .collect();

    if bar_open {
        if let Some(hovered) = resps.iter().position(egui::Response::hovered) {
            if !open[hovered] {
                for (i, cmd) in cmds.iter_mut().enumerate() {
                    *cmd = Some(egui::SetOpenCommand::Bool(i == hovered));
                }
            }
        }
    }

    for (i, menu) in menus.iter_mut().enumerate() {
        // Top-level dropdowns hang flush off the bar (flat top); the propagated style keeps
        // submenus fully rounded.
        let frame = egui::Frame::menu(ui.style())
            .corner_radius(flat)
            .stroke(egui::Stroke::NONE);

        egui::Popup::menu(&resps[i])
            .id(ids[i])
            .style(menu_style(full))
            .frame(frame)
            .open_memory(cmds[i])
            .show(|ui| (menu.content)(ui));
    }
}

// Flush against the menu bar: square top, rounded bottom (top-level dropdowns).
fn menu_corner_flat(ctx: &egui::Context) -> egui::CornerRadius {
    let r = theme::corner(ctx).sw;
    egui::CornerRadius {
        nw: 0,
        ne: 0,
        sw: r,
        se: r,
    }
}

// Fully rounded; submenus pop out beside their parent, not off the bar.
fn menu_corner_full(ctx: &egui::Context) -> egui::CornerRadius {
    theme::corner(ctx)
}

fn menu_style(corner: egui::CornerRadius) -> impl Fn(&mut egui::Style) + Send + Sync + 'static {
    move |style| {
        style.spacing.button_padding = egui::vec2(theme::field_pad_x(), theme::space(Spacing::S1));
        style.spacing.interact_size.y = 0.0;
        style.visuals.menu_corner_radius = corner;
        style.visuals.window_stroke = egui::Stroke::NONE;
        // Items extend the menu width instead of wrapping onto a second line.
        style.wrap_mode = Some(egui::TextWrapMode::Extend);

        let widgets = &mut style.visuals.widgets;
        for state in [
            &mut widgets.inactive,
            &mut widgets.hovered,
            &mut widgets.active,
            &mut widgets.open,
        ] {
            state.bg_stroke = egui::Stroke::NONE;
            state.corner_radius = egui::CornerRadius::ZERO;
        }

        widgets.inactive.weak_bg_fill = egui::Color32::TRANSPARENT;
    }
}
