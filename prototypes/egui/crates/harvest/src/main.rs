//! Harvests curated strings from the real SL XUI skin tree into Fluent `.ftl` files. Run: `cargo run -p harvest`.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

enum At {
    Attr(&'static str),
    Text,
}

struct Entry {
    // Fluent message id (no dots; Fluent identifiers are `[a-zA-Z][\w-]*`).
    key: &'static str,
    file: &'static str,
    elem: &'static str,
    at: At,
}

const LOGIN_ENTRIES: &[Entry] = &[
    Entry {
        key: "login-username",
        file: "panel_login.xml",
        elem: "username_combo",
        at: At::Attr("label"),
    },
    Entry {
        key: "login-password",
        file: "panel_login.xml",
        elem: "password_edit",
        at: At::Attr("label"),
    },
    Entry {
        key: "login-remember-username",
        file: "panel_login.xml",
        elem: "remember_name",
        at: At::Attr("label"),
    },
    Entry {
        key: "login-remember-password",
        file: "panel_login.xml",
        elem: "remember_password",
        at: At::Attr("label"),
    },
    Entry {
        key: "login-start-here",
        file: "panel_login.xml",
        elem: "location_text",
        at: At::Text,
    },
    Entry {
        key: "login-grid",
        file: "panel_login.xml",
        elem: "grid_text",
        at: At::Text,
    },
    Entry {
        key: "login-start-last-location",
        file: "panel_login.xml",
        elem: "MyLastLocation",
        at: At::Attr("label"),
    },
    Entry {
        key: "login-start-home",
        file: "panel_login.xml",
        elem: "MyHome",
        at: At::Attr("label"),
    },
    Entry {
        key: "login-log-in",
        file: "panel_login.xml",
        elem: "connect_btn",
        at: At::Attr("label"),
    },
    Entry {
        key: "login-create-account",
        file: "panel_login.xml",
        elem: "sign_btn",
        at: At::Attr("label"),
    },
    Entry {
        key: "login-need-help",
        file: "panel_login.xml",
        elem: "forgot_password_text",
        at: At::Text,
    },
    Entry {
        key: "menu-me",
        file: "menu_login.xml",
        elem: "File",
        at: At::Attr("label"),
    },
    Entry {
        key: "menu-preferences",
        file: "menu_login.xml",
        elem: "Preferences...",
        at: At::Attr("label"),
    },
    Entry {
        key: "menu-exit",
        file: "menu_login.xml",
        elem: "Quit",
        at: At::Attr("label"),
    },
    Entry {
        key: "menu-help",
        file: "menu_login.xml",
        elem: "Help",
        at: At::Attr("label"),
    },
    Entry {
        key: "menu-guidebook",
        file: "menu_login.xml",
        elem: "How To",
        at: At::Attr("label"),
    },
    Entry {
        key: "menu-knowledge-base",
        file: "menu_login.xml",
        elem: "Knowledge Base",
        at: At::Attr("label"),
    },
    Entry {
        key: "menu-wiki",
        file: "menu_login.xml",
        elem: "Wiki",
        at: At::Attr("label"),
    },
    Entry {
        key: "menu-community-forums",
        file: "menu_login.xml",
        elem: "Community Forums",
        at: At::Attr("label"),
    },
    Entry {
        key: "menu-support",
        file: "menu_login.xml",
        elem: "Support portal",
        at: At::Attr("label"),
    },
    Entry {
        key: "menu-about",
        file: "menu_login.xml",
        elem: "About Second Life",
        at: At::Attr("label"),
    },
];

const ABOUT_ENTRIES: &[Entry] = &[
    Entry {
        key: "about-title",
        file: "floater_about.xml",
        elem: "floater_about",
        at: At::Attr("title"),
    },
    Entry {
        key: "about-tab-info",
        file: "floater_about.xml",
        elem: "support_panel",
        at: At::Attr("label"),
    },
    Entry {
        key: "about-tab-credits",
        file: "floater_about.xml",
        elem: "credits_panel",
        at: At::Attr("label"),
    },
    Entry {
        key: "about-tab-licences",
        file: "floater_about.xml",
        elem: "licenses_panel",
        at: At::Attr("label"),
    },
    Entry {
        key: "about-copy",
        file: "floater_about.xml",
        elem: "copy_btn",
        at: At::Attr("label"),
    },
    Entry {
        key: "about-check-updates",
        file: "floater_about.xml",
        elem: "update_btn",
        at: At::Attr("label"),
    },
    Entry {
        key: "about-system",
        file: "strings.xml",
        elem: "AboutSystem",
        at: At::Text,
    },
];

const DEBUG_ENTRIES: &[Entry] = &[
    Entry {
        key: "menu-debug",
        file: "menu_login.xml",
        elem: "Debug",
        at: At::Attr("label"),
    },
    Entry {
        key: "menu-show-debug",
        file: "menu_login.xml",
        elem: "Show Debug Menu",
        at: At::Attr("label"),
    },
    Entry {
        key: "menu-show-debug-settings",
        file: "menu_login.xml",
        elem: "Debug Settings",
        at: At::Attr("label"),
    },
    Entry {
        key: "debug-title",
        file: "floater_settings_debug.xml",
        elem: "settings_debug",
        at: At::Attr("title"),
    },
    Entry {
        key: "debug-search",
        file: "floater_settings_debug.xml",
        elem: "filter_input",
        at: At::Attr("label"),
    },
    Entry {
        key: "debug-reset",
        file: "floater_settings_debug.xml",
        elem: "default_btn",
        at: At::Attr("label"),
    },
    Entry {
        key: "debug-changed-only",
        file: "floater_settings_debug.xml",
        elem: "hide_default",
        at: At::Attr("label"),
    },
];

const PREFERENCES_ENTRIES: &[Entry] = &[
    Entry {
        key: "pref-title",
        file: "floater_preferences.xml",
        elem: "Preferences",
        at: At::Attr("title"),
    },
    Entry {
        key: "pref-ok",
        file: "floater_preferences.xml",
        elem: "OK",
        at: At::Attr("label"),
    },
    Entry {
        key: "pref-cancel",
        file: "floater_preferences.xml",
        elem: "Cancel",
        at: At::Attr("label"),
    },
    Entry {
        key: "pref-search",
        file: "floater_preferences.xml",
        elem: "search_prefs_edit",
        at: At::Attr("label"),
    },
    Entry {
        key: "pref-cat-general",
        file: "floater_preferences.xml",
        elem: "general",
        at: At::Attr("label"),
    },
    Entry {
        key: "pref-cat-graphics",
        file: "floater_preferences.xml",
        elem: "display",
        at: At::Attr("label"),
    },
    Entry {
        key: "pref-cat-sound-media",
        file: "floater_preferences.xml",
        elem: "audio",
        at: At::Attr("label"),
    },
    Entry {
        key: "pref-cat-chat",
        file: "floater_preferences.xml",
        elem: "chat",
        at: At::Attr("label"),
    },
    Entry {
        key: "pref-cat-move-view",
        file: "floater_preferences.xml",
        elem: "move",
        at: At::Attr("label"),
    },
    Entry {
        key: "pref-cat-notifications",
        file: "floater_preferences.xml",
        elem: "msgs",
        at: At::Attr("label"),
    },
    Entry {
        key: "pref-cat-colors",
        file: "floater_preferences.xml",
        elem: "colors",
        at: At::Attr("label"),
    },
    Entry {
        key: "pref-cat-privacy",
        file: "floater_preferences.xml",
        elem: "im",
        at: At::Attr("label"),
    },
    Entry {
        key: "pref-cat-setup",
        file: "floater_preferences.xml",
        elem: "input",
        at: At::Attr("label"),
    },
    Entry {
        key: "pref-cat-advanced",
        file: "floater_preferences.xml",
        elem: "advanced1",
        at: At::Attr("label"),
    },
    Entry {
        key: "pref-cat-uploads",
        file: "floater_preferences.xml",
        elem: "uploads",
        at: At::Attr("label"),
    },
    Entry {
        key: "pref-cat-controls",
        file: "floater_preferences.xml",
        elem: "controls",
        at: At::Attr("label"),
    },
    Entry {
        key: "pref-cat-themes",
        file: "floater_preferences.xml",
        elem: "skins",
        at: At::Attr("label"),
    },
    Entry {
        key: "pref-cat-interface",
        file: "floater_preferences.xml",
        elem: "interface",
        at: At::Attr("label"),
    },
    Entry {
        key: "pref-language",
        file: "panel_preferences_general.xml",
        elem: "language_textbox",
        at: At::Text,
    },
    Entry {
        key: "pref-time-format",
        file: "panel_preferences_general.xml",
        elem: "time_format_textbox",
        at: At::Text,
    },
    Entry {
        key: "pref-content-rated",
        file: "panel_preferences_general.xml",
        elem: "maturity_desired_prompt",
        at: At::Text,
    },
    Entry {
        key: "pref-show-favorites",
        file: "panel_preferences_general.xml",
        elem: "favorites_on_login_check",
        at: At::Attr("label"),
    },
    Entry {
        key: "pref-favorites-hint",
        file: "panel_preferences_general.xml",
        elem: "favorites_check_extra_text",
        at: At::Text,
    },
    Entry {
        key: "pref-name-tags",
        file: "panel_preferences_general.xml",
        elem: "name_tags_textbox",
        at: At::Text,
    },
    Entry {
        key: "pref-tag-off",
        file: "panel_preferences_general.xml",
        elem: "radio",
        at: At::Attr("label"),
    },
    Entry {
        key: "pref-tag-on",
        file: "panel_preferences_general.xml",
        elem: "radio2",
        at: At::Attr("label"),
    },
    Entry {
        key: "pref-tag-brief",
        file: "panel_preferences_general.xml",
        elem: "radio3",
        at: At::Attr("label"),
    },
    Entry {
        key: "pref-my-name",
        file: "panel_preferences_general.xml",
        elem: "show_my_name_checkbox1",
        at: At::Attr("label"),
    },
    Entry {
        key: "pref-usernames",
        file: "panel_preferences_general.xml",
        elem: "show_slids",
        at: At::Attr("label"),
    },
    Entry {
        key: "pref-distance",
        file: "panel_preferences_general.xml",
        elem: "show_distance",
        at: At::Attr("label"),
    },
    Entry {
        key: "pref-display-names",
        file: "panel_preferences_general.xml",
        elem: "display_names_check",
        at: At::Attr("label"),
    },
    Entry {
        key: "pref-highlight-friends",
        file: "panel_preferences_general.xml",
        elem: "show_friends",
        at: At::Attr("label"),
    },
    Entry {
        key: "pref-letter-keys",
        file: "panel_preferences_general.xml",
        elem: "inworld_typing_rg_label",
        at: At::Text,
    },
    Entry {
        key: "pref-keys-chat",
        file: "panel_preferences_general.xml",
        elem: "radio_start_chat",
        at: At::Attr("label"),
    },
    Entry {
        key: "pref-keys-move",
        file: "panel_preferences_general.xml",
        elem: "radio_move",
        at: At::Attr("label"),
    },
    Entry {
        key: "pref-away-timeout",
        file: "panel_preferences_general.xml",
        elem: "title_afk_text",
        at: At::Text,
    },
    Entry {
        key: "pref-dnd-response",
        file: "panel_preferences_general.xml",
        elem: "text_box3",
        at: At::Text,
    },
    // Combo options. Listed ascending (mildest first) to match our UI order, regardless of the
    // XUI's own ordering. Some items are untranslated upstream and fall back to English.
    Entry {
        key: "pref-time-12h",
        file: "panel_preferences_general.xml",
        elem: "12H",
        at: At::Attr("label"),
    },
    Entry {
        key: "pref-time-24h",
        file: "panel_preferences_general.xml",
        elem: "24H",
        at: At::Attr("label"),
    },
    Entry {
        key: "pref-rating-pg",
        file: "panel_preferences_general.xml",
        elem: "Desired_PG",
        at: At::Attr("label"),
    },
    Entry {
        key: "pref-rating-mature",
        file: "panel_preferences_general.xml",
        elem: "Desired_Mature",
        at: At::Attr("label"),
    },
    Entry {
        key: "pref-rating-adult",
        file: "panel_preferences_general.xml",
        elem: "Desired_Adult",
        at: At::Attr("label"),
    },
    Entry {
        key: "pref-tags-none",
        file: "panel_preferences_general.xml",
        elem: "no_tags",
        at: At::Attr("label"),
    },
    Entry {
        key: "pref-tags-mine",
        file: "panel_preferences_general.xml",
        elem: "my_tag",
        at: At::Attr("label"),
    },
    Entry {
        key: "pref-tags-all",
        file: "panel_preferences_general.xml",
        elem: "all_tags",
        at: At::Attr("label"),
    },
    Entry {
        key: "pref-afk-2",
        file: "panel_preferences_general.xml",
        elem: "item0",
        at: At::Attr("label"),
    },
    Entry {
        key: "pref-afk-5",
        file: "panel_preferences_general.xml",
        elem: "item1",
        at: At::Attr("label"),
    },
    Entry {
        key: "pref-afk-10",
        file: "panel_preferences_general.xml",
        elem: "item2",
        at: At::Attr("label"),
    },
    Entry {
        key: "pref-afk-30",
        file: "panel_preferences_general.xml",
        elem: "item3",
        at: At::Attr("label"),
    },
    Entry {
        key: "pref-afk-never",
        file: "panel_preferences_general.xml",
        elem: "item4",
        at: At::Attr("label"),
    },
];

const GROUPS: &[(&str, &[Entry])] = &[
    ("login.ftl", LOGIN_ENTRIES),
    ("about.ftl", ABOUT_ENTRIES),
    ("debug.ftl", DEBUG_ENTRIES),
    ("preferences.ftl", PREFERENCES_ENTRIES),
];

// Tried (in order) when a primary entry yields nothing for a locale: some skins carry the same
// concept under a different element, sometimes in another floater. E.g. EN's "Create account" is
// the `sign_btn` button label but the JA skin localizes it as the `sign_up_text` text node; the
// login `grid_text` label is untranslated, but god-tools has the same word as a panel label.
const FALLBACKS: &[Entry] = &[
    Entry {
        key: "login-create-account",
        file: "panel_login.xml",
        elem: "sign_up_text",
        at: At::Text,
    },
    Entry {
        key: "login-grid",
        file: "floater_god_tools.xml",
        elem: "grid",
        at: At::Attr("label"),
    },
];

const LOCALES: &[&str] = &[
    "en", "de", "fr", "es", "it", "ja", "pt", "ru", "pl", "tr", "zh", "da",
];

fn main() {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let xui_root = manifest.join("../../../../indra/newview/skins/default/xui");
    let out_root = manifest.join("../../i18n");

    let total: usize = GROUPS.iter().map(|(_, e)| e.len()).sum();
    let mut summary: Vec<(String, usize)> = Vec::new();

    for &locale in LOCALES {
        let dir = xui_root.join(locale);
        if !dir.exists() {
            eprintln!("skip {locale}: no xui dir");
            continue;
        }

        let mut count = 0;

        for (out, entries) in GROUPS {
            let mut found: BTreeMap<&str, String> = BTreeMap::new();

            for entry in *entries {
                if let Some(val) = read_entry(&dir, entry) {
                    found.insert(entry.key, to_fluent(&val));
                }
            }

            for fb in FALLBACKS {
                if entries.iter().any(|e| e.key == fb.key) && !found.contains_key(fb.key) {
                    if let Some(val) = read_entry(&dir, fb) {
                        found.insert(fb.key, to_fluent(&val));
                    }
                }
            }

            if locale == "en" {
                for entry in *entries {
                    if !found.contains_key(entry.key) {
                        eprintln!("WARN en missing source for {}", entry.key);
                    }
                }
            }

            count += found.len();
            write_ftl(&out_root, locale, out, &found);
        }

        summary.push((locale.to_string(), count));
    }

    println!("Harvested {total} keys across locales:");
    for (loc, n) in &summary {
        println!("  {loc}: {n}/{total}");
    }
    println!("Output: {}", out_root.display());
}

fn read_entry(dir: &Path, entry: &Entry) -> Option<String> {
    let path = dir.join(entry.file);
    let xml = fs::read_to_string(&path).ok()?;
    let doc = roxmltree::Document::parse(&xml).ok()?;

    let node = doc
        .descendants()
        .find(|node| node.attribute("name") == Some(entry.elem))?;

    let raw = match entry.at {
        At::Attr(attr) => node.attribute(attr)?.to_string(),
        At::Text => node.text()?.trim().to_string(),
    };
    let raw = raw.trim().to_string();

    (!raw.is_empty()).then_some(raw)
}

// Convert XUI `[BRACKET]` placeholders to Fluent `{ $camelCase }`.
fn to_fluent(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars();

    while let Some(ch) = chars.next() {
        if ch != '[' {
            out.push(ch);
            continue;
        }

        let mut name = String::new();

        for inner in chars.by_ref() {
            if inner == ']' {
                break;
            }

            name.push(inner);
        }

        // [X,number,2] etc. -> take the leading token as the variable.
        let var = name.split(',').next().unwrap_or("").trim();

        out.push_str("{ $");
        out.push_str(&to_camel(var));
        out.push_str(" }");
    }

    out.trim().to_string()
}

fn to_camel(input: &str) -> String {
    let mut out = String::new();
    let mut upper = false;

    for (index, ch) in input.chars().enumerate() {
        if ch == '_' || ch == '-' {
            upper = true;
        } else if upper {
            out.extend(ch.to_uppercase());
            upper = false;
        } else if index == 0 {
            out.extend(ch.to_lowercase());
        } else {
            out.push(ch.to_ascii_lowercase());
        }
    }

    out
}

fn write_ftl(out_root: &Path, locale: &str, file: &str, msgs: &BTreeMap<&str, String>) {
    let dir = out_root.join(locale);
    fs::create_dir_all(&dir).expect("create locale dir");

    let mut body = String::from("# Generated by `cargo run --features harvest --bin harvest`.\n");
    body.push_str("# Source: indra/newview/skins/default/xui/<locale>/. Do not edit by hand.\n\n");

    for (key, val) in msgs {
        if val.contains('\n') {
            let _ = writeln!(body, "{key} =");

            for line in val.lines() {
                let _ = writeln!(body, "    {line}");
            }
        } else {
            let _ = writeln!(body, "{key} = {val}");
        }
    }

    fs::write(dir.join(file), body).expect("write ftl");
}
