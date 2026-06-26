pub mod about;
pub mod confirm_quit;
pub mod debug_settings;
pub mod login;
pub mod preferences;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Login,
}

impl Screen {
    pub const ALL: &'static [Screen] = &[Self::Login];

    pub const fn title(self) -> &'static str {
        match self {
            Self::Login => "Login",
        }
    }
}
