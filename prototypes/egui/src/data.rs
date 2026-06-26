//! Structs mirror the shape of SL's LLSD login/about/settings payloads.

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Grid {
    #[allow(dead_code)]
    pub id: String,
    pub label: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RememberedUser {
    pub username: String,
    pub display: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StartLocation {
    pub value: String,
    pub label: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LoginData {
    pub grids: Vec<Grid>,
    pub remembered_users: Vec<RememberedUser>,
    pub start_locations: Vec<StartLocation>,
    pub remember_username: bool,
    pub remember_password: bool,
    pub help_url: String,
    pub create_account_url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AboutData {
    pub channel: String,
    pub version: String,
    pub address_size: u8,
    pub opengl_version: String,
    pub cpu: String,
    pub memory_mb: u32,
    pub concurrency: u32,
    pub os_version: String,
    pub gpu_vendor: String,
    pub gpu: String,
    pub credits_alchemy: Vec<String>,
    pub credits_sl: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum SettingKind {
    Boolean,
    String,
    S32,
    F32,
    Color,
    Vec3,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DebugSetting {
    pub name: String,
    pub kind: SettingKind,
    pub value: String,
    pub default: String,
    pub comment: String,
}

impl DebugSetting {
    pub fn changed(&self) -> bool {
        self.value != self.default
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct DebugSettingsData {
    pub settings: Vec<DebugSetting>,
}

// The prototype's single data seam. A real backend (session, inventory, settings) would implement
// this in place of `MockData`; screens reach data only through `Services::model`.
pub trait Model {
    fn login(&self) -> &LoginData;
    fn about(&self) -> &AboutData;
    fn debug_settings(&self) -> &DebugSettingsData;
}

pub struct MockData {
    login: LoginData,
    about: AboutData,
    debug_settings: DebugSettingsData,
}

impl MockData {
    pub fn load() -> Self {
        let login: LoginData = serde_json::from_str(include_str!("../fixtures/login.json"))
            .expect("login.json fixture is valid");

        let about: AboutData = serde_json::from_str(include_str!("../fixtures/about.json"))
            .expect("about.json fixture is valid");

        let debug_settings: DebugSettingsData =
            serde_json::from_str(include_str!("../fixtures/debug_settings.json"))
                .expect("debug_settings.json fixture is valid");

        Self {
            login,
            about,
            debug_settings,
        }
    }
}

impl Model for MockData {
    fn login(&self) -> &LoginData {
        &self.login
    }

    fn about(&self) -> &AboutData {
        &self.about
    }

    fn debug_settings(&self) -> &DebugSettingsData {
        &self.debug_settings
    }
}
