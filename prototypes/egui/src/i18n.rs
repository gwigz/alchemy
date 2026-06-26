//! Locale `.ftl` files are embedded at compile time, so this works in WASM with no runtime I/O.

use std::borrow::Cow;
use std::collections::HashMap;

use fluent_templates::fluent_bundle::FluentValue;
use fluent_templates::{static_loader, Loader};
use unic_langid::{langid, LanguageIdentifier};

static_loader! {
    static LOCALES = {
        locales: "./i18n",
        fallback_language: "en",
    };
}

pub fn available() -> Vec<(&'static str, LanguageIdentifier)> {
    vec![
        ("English", langid!("en")),
        ("Deutsch", langid!("de")),
        ("Français", langid!("fr")),
        ("Español", langid!("es")),
        ("Italiano", langid!("it")),
        ("日本語", langid!("ja")),
        ("Português", langid!("pt")),
        ("Русский", langid!("ru")),
    ]
}

pub fn tr(lang: &LanguageIdentifier, key: &str) -> String {
    LOCALES.lookup(lang, key)
}

pub fn tr_args(lang: &LanguageIdentifier, key: &str, args: &[(&'static str, &str)]) -> String {
    let map: HashMap<Cow<'static, str>, FluentValue> = args
        .iter()
        .map(|(k, v)| (Cow::Borrowed(*k), FluentValue::from(v.to_string())))
        .collect();

    LOCALES.lookup_with_args(lang, key, &map)
}
