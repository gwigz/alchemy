//! egui only reads TTF/OTF, so Alchemy's `.woff2` faces were converted once into `assets/fonts/`.

use std::sync::Arc;

use eframe::egui::{self, FontData, FontDefinitions, FontFamily};

const INTER: &[u8] = include_bytes!("../../assets/fonts/InterVariable.ttf");
const IBM_PLEX_SANS: &[u8] = include_bytes!("../../assets/fonts/IBMPlexSansVar-Roman.ttf");
const SOURCE_SANS: &[u8] = include_bytes!("../../assets/fonts/SourceSans3VF-Upright.ttf");
const DEJAVU_SANS: &[u8] = include_bytes!("../../assets/fonts/DejaVuSans.ttf");
const OPEN_DYSLEXIC: &[u8] = include_bytes!("../../assets/fonts/OpenDyslexic-Regular.otf");
const LEXICA: &[u8] = include_bytes!("../../assets/fonts/LexicaUltralegible-Regular.ttf");

const CASCADIA: &[u8] = include_bytes!("../../assets/fonts/CascadiaCodeNF.ttf");
const IBM_PLEX_MONO: &[u8] = include_bytes!("../../assets/fonts/IBMPlexMono-Regular.ttf");
const ZERO_X_PROTO: &[u8] = include_bytes!("../../assets/fonts/0xProto-Regular.ttf");
const SOURCE_CODE: &[u8] = include_bytes!("../../assets/fonts/SourceCodeVF-Upright.ttf");
const DEJAVU_MONO: &[u8] = include_bytes!("../../assets/fonts/DejaVuSansMono.ttf");

// Source Han Sans (the viewer's CJK family), subset to the glyphs our locales use.
const SOURCE_HAN_JP: &[u8] = include_bytes!("../../assets/fonts/SourceHanSansJP-Subset.ttf");

const FACES: &[(&str, &[u8])] = &[
    ("inter", INTER),
    ("ibm_plex_sans", IBM_PLEX_SANS),
    ("source_sans", SOURCE_SANS),
    ("dejavu_sans", DEJAVU_SANS),
    ("open_dyslexic", OPEN_DYSLEXIC),
    ("lexica", LEXICA),
    ("cascadia", CASCADIA),
    ("ibm_plex_mono", IBM_PLEX_MONO),
    ("zero_x_proto", ZERO_X_PROTO),
    ("source_code", SOURCE_CODE),
    ("dejavu_mono", DEJAVU_MONO),
    ("source_han_jp", SOURCE_HAN_JP),
];

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum UiFont {
    Default,
    Inter,
    IbmPlexSans,
    SourceSans,
    DejaVuSans,
    OpenDyslexic,
    Lexica,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MonoFont {
    Default,
    CascadiaCode,
    IbmPlexMono,
    ZeroXProto,
    SourceCode,
    DejaVuMono,
}

impl UiFont {
    fn key(self) -> Option<&'static str> {
        match self {
            UiFont::Default => None,
            UiFont::Inter => Some("inter"),
            UiFont::IbmPlexSans => Some("ibm_plex_sans"),
            UiFont::SourceSans => Some("source_sans"),
            UiFont::DejaVuSans => Some("dejavu_sans"),
            UiFont::OpenDyslexic => Some("open_dyslexic"),
            UiFont::Lexica => Some("lexica"),
        }
    }
}

impl MonoFont {
    fn key(self) -> Option<&'static str> {
        match self {
            MonoFont::Default => None,
            MonoFont::CascadiaCode => Some("cascadia"),
            MonoFont::IbmPlexMono => Some("ibm_plex_mono"),
            MonoFont::ZeroXProto => Some("zero_x_proto"),
            MonoFont::SourceCode => Some("source_code"),
            MonoFont::DejaVuMono => Some("dejavu_mono"),
        }
    }
}

// Inter is Alchemy's default SansSerifBase.
pub fn ui_fonts() -> &'static [(&'static str, UiFont)] {
    &[
        ("Inter", UiFont::Inter),
        ("IBM Plex Sans", UiFont::IbmPlexSans),
        ("Source Sans 3", UiFont::SourceSans),
        ("DejaVu Sans", UiFont::DejaVuSans),
        ("OpenDyslexic", UiFont::OpenDyslexic),
        ("Lexica Ultralegible", UiFont::Lexica),
        ("egui default", UiFont::Default),
    ]
}

// Cascadia Code is Alchemy's default MonospaceBase.
pub fn mono_fonts() -> &'static [(&'static str, MonoFont)] {
    &[
        ("Cascadia Code", MonoFont::CascadiaCode),
        ("IBM Plex Mono", MonoFont::IbmPlexMono),
        ("0xProto", MonoFont::ZeroXProto),
        ("Source Code Pro", MonoFont::SourceCode),
        ("DejaVu Sans Mono", MonoFont::DejaVuMono),
        ("egui default", MonoFont::Default),
    ]
}

// Triggers a glyph-atlas rebuild, so call only on selection change.
pub fn install_fonts(ctx: &egui::Context, ui: UiFont, mono: MonoFont) {
    let mut defs = FontDefinitions::default();

    for (key, bytes) in FACES {
        defs.font_data
            .insert((*key).to_owned(), Arc::new(FontData::from_static(bytes)));
    }

    if let Some(key) = ui.key() {
        if let Some(list) = defs.families.get_mut(&FontFamily::Proportional) {
            list.insert(0, key.to_owned());
        }
    }
    if let Some(key) = mono.key() {
        if let Some(list) = defs.families.get_mut(&FontFamily::Monospace) {
            list.insert(0, key.to_owned());
        }
    }

    // Append CJK at the back of both chains so it only fills glyphs the chosen Latin font lacks.
    for family in [FontFamily::Proportional, FontFamily::Monospace] {
        if let Some(list) = defs.families.get_mut(&family) {
            list.push("source_han_jp".to_owned());
        }
    }

    // Each iconflow pack font gets its own family so the UI font can't shadow its codepoints.
    for asset in iconflow::fonts() {
        defs.font_data.insert(
            asset.family.to_owned(),
            Arc::new(FontData::from_static(asset.bytes)),
        );
        defs.families.insert(
            FontFamily::Name(asset.family.into()),
            vec![asset.family.to_owned()],
        );
    }

    ctx.set_fonts(defs);
}
