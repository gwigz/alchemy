//! Quit confirmation, modeled on the viewer's `ConfirmQuit` alertmodal.

use eframe::egui::{self, RichText};
use twill::tokens::{FontSize, Spacing};

use crate::context::{self, FloaterKind, Services};
use crate::{theme, widgets};

#[derive(Default)]
pub struct ConfirmQuitState {
    pub open: bool,
    dont_show_again: bool,
}

impl context::Floater for ConfirmQuitState {
    fn kind(&self) -> FloaterKind {
        FloaterKind::ConfirmQuit
    }

    fn open(&self) -> bool {
        self.open
    }

    fn set_open(&mut self, open: bool) {
        self.open = open;
    }

    fn show(&mut self, ctx: &egui::Context, _services: &Services) {
        if !self.open {
            return;
        }

        show_modal(ctx, self);
    }
}

fn show_modal(ctx: &egui::Context, state: &mut ConfirmQuitState) {
    egui::Modal::new(egui::Id::new("confirm_quit_modal"))
        .frame(theme::window_frame(ctx, false))
        .show(ctx, |ui| {
            let width = theme::space(Spacing::S56);
            ui.set_width(width);
            ui.add_space(theme::space(Spacing::S3));

            ui.vertical_centered(|ui| {
                ui.label(
                    RichText::new("Are you sure you want to quit?")
                        .size(theme::font(FontSize::Base)),
                );

                ui.add_space(theme::space(Spacing::S1_5));
                ui.checkbox(&mut state.dont_show_again, "Don't show me this again");
            });

            ui.add_space(theme::space(Spacing::S1_5));

            // Centered row with Quit first, matching the viewer's dialog.
            let btn_w = theme::space(Spacing::S24);
            let gap = theme::space(Spacing::S2);
            let pad = ((width - (btn_w * 2.0 + gap)) / 2.0).max(0.0);

            let size = egui::vec2(btn_w, theme::field_h());
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                ui.add_space(pad);

                if widgets::field::button(ui, widgets::field::ButtonVariant::Primary, size, "Quit")
                    .clicked()
                {
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                    state.open = false;
                }

                ui.add_space(gap);

                if widgets::field::button(
                    ui,
                    widgets::field::ButtonVariant::Secondary,
                    size,
                    "Don't Quit",
                )
                .clicked()
                {
                    state.open = false;
                }
            });

            ui.add_space(theme::space(Spacing::S0_5));
        });

    // Escape dismisses, like cancelling the dialog.
    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        state.open = false;
    }
}
