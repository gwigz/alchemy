//! Login screen, modeled on `panel_login.xml`.

use eframe::egui::{self, RichText, Vec2};
use twill::tokens::{BorderRadius, FontSize, Spacing};
use unic_langid::LanguageIdentifier;

use crate::context::{self, Command, FloaterKind, Services};
use crate::data::LoginData;
use crate::i18n::{tr, tr_args};
use crate::{notify, theme, widgets};

const SL_LOGO: egui::ImageSource<'static> = egui::include_image!(
    "../../../../indra/newview/skins/default/textures/windows/login_sl_logo_horizontal.png"
);

const APP_NAME: &str = "Second Life";

const KNOWLEDGE_BASE_URL: &str = "http://community.secondlife.com/t5/English-Knowledge-Base/Second-Life-User-s-Guide/ta-p/1244857";
const WIKI_URL: &str = "http://wiki.secondlife.com";
const FORUMS_URL: &str = "http://community.secondlife.com/t5/Forums/ct-p/Forums";
const SUPPORT_URL: &str = "https://support.secondlife.com/";

#[derive(Default, Clone, Copy, PartialEq, Eq)]
enum LogLevel {
    Debug,
    #[default]
    Info,
    Warning,
    Error,
    None,
}

#[derive(Default)]
pub struct LoginState {
    pub username: String,
    pub password: String,
    pub remember_username: bool,
    pub remember_password: bool,
    pub start_location: usize,
    pub grid: usize,
    show_debug: bool,
    log_level: LogLevel,
    initialized: bool,
}

pub fn show(ui: &mut egui::Ui, state: &mut LoginState, services: &Services) {
    let login = services.model.login();
    let lang = services.lang;

    if !state.initialized {
        state.remember_username = login.remember_username;
        state.remember_password = login.remember_password;

        if let Some(first) = login.remembered_users.first() {
            first.username.clone_into(&mut state.username);
        }

        state.initialized = true;
    }

    egui::TopBottomPanel::top("login_menu")
        .frame(egui::Frame::NONE.fill(egui::Color32::BLACK))
        .show_separator_line(false)
        .show_inside(ui, |ui| login_menu_bar(ui, lang, state));

    egui::CentralPanel::default()
        .frame(egui::Frame::NONE)
        .show_inside(ui, |ui| {
            let p = theme::active(ui.ctx());
            let avail = ui.available_size();

            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;

                let web_w = (avail.x - theme::sidebar_w()).max(0.0);
                let (rect, _) =
                    ui.allocate_exact_size(Vec2::new(web_w, avail.y), egui::Sense::hover());

                ui.painter()
                    .rect_filled(rect, theme::radius(BorderRadius::None), p.background);

                ui.allocate_ui_with_layout(
                    Vec2::new(theme::sidebar_w(), avail.y),
                    egui::Layout::top_down(egui::Align::Center),
                    |ui| {
                        let bg = ui.available_rect_before_wrap();

                        ui.painter()
                            .rect_filled(bg, theme::radius(BorderRadius::None), p.card);

                        ui.add_space(theme::space(Spacing::S10));

                        ui.add(egui::Image::new(SL_LOGO).fit_to_exact_size(Vec2::new(
                            theme::space(Spacing::S40),
                            theme::space(Spacing::S20),
                        )));

                        ui.add_space(theme::space(Spacing::S7));

                        ui.allocate_ui_with_layout(
                            Vec2::new(theme::field_w(), ui.available_height()),
                            egui::Layout::top_down(egui::Align::Min),
                            |ui| login_form(ui, login, state, lang),
                        );
                    },
                );
            });
        });
}

fn login_form(
    ui: &mut egui::Ui,
    login: &LoginData,
    state: &mut LoginState,
    lang: &LanguageIdentifier,
) {
    let p = theme::active(ui.ctx());

    username_field(ui, login, state, lang);

    ui.add_space(theme::space(Spacing::S1));

    ui.checkbox(
        &mut state.remember_username,
        tr(lang, "login-remember-username"),
    );

    ui.add_space(theme::space(Spacing::S3));

    widgets::field::password(ui, &mut state.password, &tr(lang, "login-password"));

    ui.add_space(theme::space(Spacing::S1));

    ui.checkbox(
        &mut state.remember_password,
        tr(lang, "login-remember-password"),
    );

    ui.add_space(theme::space(Spacing::S3_5));

    widgets::field::label(ui, &tr(lang, "login-start-here"));

    // The standard options are localized like the viewer; typed locations keep their own label.
    let start_labels: Vec<String> = login
        .start_locations
        .iter()
        .map(|loc| match loc.value.as_str() {
            "last" => tr(lang, "login-start-last-location"),
            "home" => tr(lang, "login-start-home"),
            _ => loc.label.clone(),
        })
        .collect();

    widgets::combo::select(
        ui,
        "start_location",
        &mut state.start_location,
        start_labels.iter().map(String::as_str),
    );

    ui.add_space(theme::space(Spacing::S3_5));

    widgets::field::label(ui, &tr(lang, "login-grid"));

    widgets::combo::select(
        ui,
        "grid",
        &mut state.grid,
        login.grids.iter().map(|grid| grid.label.as_str()),
    );

    ui.add_space(theme::space(Spacing::S6));

    if widgets::field::primary_button(ui, &tr(lang, "login-log-in")).clicked() {
        if state.username.trim().is_empty() {
            notify::error(ui.ctx(), "Please enter a username.");
        } else {
            notify::success(
                ui.ctx(),
                format!(
                    "Logging in as {} on {}…",
                    state.username, login.grids[state.grid].label
                ),
            );
        }
    }

    ui.add_space(theme::space(Spacing::S2_5));

    ui.vertical_centered(|ui| {
        // `override_text_color` (in theme::install) would force this to the body color.
        // Opens in a new window/tab rather than navigating away from the viewer.
        ui.add(
            egui::Hyperlink::from_label_and_url(
                RichText::new(tr(lang, "login-need-help"))
                    .size(theme::font(FontSize::Base))
                    .color(p.primary),
                &login.help_url,
            )
            .open_in_new_tab(true),
        );
    });

    ui.add_space(theme::space(Spacing::S2_5));

    if widgets::field::secondary_button(ui, &tr(lang, "login-create-account")).clicked() {
        ui.ctx()
            .open_url(egui::OpenUrl::new_tab(&login.create_account_url));
    }
}

fn username_field(
    ui: &mut egui::Ui,
    login: &LoginData,
    state: &mut LoginState,
    lang: &LanguageIdentifier,
) {
    let options: Vec<(&str, &str)> = login
        .remembered_users
        .iter()
        .map(|u| (u.display.as_str(), u.username.as_str()))
        .collect();

    widgets::combo::editable(
        ui,
        "username_combo",
        &mut state.username,
        &tr(lang, "login-username"),
        &options,
    );
}

fn login_menu_bar(ui: &mut egui::Ui, lang: &LanguageIdentifier, state: &mut LoginState) {
    let debug_visible = state.show_debug;

    // Split disjoint field borrows so the Me and Debug closures can coexist.
    let LoginState {
        show_debug,
        log_level,
        ..
    } = state;

    let mut menus = vec![
        widgets::menu::Menu::new(tr(lang, "menu-me"), |ui| me_items(ui, lang, show_debug)),
        widgets::menu::Menu::new(tr(lang, "menu-help"), |ui| help_items(ui, lang)),
    ];

    if debug_visible {
        menus.push(widgets::menu::Menu::new(tr(lang, "menu-debug"), |ui| {
            debug_items(ui, lang, log_level);
        }));
    }

    widgets::menu::bar(ui, &mut menus);
}

fn me_items(ui: &mut egui::Ui, lang: &LanguageIdentifier, show_debug: &mut bool) {
    if widgets::menu::item(ui, &tr(lang, "menu-preferences"), "control|P").clicked() {
        context::post(ui.ctx(), Command::OpenFloater(FloaterKind::Preferences));
        ui.close();
    }

    if widgets::menu::item(ui, &tr(lang, "menu-show-debug"), "control|alt|D").clicked() {
        *show_debug = !*show_debug;
    }

    let exit = tr_args(lang, "menu-exit", &[("appName", APP_NAME)]);
    if widgets::menu::item(ui, &exit, "control|Q").clicked() {
        context::post(ui.ctx(), Command::OpenFloater(FloaterKind::ConfirmQuit));
        ui.close();
    }
}

fn help_items(ui: &mut egui::Ui, lang: &LanguageIdentifier) {
    if widgets::menu::item(ui, &tr(lang, "menu-guidebook"), "F1").clicked() {
        notify::info(ui.ctx(), "Guidebook isn't wired up in this mock yet.");
        ui.close();
    }

    ui.separator();

    widgets::menu::link(ui, &tr(lang, "menu-knowledge-base"), KNOWLEDGE_BASE_URL);
    widgets::menu::link(ui, &tr(lang, "menu-wiki"), WIKI_URL);
    widgets::menu::link(ui, &tr(lang, "menu-community-forums"), FORUMS_URL);
    widgets::menu::link(ui, &tr(lang, "menu-support"), SUPPORT_URL);

    ui.separator();

    let about = tr_args(lang, "menu-about", &[("appName", APP_NAME)]);
    if widgets::menu::item(ui, &about, "").clicked() {
        context::post(ui.ctx(), Command::OpenFloater(FloaterKind::About));
        ui.close();
    }
}

fn debug_items(ui: &mut egui::Ui, lang: &LanguageIdentifier, log_level: &mut LogLevel) {
    if widgets::menu::item(ui, &tr(lang, "menu-show-debug-settings"), "").clicked() {
        context::post(ui.ctx(), Command::OpenFloater(FloaterKind::DebugSettings));
        ui.close();
    }

    if widgets::menu::item(ui, "Show Color settings", "").clicked() {
        stub(ui);
    }

    ui.separator();

    ui.menu_button("Set Logging Level", |ui| {
        for (label, level) in [
            ("Debug", LogLevel::Debug),
            ("Info", LogLevel::Info),
            ("Warning", LogLevel::Warning),
            ("Error", LogLevel::Error),
            ("None", LogLevel::None),
        ] {
            if widgets::menu::radio_item(ui, label, *log_level == level).clicked() {
                *log_level = level;
            }
        }
    });

    ui.separator();

    if widgets::menu::item(ui, "Set Window Size...", "").clicked() {
        stub(ui);
    }
}

fn stub(ui: &mut egui::Ui) {
    notify::info(ui.ctx(), "That debug tool isn't wired up in this mock yet.");
    ui.close();
}
