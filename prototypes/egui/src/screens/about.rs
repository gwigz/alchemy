//! About floater, modeled on `floater_about.xml` (the `sl_about` floater).

use eframe::egui;
use twill::tokens::{FontSize, Spacing};
use unic_langid::LanguageIdentifier;

use crate::context::{self, FloaterKind, Services};
use crate::data::AboutData;
use crate::i18n::{tr, tr_args};
use crate::{floater, theme, widgets};

const LOGO: egui::ImageSource<'static> = egui::include_image!(
    "../../../../indra/newview/skins/default/textures/alchemy/alchemy_128.png"
);

// Uppercase to match the generated `about-title` ("ABOUT { app }").
const APP_NAME: &str = "ALCHEMY";
const RELEASE_NOTES_URL: &str = "https://github.com/AlchemyViewer/Alchemy/releases";

const LICENCES: &str = "Alchemy is built on the Second Life viewer and dozens of \
open-source libraries (APR, Boost, cURL, Expat, FreeType, HarfBuzz, OpenJPEG, \
OpenSSL, meshoptimizer, zlib and more), each under its own license.\n\n\
(Abbreviated for the prototype; the shipping viewer lists every library's full \
license text here.)";

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum AboutTab {
    #[default]
    Info,
    Credits,
    Licences,
}

#[derive(Default)]
pub struct AboutState {
    pub open: bool,
    tab: AboutTab,
}

impl context::Floater for AboutState {
    fn kind(&self) -> FloaterKind {
        FloaterKind::About
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

        let title = tr_args(
            services.lang,
            "about-title",
            &[("capitalizedAppName", APP_NAME)],
        );
        let about = services.model.about();
        let mut open = true;
        let size = egui::vec2(480.0, 380.0);

        floater::window(
            ctx,
            FloaterKind::About.key(),
            title,
            floater::WindowOpts {
                default_size: size,
                min_size: None,
                fixed_size: Some(size),
                collapsible: true,
            },
            &mut open,
            |ui| contents(ui, &mut self.tab, about, services.lang),
        );

        self.open = open;
    }
}

fn contents(ui: &mut egui::Ui, tab: &mut AboutTab, about: &AboutData, lang: &LanguageIdentifier) {
    ui.add_space(theme::space(Spacing::S1));

    let info = tr(lang, "about-tab-info");
    let credits = tr(lang, "about-tab-credits");
    let licences = tr(lang, "about-tab-licences");

    widgets::tabs::bar(
        ui,
        tab,
        &[
            (AboutTab::Info, &info),
            (AboutTab::Credits, &credits),
            (AboutTab::Licences, &licences),
        ],
    );

    ui.add_space(theme::space(Spacing::S2));

    match tab {
        AboutTab::Info => info_tab(ui, about, lang),
        AboutTab::Credits => credits_tab(ui, about),
        AboutTab::Licences => licences_tab(ui),
    }
}

fn info_tab(ui: &mut egui::Ui, d: &AboutData, lang: &LanguageIdentifier) {
    let p = theme::active(ui.ctx());

    let memory = d.memory_mb.to_string();
    let concurrency = d.concurrency.to_string();

    let system = tr_args(
        lang,
        "about-system",
        &[
            ("cpu", d.cpu.as_str()),
            ("memoryMb", memory.as_str()),
            ("concurrency", concurrency.as_str()),
            ("osVersion", d.os_version.as_str()),
            ("graphicsCardVendor", d.gpu_vendor.as_str()),
            ("graphicsCard", d.gpu.as_str()),
        ],
    );

    let header = format!("{} {} ({}bit)", d.channel, d.version, d.address_size);
    let opengl = format!("OpenGL Version: {}", d.opengl_version);
    let body = format!("{header}\n\n{system}\n\n{opengl}");

    let content_h = ui.available_height();
    let logo_col_w = theme::space(Spacing::S36);

    ui.horizontal_top(|ui| {
        ui.allocate_ui_with_layout(
            egui::vec2(logo_col_w, content_h),
            egui::Layout::top_down(egui::Align::Center),
            |ui| {
                ui.add(
                    egui::Image::new(LOGO).fit_to_exact_size(egui::vec2(logo_col_w, logo_col_w)),
                );

                ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
                    let label = widgets::icon::labeled(
                        widgets::icon::copy(),
                        &tr(lang, "about-copy"),
                        theme::font(FontSize::Base),
                        p.secondary_foreground,
                    );

                    let copy = widgets::field::button_labeled(
                        ui,
                        widgets::field::ButtonVariant::Secondary,
                        egui::vec2(logo_col_w, theme::field_h()),
                        label,
                    );

                    if copy.clicked() {
                        ui.ctx().copy_text(body.clone());
                    }
                });
            },
        );

        // Force vertical layout so labels stack; the outer `horizontal_top` would lay them left-to-right.
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                    ui.label(&header);
                    ui.hyperlink_to("Release Notes", RELEASE_NOTES_URL);
                    ui.add_space(theme::space(Spacing::S2));
                    ui.label(&system);
                    ui.add_space(theme::space(Spacing::S2));
                    ui.label(&opengl);
                });
            });
    });
}

fn credits_tab(ui: &mut egui::Ui, d: &AboutData) {
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            egui::CollapsingHeader::new("Alchemy")
                .default_open(true)
                .show(ui, |ui| {
                    ui.label("Alchemy is brought to you by the Alchemy Development Group:");
                    ui.add_space(theme::space(Spacing::S1));

                    for name in &d.credits_alchemy {
                        ui.label(name);
                    }
                });

            egui::CollapsingHeader::new("Second Life").show(ui, |ui| {
                ui.label("Second Life is brought to you by the Lindens, with open source contributions from:");
                ui.add_space(theme::space(Spacing::S1));

                for name in &d.credits_sl {
                    ui.label(name);
                }
            });
        });
}

fn licences_tab(ui: &mut egui::Ui) {
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.label(LICENCES);
        });
}
