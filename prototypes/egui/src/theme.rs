//! Palettes hand-translated from `indra/newview/skins/<skin>/colors.xml`.

use eframe::egui::{self, Color32};
use twill::egui::to_corner_radius;
use twill::tokens::{BorderRadius, BorderWidth, ColorValue, FontSize, Spacing};

pub mod fonts;

const fn to_color32(color: ColorValue) -> Color32 {
    Color32::from_rgb(color.r, color.g, color.b)
}

const fn rgb(r: u8, g: u8, b: u8) -> Color32 {
    to_color32(ColorValue::from_rgb(r, g, b))
}

#[derive(Clone, Copy)]
#[allow(dead_code)]
pub struct Palette {
    pub background: Color32,
    pub foreground: Color32,
    pub card: Color32,
    pub card_foreground: Color32,
    pub popover: Color32,
    pub popover_foreground: Color32,
    pub primary: Color32,
    pub primary_foreground: Color32,
    pub secondary: Color32,
    pub secondary_foreground: Color32,
    pub muted: Color32,
    pub muted_foreground: Color32,
    pub accent: Color32,
    pub accent_foreground: Color32,
    pub destructive: Color32,
    pub border: Color32,
    pub input: Color32,
    pub input_focus: Color32,
    pub input_readonly: Color32,
    pub input_foreground: Color32,
    pub ring: Color32,
}

// SL default skin: cyan EmphasisColor on dark greys.
const PALETTE_DEFAULT: Palette = Palette {
    background: rgb(33, 34, 35),
    foreground: rgb(205, 205, 205),
    card: rgb(20, 20, 20),
    card_foreground: rgb(205, 205, 205),
    popover: rgb(43, 43, 43),
    popover_foreground: rgb(205, 205, 205),
    primary: rgb(77, 209, 255),
    primary_foreground: rgb(0, 0, 0),
    // Title-bar band, a step lighter than the (43,43,43) body so it reads distinctly.
    secondary: rgb(54, 54, 54),
    secondary_foreground: rgb(255, 255, 255),
    muted: rgb(54, 54, 54),
    muted_foreground: rgb(200, 200, 200),
    accent: rgb(77, 209, 255),
    accent_foreground: rgb(0, 0, 0),
    destructive: rgb(200, 50, 50),
    border: rgb(60, 60, 60),
    // Line-editor fields: TextField_Off/_Active textures, light grey, pure white on focus,
    // with dark TextFgColor text (the classic SL look).
    input: rgb(192, 192, 192),
    input_focus: rgb(255, 255, 255),
    // TextBgReadOnlyColor (White_05) over the floater body: a dark inset, unlike the light field.
    input_readonly: rgb(54, 54, 54),
    input_foreground: rgb(26, 26, 26),
    ring: rgb(77, 209, 255),
};

// Alchemy skin: AlBlue cyan on AlCoalBlack, AlchemyText foreground.
const PALETTE_ALCHEMY: Palette = Palette {
    background: rgb(37, 40, 42),
    foreground: rgb(203, 205, 205),
    // Cards/popovers tinted into the blue-grey family (was flat neutral 42,42,42) and a step above
    // the background so they read as raised.
    card: rgb(44, 47, 50),
    card_foreground: rgb(203, 205, 205),
    popover: rgb(44, 47, 50),
    popover_foreground: rgb(203, 205, 205),
    primary: rgb(0, 161, 223),
    primary_foreground: rgb(255, 255, 255),
    secondary: rgb(50, 53, 56),
    secondary_foreground: rgb(203, 205, 205),
    muted: rgb(50, 53, 56),
    muted_foreground: rgb(159, 162, 163),
    accent: rgb(0, 74, 136),
    accent_foreground: rgb(255, 255, 255),
    destructive: rgb(177, 32, 41),
    // Softer than the old 84,87,90 (which outlined everything harshly); sits at the top of the
    // bg -> card -> secondary -> border ramp.
    border: rgb(60, 64, 68),
    // TextField_Off/_Active textures, composited over the panel.
    input: rgb(27, 28, 28),
    input_focus: rgb(20, 20, 20),
    // TextBgReadOnlyColor is AlchemyBlack1.
    input_readonly: rgb(28, 28, 28),
    input_foreground: rgb(203, 205, 205),
    ring: rgb(0, 161, 223),
};

// Gemini skin: warm/retro orange emphasis, brown accents, white text.
const PALETTE_GEMINI: Palette = Palette {
    background: rgb(32, 32, 32),
    foreground: rgb(255, 255, 255),
    card: rgb(40, 38, 36),
    card_foreground: rgb(255, 255, 255),
    popover: rgb(40, 38, 36),
    popover_foreground: rgb(255, 255, 255),
    primary: rgb(242, 105, 44),
    primary_foreground: rgb(20, 20, 20),
    secondary: rgb(70, 60, 52),
    secondary_foreground: rgb(255, 255, 255),
    muted: rgb(70, 60, 52),
    muted_foreground: rgb(180, 175, 170),
    accent: rgb(130, 191, 255),
    accent_foreground: rgb(20, 20, 20),
    destructive: rgb(200, 60, 50),
    border: rgb(70, 60, 52),
    // TextField textures: translucent near-black composited over the panel; no focus shift.
    input: rgb(23, 23, 23),
    input_focus: rgb(23, 23, 23),
    // TextBgReadOnlyColor (0.05 @ 25%) over the panel.
    input_readonly: rgb(33, 32, 30),
    input_foreground: rgb(255, 255, 255),
    ring: rgb(242, 105, 44),
};

// Heretic skin: neon synthwave PinkBite on Obsidian, lime accent.
const PALETTE_HERETIC: Palette = Palette {
    background: rgb(4, 12, 28),
    foreground: rgb(255, 255, 255),
    card: rgb(12, 20, 36),
    card_foreground: rgb(255, 255, 255),
    popover: rgb(12, 20, 36),
    popover_foreground: rgb(255, 255, 255),
    primary: rgb(237, 48, 156),
    primary_foreground: rgb(255, 255, 255),
    secondary: rgb(28, 36, 58),
    secondary_foreground: rgb(255, 255, 255),
    muted: rgb(28, 36, 58),
    muted_foreground: rgb(150, 160, 180),
    accent: rgb(141, 217, 68),
    accent_foreground: rgb(12, 20, 36),
    destructive: rgb(193, 2, 76),
    border: rgb(40, 46, 66),
    // TextField textures: translucent near-black composited over the Obsidian panel; no focus shift.
    input: rgb(9, 13, 21),
    input_focus: rgb(9, 13, 21),
    // TextBgReadOnlyColor (Obsidian_25) over the panel.
    input_readonly: rgb(10, 18, 34),
    input_foreground: rgb(255, 255, 255),
    ring: rgb(237, 48, 156),
};

// Ionic skin: olive emphasis on the darkest greys, muted text.
const PALETTE_IONIC: Palette = Palette {
    background: rgb(35, 35, 35),
    foreground: rgb(184, 184, 184),
    card: rgb(28, 28, 28),
    card_foreground: rgb(184, 184, 184),
    popover: rgb(28, 28, 28),
    popover_foreground: rgb(184, 184, 184),
    primary: rgb(81, 131, 28),
    primary_foreground: rgb(235, 235, 235),
    secondary: rgb(50, 50, 50),
    secondary_foreground: rgb(219, 219, 219),
    muted: rgb(50, 50, 50),
    muted_foreground: rgb(150, 150, 150),
    accent: rgb(51, 86, 16),
    accent_foreground: rgb(235, 235, 235),
    destructive: rgb(180, 60, 50),
    border: rgb(55, 55, 55),
    // TextField_Off/_Active textures, composited over the panel.
    input: rgb(27, 27, 27),
    input_focus: rgb(20, 20, 20),
    // TextBgReadOnlyColor is PlvrWindowBackgroundMidLight.
    input_readonly: rgb(33, 33, 33),
    input_foreground: rgb(184, 184, 184),
    ring: rgb(133, 163, 85),
};

// Catppuccin Latte (light): blue accent on the warm latte base, dark slate text.
const PALETTE_CATPPUCCIN_LATTE: Palette = Palette {
    background: rgb(239, 241, 245),
    foreground: rgb(76, 79, 105),
    card: rgb(230, 233, 239),
    card_foreground: rgb(76, 79, 105),
    popover: rgb(230, 233, 239),
    popover_foreground: rgb(76, 79, 105),
    primary: rgb(30, 102, 245),
    primary_foreground: rgb(239, 241, 245),
    secondary: rgb(204, 208, 218),
    secondary_foreground: rgb(76, 79, 105),
    muted: rgb(204, 208, 218),
    muted_foreground: rgb(108, 111, 133),
    accent: rgb(234, 118, 203),
    accent_foreground: rgb(239, 241, 245),
    destructive: rgb(210, 15, 57),
    border: rgb(220, 224, 232),
    input: rgb(255, 255, 255),
    input_focus: rgb(255, 255, 255),
    input_readonly: rgb(204, 208, 218),
    input_foreground: rgb(76, 79, 105),
    ring: rgb(30, 102, 245),
};

// Catppuccin Frappé: blue accent on the muted frappé base.
const PALETTE_CATPPUCCIN_FRAPPE: Palette = Palette {
    background: rgb(48, 52, 70),
    foreground: rgb(198, 208, 245),
    card: rgb(65, 69, 89),
    card_foreground: rgb(198, 208, 245),
    popover: rgb(65, 69, 89),
    popover_foreground: rgb(198, 208, 245),
    primary: rgb(140, 170, 238),
    primary_foreground: rgb(48, 52, 70),
    secondary: rgb(65, 69, 89),
    secondary_foreground: rgb(198, 208, 245),
    muted: rgb(65, 69, 89),
    muted_foreground: rgb(165, 173, 206),
    accent: rgb(244, 184, 228),
    accent_foreground: rgb(48, 52, 70),
    destructive: rgb(231, 130, 132),
    border: rgb(81, 87, 109),
    input: rgb(41, 44, 60),
    input_focus: rgb(35, 38, 52),
    input_readonly: rgb(41, 44, 60),
    input_foreground: rgb(198, 208, 245),
    ring: rgb(140, 170, 238),
};

// Catppuccin Macchiato: blue accent on the deeper macchiato base.
const PALETTE_CATPPUCCIN_MACCHIATO: Palette = Palette {
    background: rgb(36, 39, 58),
    foreground: rgb(202, 211, 245),
    card: rgb(54, 58, 79),
    card_foreground: rgb(202, 211, 245),
    popover: rgb(54, 58, 79),
    popover_foreground: rgb(202, 211, 245),
    primary: rgb(138, 173, 244),
    primary_foreground: rgb(36, 39, 58),
    secondary: rgb(54, 58, 79),
    secondary_foreground: rgb(202, 211, 245),
    muted: rgb(54, 58, 79),
    muted_foreground: rgb(165, 173, 203),
    accent: rgb(245, 189, 230),
    accent_foreground: rgb(36, 39, 58),
    destructive: rgb(237, 135, 150),
    border: rgb(73, 77, 100),
    input: rgb(30, 32, 48),
    input_focus: rgb(24, 25, 38),
    input_readonly: rgb(30, 32, 48),
    input_foreground: rgb(202, 211, 245),
    ring: rgb(138, 173, 244),
};

// Catppuccin Mocha: blue accent on the darkest mocha base.
const PALETTE_CATPPUCCIN_MOCHA: Palette = Palette {
    background: rgb(30, 30, 46),
    foreground: rgb(205, 214, 244),
    card: rgb(49, 50, 68),
    card_foreground: rgb(205, 214, 244),
    popover: rgb(49, 50, 68),
    popover_foreground: rgb(205, 214, 244),
    primary: rgb(137, 180, 250),
    primary_foreground: rgb(30, 30, 46),
    secondary: rgb(49, 50, 68),
    secondary_foreground: rgb(205, 214, 244),
    muted: rgb(49, 50, 68),
    muted_foreground: rgb(166, 173, 200),
    accent: rgb(245, 194, 231),
    accent_foreground: rgb(30, 30, 46),
    destructive: rgb(243, 139, 168),
    border: rgb(69, 71, 90),
    input: rgb(24, 24, 37),
    input_focus: rgb(17, 17, 27),
    input_readonly: rgb(24, 24, 37),
    input_foreground: rgb(205, 214, 244),
    ring: rgb(137, 180, 250),
};

// Dracula: purple primary on #282a36, pink accent, comment-grey muted text.
const PALETTE_DRACULA: Palette = Palette {
    background: rgb(40, 42, 54),
    foreground: rgb(248, 248, 242),
    card: rgb(45, 47, 61),
    card_foreground: rgb(248, 248, 242),
    popover: rgb(45, 47, 61),
    popover_foreground: rgb(248, 248, 242),
    primary: rgb(189, 147, 249),
    primary_foreground: rgb(40, 42, 54),
    secondary: rgb(68, 69, 90),
    secondary_foreground: rgb(248, 248, 242),
    muted: rgb(68, 69, 90),
    muted_foreground: rgb(98, 114, 164),
    accent: rgb(255, 121, 198),
    accent_foreground: rgb(40, 42, 54),
    destructive: rgb(255, 85, 85),
    border: rgb(54, 56, 69),
    input: rgb(33, 34, 44),
    input_focus: rgb(25, 26, 34),
    input_readonly: rgb(45, 47, 61),
    input_foreground: rgb(248, 248, 242),
    ring: rgb(189, 147, 249),
};

// Nord: frost cyan (nord8) on polar-night nord0, snow-storm text.
const PALETTE_NORD: Palette = Palette {
    background: rgb(46, 52, 64),
    foreground: rgb(216, 222, 233),
    card: rgb(59, 66, 82),
    card_foreground: rgb(216, 222, 233),
    popover: rgb(59, 66, 82),
    popover_foreground: rgb(216, 222, 233),
    primary: rgb(136, 192, 208),
    primary_foreground: rgb(46, 52, 64),
    secondary: rgb(67, 76, 94),
    secondary_foreground: rgb(216, 222, 233),
    muted: rgb(67, 76, 94),
    muted_foreground: rgb(143, 153, 173),
    accent: rgb(129, 161, 193),
    accent_foreground: rgb(46, 52, 64),
    destructive: rgb(191, 97, 106),
    border: rgb(59, 66, 82),
    input: rgb(41, 46, 57),
    input_focus: rgb(35, 39, 49),
    input_readonly: rgb(59, 66, 82),
    input_foreground: rgb(216, 222, 233),
    ring: rgb(136, 192, 208),
};

// Tokyo Night: blue primary on #1a1b26, magenta accent, comment-blue muted text.
const PALETTE_TOKYO_NIGHT: Palette = Palette {
    background: rgb(26, 27, 38),
    foreground: rgb(192, 202, 245),
    card: rgb(31, 35, 53),
    card_foreground: rgb(192, 202, 245),
    popover: rgb(31, 35, 53),
    popover_foreground: rgb(192, 202, 245),
    primary: rgb(122, 162, 247),
    primary_foreground: rgb(26, 27, 38),
    secondary: rgb(41, 46, 66),
    secondary_foreground: rgb(192, 202, 245),
    muted: rgb(41, 46, 66),
    muted_foreground: rgb(86, 95, 137),
    accent: rgb(187, 154, 247),
    accent_foreground: rgb(26, 27, 38),
    destructive: rgb(247, 118, 142),
    border: rgb(41, 46, 66),
    input: rgb(22, 22, 30),
    input_focus: rgb(16, 16, 22),
    input_readonly: rgb(31, 35, 53),
    input_foreground: rgb(192, 202, 245),
    ring: rgb(122, 162, 247),
};

// Gruvbox Dark: bright orange emphasis on bg0, aqua-blue accent, retro warm text.
const PALETTE_GRUVBOX_DARK: Palette = Palette {
    background: rgb(40, 40, 40),
    foreground: rgb(235, 219, 178),
    card: rgb(60, 56, 54),
    card_foreground: rgb(235, 219, 178),
    popover: rgb(60, 56, 54),
    popover_foreground: rgb(235, 219, 178),
    primary: rgb(254, 128, 25),
    primary_foreground: rgb(40, 40, 40),
    secondary: rgb(60, 56, 54),
    secondary_foreground: rgb(235, 219, 178),
    muted: rgb(60, 56, 54),
    muted_foreground: rgb(168, 153, 132),
    accent: rgb(131, 165, 152),
    accent_foreground: rgb(40, 40, 40),
    destructive: rgb(251, 73, 52),
    border: rgb(80, 73, 69),
    input: rgb(29, 32, 33),
    input_focus: rgb(24, 26, 27),
    input_readonly: rgb(50, 48, 47),
    input_foreground: rgb(235, 219, 178),
    ring: rgb(254, 128, 25),
};

// Gruvbox Light (light): orange emphasis on the cream bg0, dark warm text.
const PALETTE_GRUVBOX_LIGHT: Palette = Palette {
    background: rgb(251, 241, 199),
    foreground: rgb(60, 56, 54),
    card: rgb(235, 219, 178),
    card_foreground: rgb(60, 56, 54),
    popover: rgb(235, 219, 178),
    popover_foreground: rgb(60, 56, 54),
    primary: rgb(214, 93, 14),
    primary_foreground: rgb(251, 241, 199),
    secondary: rgb(235, 219, 178),
    secondary_foreground: rgb(60, 56, 54),
    muted: rgb(235, 219, 178),
    muted_foreground: rgb(124, 111, 100),
    accent: rgb(69, 133, 136),
    accent_foreground: rgb(251, 241, 199),
    destructive: rgb(204, 36, 29),
    border: rgb(213, 196, 161),
    input: rgb(249, 245, 215),
    input_focus: rgb(255, 255, 255),
    input_readonly: rgb(235, 219, 178),
    input_foreground: rgb(60, 56, 54),
    ring: rgb(214, 93, 14),
};

// Solarized Dark: blue accent on base03, cyan accent, base0 body text.
const PALETTE_SOLARIZED_DARK: Palette = Palette {
    background: rgb(0, 43, 54),
    foreground: rgb(131, 148, 150),
    card: rgb(7, 54, 66),
    card_foreground: rgb(131, 148, 150),
    popover: rgb(7, 54, 66),
    popover_foreground: rgb(131, 148, 150),
    primary: rgb(38, 139, 210),
    primary_foreground: rgb(253, 246, 227),
    secondary: rgb(7, 54, 66),
    secondary_foreground: rgb(147, 161, 161),
    muted: rgb(7, 54, 66),
    muted_foreground: rgb(88, 110, 117),
    accent: rgb(42, 161, 152),
    accent_foreground: rgb(253, 246, 227),
    destructive: rgb(220, 50, 47),
    border: rgb(7, 54, 66),
    input: rgb(7, 54, 66),
    input_focus: rgb(0, 36, 46),
    input_readonly: rgb(7, 54, 66),
    input_foreground: rgb(147, 161, 161),
    ring: rgb(38, 139, 210),
};

// Solarized Light (light): blue accent on base3, cyan accent, base00 body text.
const PALETTE_SOLARIZED_LIGHT: Palette = Palette {
    background: rgb(253, 246, 227),
    foreground: rgb(101, 123, 131),
    card: rgb(238, 232, 213),
    card_foreground: rgb(101, 123, 131),
    popover: rgb(238, 232, 213),
    popover_foreground: rgb(101, 123, 131),
    primary: rgb(38, 139, 210),
    primary_foreground: rgb(253, 246, 227),
    secondary: rgb(238, 232, 213),
    secondary_foreground: rgb(101, 123, 131),
    muted: rgb(238, 232, 213),
    muted_foreground: rgb(147, 161, 161),
    accent: rgb(42, 161, 152),
    accent_foreground: rgb(253, 246, 227),
    destructive: rgb(220, 50, 47),
    border: rgb(219, 211, 186),
    input: rgb(253, 246, 227),
    input_focus: rgb(255, 255, 255),
    input_readonly: rgb(238, 232, 213),
    input_foreground: rgb(101, 123, 131),
    ring: rgb(38, 139, 210),
};

// Rosé Pine: iris primary on the #191724 base, rose accent, subtle muted text.
const PALETTE_ROSE_PINE: Palette = Palette {
    background: rgb(25, 23, 36),
    foreground: rgb(224, 222, 244),
    card: rgb(31, 29, 46),
    card_foreground: rgb(224, 222, 244),
    popover: rgb(31, 29, 46),
    popover_foreground: rgb(224, 222, 244),
    primary: rgb(196, 167, 231),
    primary_foreground: rgb(25, 23, 36),
    secondary: rgb(38, 35, 58),
    secondary_foreground: rgb(224, 222, 244),
    muted: rgb(38, 35, 58),
    muted_foreground: rgb(144, 140, 170),
    accent: rgb(235, 188, 186),
    accent_foreground: rgb(25, 23, 36),
    destructive: rgb(235, 111, 146),
    border: rgb(64, 61, 82),
    input: rgb(33, 32, 46),
    input_focus: rgb(26, 25, 38),
    input_readonly: rgb(38, 35, 58),
    input_foreground: rgb(224, 222, 244),
    ring: rgb(196, 167, 231),
};

// Rosé Pine Dawn (light): iris primary on the #faf4ed base, rose accent, dark text.
const PALETTE_ROSE_PINE_DAWN: Palette = Palette {
    background: rgb(250, 244, 237),
    foreground: rgb(87, 82, 121),
    card: rgb(242, 233, 225),
    card_foreground: rgb(87, 82, 121),
    popover: rgb(242, 233, 225),
    popover_foreground: rgb(87, 82, 121),
    primary: rgb(144, 122, 169),
    primary_foreground: rgb(250, 244, 237),
    secondary: rgb(242, 233, 225),
    secondary_foreground: rgb(87, 82, 121),
    muted: rgb(242, 233, 225),
    muted_foreground: rgb(121, 117, 147),
    accent: rgb(215, 130, 126),
    accent_foreground: rgb(250, 244, 237),
    destructive: rgb(180, 99, 122),
    border: rgb(223, 218, 217),
    input: rgb(255, 250, 243),
    input_focus: rgb(255, 255, 255),
    input_readonly: rgb(242, 233, 225),
    input_foreground: rgb(87, 82, 121),
    ring: rgb(144, 122, 169),
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Theme {
    Default,
    Alchemy,
    Gemini,
    Heretic,
    Ionic,
    CatppuccinLatte,
    CatppuccinFrappe,
    CatppuccinMacchiato,
    CatppuccinMocha,
    Dracula,
    Nord,
    TokyoNight,
    GruvboxDark,
    GruvboxLight,
    SolarizedDark,
    SolarizedLight,
    RosePine,
    RosePineDawn,
}

impl Theme {
    pub fn palette(self) -> Palette {
        match self {
            Theme::Default => PALETTE_DEFAULT,
            Theme::Alchemy => PALETTE_ALCHEMY,
            Theme::Gemini => PALETTE_GEMINI,
            Theme::Heretic => PALETTE_HERETIC,
            Theme::Ionic => PALETTE_IONIC,
            Theme::CatppuccinLatte => PALETTE_CATPPUCCIN_LATTE,
            Theme::CatppuccinFrappe => PALETTE_CATPPUCCIN_FRAPPE,
            Theme::CatppuccinMacchiato => PALETTE_CATPPUCCIN_MACCHIATO,
            Theme::CatppuccinMocha => PALETTE_CATPPUCCIN_MOCHA,
            Theme::Dracula => PALETTE_DRACULA,
            Theme::Nord => PALETTE_NORD,
            Theme::TokyoNight => PALETTE_TOKYO_NIGHT,
            Theme::GruvboxDark => PALETTE_GRUVBOX_DARK,
            Theme::GruvboxLight => PALETTE_GRUVBOX_LIGHT,
            Theme::SolarizedDark => PALETTE_SOLARIZED_DARK,
            Theme::SolarizedLight => PALETTE_SOLARIZED_LIGHT,
            Theme::RosePine => PALETTE_ROSE_PINE,
            Theme::RosePineDawn => PALETTE_ROSE_PINE_DAWN,
        }
    }

    pub fn group(self) -> &'static str {
        match self {
            Theme::Default | Theme::Alchemy | Theme::Gemini | Theme::Heretic | Theme::Ionic => {
                "Classic"
            }
            Theme::CatppuccinLatte
            | Theme::CatppuccinFrappe
            | Theme::CatppuccinMacchiato
            | Theme::CatppuccinMocha => "Catppuccin",
            Theme::Dracula => "Dracula",
            Theme::Nord => "Nord",
            Theme::TokyoNight => "Tokyo Night",
            Theme::GruvboxDark | Theme::GruvboxLight => "Gruvbox",
            Theme::SolarizedDark | Theme::SolarizedLight => "Solarized",
            Theme::RosePine | Theme::RosePineDawn => "Rosé Pine",
        }
    }

    pub fn dark(self) -> bool {
        !matches!(
            self,
            Theme::CatppuccinLatte
                | Theme::GruvboxLight
                | Theme::SolarizedLight
                | Theme::RosePineDawn
        )
    }
}

pub fn themes() -> &'static [(&'static str, Theme)] {
    &[
        ("Default", Theme::Default),
        ("Alchemy", Theme::Alchemy),
        ("Gemini", Theme::Gemini),
        ("Heretic", Theme::Heretic),
        ("Ionic", Theme::Ionic),
        ("Catppuccin Latte", Theme::CatppuccinLatte),
        ("Catppuccin Frappé", Theme::CatppuccinFrappe),
        ("Catppuccin Macchiato", Theme::CatppuccinMacchiato),
        ("Catppuccin Mocha", Theme::CatppuccinMocha),
        ("Dracula", Theme::Dracula),
        ("Nord", Theme::Nord),
        ("Tokyo Night", Theme::TokyoNight),
        ("Gruvbox Dark", Theme::GruvboxDark),
        ("Gruvbox Light", Theme::GruvboxLight),
        ("Solarized Dark", Theme::SolarizedDark),
        ("Solarized Light", Theme::SolarizedLight),
        ("Rosé Pine", Theme::RosePine),
        ("Rosé Pine Dawn", Theme::RosePineDawn),
    ]
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum RadiusPref {
    None,
    Small,
    Medium,
    Large,
}

impl RadiusPref {
    pub fn token(self) -> BorderRadius {
        match self {
            RadiusPref::None => BorderRadius::None,
            RadiusPref::Small => BorderRadius::Sm,
            RadiusPref::Medium => BorderRadius::Md,
            RadiusPref::Large => BorderRadius::Lg,
        }
    }

    pub fn px(self) -> f32 {
        radius(self.token())
    }
}

pub fn radii() -> &'static [(&'static str, RadiusPref)] {
    &[
        ("None", RadiusPref::None),
        ("Small", RadiusPref::Small),
        ("Medium", RadiusPref::Medium),
        ("Large", RadiusPref::Large),
    ]
}

#[allow(clippy::cast_precision_loss)]
pub fn space(token: Spacing) -> f32 {
    token.to_px().unwrap_or(0) as f32
}

#[allow(clippy::cast_possible_truncation)]
pub fn round_i8(px: f32) -> i8 {
    px.round().clamp(f32::from(i8::MIN), f32::from(i8::MAX)) as i8
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub fn round_u8(px: f32) -> u8 {
    px.round().clamp(0.0, f32::from(u8::MAX)) as u8
}

// Hook for a future "UI text size" preference.
const FONT_SCALE: f32 = 1.0;

// Our own compact scale (Base = 12, ~2px per step), not twill's Tailwind sizes.
pub fn font(token: FontSize) -> f32 {
    let px = match token {
        FontSize::Xs => 8.0,
        FontSize::Sm => 10.0,
        FontSize::Base => 12.0,
        FontSize::Lg => 14.0,
        FontSize::Xl => 16.0,
        FontSize::S2xl => 20.0,
        FontSize::S3xl => 24.0,
        FontSize::S4xl => 30.0,
        _ => 36.0,
    };
    px * FONT_SCALE
}

pub fn radius(token: BorderRadius) -> f32 {
    to_corner_radius(token)
}

pub const fn border_width(token: BorderWidth) -> f32 {
    match token {
        BorderWidth::S0 => 0.0,
        BorderWidth::S1 => 1.0,
        BorderWidth::S2 => 2.0,
        BorderWidth::S4 => 4.0,
        BorderWidth::S8 => 8.0,
    }
}

pub fn sidebar_w() -> f32 {
    space(Spacing::S72)
}
pub fn field_w() -> f32 {
    space(Spacing::S56)
}
pub fn field_h() -> f32 {
    space(Spacing::S7)
}
pub fn field_pad_x() -> f32 {
    space(Spacing::S2)
}

fn palette_id() -> egui::Id {
    egui::Id::new("alchemy_active_palette")
}
fn radius_id() -> egui::Id {
    egui::Id::new("alchemy_active_radius")
}

pub fn active(ctx: &egui::Context) -> Palette {
    ctx.data(|d| d.get_temp::<Palette>(palette_id()))
        .unwrap_or(PALETTE_DEFAULT)
}

pub fn active_radius(ctx: &egui::Context) -> RadiusPref {
    ctx.data(|d| d.get_temp::<RadiusPref>(radius_id()))
        .unwrap_or(RadiusPref::Small)
}

pub fn corner(ctx: &egui::Context) -> egui::CornerRadius {
    egui::CornerRadius::same(round_u8(active_radius(ctx).px()))
}

// A borderless frame with the standard S1 border stroke and active corner radius. Callers
// chain `.fill(...)` / `.inner_margin(...)` for filled or padded variants.
pub fn bordered_frame(ctx: &egui::Context) -> egui::Frame {
    egui::Frame::NONE
        .stroke(egui::Stroke::new(
            border_width(BorderWidth::S1),
            active(ctx).border,
        ))
        .corner_radius(corner(ctx))
}

pub fn mix(a: Color32, b: Color32, t: f32) -> Color32 {
    let l = |x: u8, y: u8| round_u8(f32::from(x) + (f32::from(y) - f32::from(x)) * t);
    Color32::from_rgb(l(a.r(), b.r()), l(a.g(), b.g()), l(a.b(), b.b()))
}

const WINDOW_INACTIVE_OPACITY: f32 = 0.9;

pub fn window_inactive(ctx: &egui::Context, id: egui::Id) -> bool {
    ctx.top_layer_id() != Some(egui::LayerId::new(egui::Order::Middle, id))
}

pub fn window_frame(ctx: &egui::Context, inactive: bool) -> egui::Frame {
    let mut frame = egui::Frame::window(&ctx.style());
    if inactive {
        frame.fill = frame.fill.gamma_multiply(WINDOW_INACTIVE_OPACITY);
        frame.stroke.color = frame.stroke.color.gamma_multiply(WINDOW_INACTIVE_OPACITY);
    }
    frame
}

pub fn window_content_opacity(inactive: bool) -> f32 {
    if inactive {
        WINDOW_INACTIVE_OPACITY
    } else {
        1.0
    }
}

pub fn install(ctx: &egui::Context, theme: Theme, radius: RadiusPref) {
    let p = theme.palette();
    ctx.data_mut(|d| {
        d.insert_temp(palette_id(), p);
        d.insert_temp(radius_id(), radius);
    });

    let mut visuals = if theme.dark() {
        egui::Visuals::dark()
    } else {
        egui::Visuals::light()
    };
    visuals.panel_fill = p.background;
    visuals.window_fill = p.popover;
    visuals.extreme_bg_color = p.input;
    visuals.override_text_color = Some(p.foreground);
    visuals.hyperlink_color = p.primary;
    visuals.selection.bg_fill = p.ring.gamma_multiply(0.5);
    visuals.selection.stroke = egui::Stroke::new(border_width(BorderWidth::S1), p.ring);

    visuals.popup_shadow = egui::epaint::Shadow {
        offset: [0, 2],
        blur: 6,
        spread: 0,
        color: Color32::from_black_alpha(50),
    };
    visuals.window_shadow = egui::epaint::Shadow {
        offset: [0, 4],
        blur: 12,
        spread: 0,
        color: Color32::from_black_alpha(60),
    };

    let corner = egui::CornerRadius::same(round_u8(radius.px()));
    let stroke = egui::Stroke::new(border_width(BorderWidth::S1), p.border);

    visuals.window_fill = p.popover;
    visuals.window_stroke = egui::Stroke::new(border_width(BorderWidth::S1), p.border);

    visuals.widgets.noninteractive.corner_radius = corner;
    visuals.widgets.noninteractive.bg_fill = p.card;
    visuals.widgets.noninteractive.weak_bg_fill = p.card;

    visuals.widgets.inactive.corner_radius = corner;
    visuals.widgets.inactive.bg_fill = p.secondary;
    visuals.widgets.inactive.weak_bg_fill = p.secondary;
    visuals.widgets.inactive.bg_stroke = stroke;

    visuals.widgets.hovered.corner_radius = corner;
    visuals.widgets.hovered.bg_fill = mix(p.secondary, p.foreground, 0.12);
    visuals.widgets.hovered.weak_bg_fill = mix(p.secondary, p.foreground, 0.12);
    visuals.widgets.hovered.bg_stroke = stroke;

    visuals.widgets.active.corner_radius = corner;
    visuals.widgets.active.bg_fill = mix(p.secondary, p.foreground, 0.2);
    visuals.widgets.active.weak_bg_fill = mix(p.secondary, p.foreground, 0.2);
    visuals.widgets.active.bg_stroke = egui::Stroke::new(border_width(BorderWidth::S1), p.ring);

    visuals.widgets.open.corner_radius = corner;
    visuals.widgets.open.bg_fill = p.secondary;
    visuals.widgets.open.weak_bg_fill = p.secondary;
    visuals.widgets.open.bg_stroke = stroke;

    // Don't grow widgets on hover/press; the close X looks jittery when it expands.
    visuals.widgets.hovered.expansion = 0.0;
    visuals.widgets.active.expansion = 0.0;

    let mut style = (*ctx.style()).clone();
    style.spacing.button_padding.x = field_pad_x();
    let body = egui::FontId::proportional(font(FontSize::Base));
    style
        .text_styles
        .insert(egui::TextStyle::Body, body.clone());
    style.text_styles.insert(egui::TextStyle::Button, body);
    style.spacing.menu_margin = egui::Margin::symmetric(0, round_i8(space(Spacing::S1_5)));
    // Title-bar height is title height plus this margin's vertical sum, so a slim top margin trims the bar.
    style.spacing.window_margin = egui::Margin {
        left: round_i8(space(Spacing::S2)),
        right: round_i8(space(Spacing::S2)),
        top: round_i8(space(Spacing::S1)),
        bottom: round_i8(space(Spacing::S2)),
    };
    style.visuals = visuals;

    ctx.set_style(style);
}
