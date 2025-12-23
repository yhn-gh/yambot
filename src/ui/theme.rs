use eframe::egui::{self, Color32};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Theme variants for the UI
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ThemeKind {
    Twilight,
    MaterialDark,
    Amoled,
    CatppuccinMocha,
    NeonCyber,
    NeonPurple,
    MatrixGreen,
    MinimalGray,
    ForestDark,
    LightElegant,
}

impl Default for ThemeKind {
    fn default() -> Self {
        ThemeKind::Twilight
    }
}

impl fmt::Display for ThemeKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            ThemeKind::Twilight => "Twilight",
            ThemeKind::MaterialDark => "MaterialDark",
            ThemeKind::Amoled => "Amoled",
            ThemeKind::CatppuccinMocha => "CatppuccinMocha",
            ThemeKind::NeonCyber => "NeonCyber",
            ThemeKind::NeonPurple => "NeonPurple",
            ThemeKind::MatrixGreen => "MatrixGreen",
            ThemeKind::MinimalGray => "MinimalGray",
            ThemeKind::ForestDark => "ForestDark",
            ThemeKind::LightElegant => "LightElegant",
        };
        write!(f, "{}", name)
    }
}

impl ThemeKind {
    /// Parse a theme from a string name
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "Twilight" => Some(ThemeKind::Twilight),
            "MaterialDark" => Some(ThemeKind::MaterialDark),
            "Amoled" => Some(ThemeKind::Amoled),
            "CatppuccinMocha" => Some(ThemeKind::CatppuccinMocha),
            "NeonCyber" => Some(ThemeKind::NeonCyber),
            "NeonPurple" => Some(ThemeKind::NeonPurple),
            "MatrixGreen" => Some(ThemeKind::MatrixGreen),
            "MinimalGray" => Some(ThemeKind::MinimalGray),
            "ForestDark" => Some(ThemeKind::ForestDark),
            "LightElegant" => Some(ThemeKind::LightElegant),
            _ => None,
        }
    }

    /// Get human-readable display name
    pub fn display_name(&self) -> &'static str {
        match self {
            ThemeKind::Twilight => "Twilight",
            ThemeKind::MaterialDark => "Material Dark",
            ThemeKind::Amoled => "AMOLED",
            ThemeKind::CatppuccinMocha => "Catppuccin Mocha",
            ThemeKind::NeonCyber => "Neon Cyber",
            ThemeKind::NeonPurple => "Neon Purple",
            ThemeKind::MatrixGreen => "Matrix Green",
            ThemeKind::MinimalGray => "Minimal Gray",
            ThemeKind::ForestDark => "Forest Dark",
            ThemeKind::LightElegant => "Light Elegant",
        }
    }

    /// Get all available themes
    pub fn all() -> [ThemeKind; 10] {
        [
            ThemeKind::Twilight,
            ThemeKind::MaterialDark,
            ThemeKind::Amoled,
            ThemeKind::CatppuccinMocha,
            ThemeKind::NeonCyber,
            ThemeKind::NeonPurple,
            ThemeKind::MatrixGreen,
            ThemeKind::MinimalGray,
            ThemeKind::ForestDark,
            ThemeKind::LightElegant,
        ]
    }
}

/// Apply a theme to the egui context
pub fn apply_theme(ctx: &egui::Context, theme: ThemeKind) {
    match theme {
        ThemeKind::Twilight => apply_twilight_theme(ctx),
        ThemeKind::MaterialDark => apply_material_theme(ctx),
        ThemeKind::Amoled => apply_amoled_theme(ctx),
        ThemeKind::CatppuccinMocha => apply_catppuccin_mocha_theme(ctx),
        ThemeKind::NeonCyber => apply_neon_cyber_theme(ctx),
        ThemeKind::NeonPurple => apply_neon_purple_theme(ctx),
        ThemeKind::MatrixGreen => apply_matrix_green_theme(ctx),
        ThemeKind::MinimalGray => apply_minimal_gray_theme(ctx),
        ThemeKind::ForestDark => apply_forest_dark_theme(ctx),
        ThemeKind::LightElegant => apply_light_elegant_theme(ctx),
    }
}

fn apply_twilight_theme(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    style.visuals = egui::Visuals {
        dark_mode: true,
        override_text_color: Some(Color32::from_rgb(220, 220, 235)),
        faint_bg_color: Color32::from_rgb(30, 30, 48),
        extreme_bg_color: Color32::from_rgb(24, 24, 36),
        window_fill: Color32::from_rgb(28, 28, 40),
        panel_fill: Color32::from_rgb(32, 32, 48),
        window_shadow: egui::epaint::Shadow {
            offset: [0, 8],
            blur: 16,
            spread: 0,
            color: Color32::from_black_alpha(96),
        },
        widgets: egui::style::Widgets {
            noninteractive: egui::style::WidgetVisuals {
                bg_fill: Color32::from_rgb(40, 40, 60),
                bg_stroke: egui::Stroke::new(1.0, Color32::from_rgb(60, 60, 90)),
                fg_stroke: egui::Stroke::new(1.0, Color32::WHITE),
                corner_radius: egui::CornerRadius::same(8),
                expansion: 0.0,
                weak_bg_fill: Color32::from_rgb(40, 40, 60),
            },
            inactive: egui::style::WidgetVisuals {
                bg_fill: Color32::from_rgb(50, 50, 70),
                bg_stroke: egui::Stroke::new(1.0, Color32::from_rgb(80, 80, 110)),
                fg_stroke: egui::Stroke::new(1.0, Color32::WHITE),
                corner_radius: egui::CornerRadius::same(8),
                expansion: 0.0,
                weak_bg_fill: Color32::from_rgb(50, 50, 70),
            },
            hovered: egui::style::WidgetVisuals {
                bg_fill: Color32::from_rgb(65, 65, 95),
                bg_stroke: egui::Stroke::new(1.0, Color32::from_rgb(120, 120, 160)),
                fg_stroke: egui::Stroke::new(1.5, Color32::WHITE),
                corner_radius: egui::CornerRadius::same(8),
                expansion: 0.0,
                weak_bg_fill: Color32::from_rgb(65, 65, 95),
            },
            active: egui::style::WidgetVisuals {
                bg_fill: Color32::from_rgb(80, 80, 120),
                bg_stroke: egui::Stroke::new(1.5, Color32::from_rgb(160, 160, 200)),
                fg_stroke: egui::Stroke::new(1.5, Color32::WHITE),
                corner_radius: egui::CornerRadius::same(8),
                expansion: 0.0,
                weak_bg_fill: Color32::from_rgb(80, 80, 120),
            },
            open: egui::style::WidgetVisuals {
                bg_fill: Color32::from_rgb(70, 70, 110),
                bg_stroke: egui::Stroke::new(1.0, Color32::from_rgb(130, 130, 170)),
                fg_stroke: egui::Stroke::new(1.0, Color32::WHITE),
                corner_radius: egui::CornerRadius::same(8),
                expansion: 0.0,
                weak_bg_fill: Color32::from_rgb(70, 70, 110),
            },
        },
        ..Default::default()
    };

    style.spacing.item_spacing = egui::vec2(8.0, 8.0);
    style.spacing.button_padding = egui::vec2(8.0, 6.0);
    style.spacing.window_margin = egui::Margin::same(8);

    ctx.set_style(style);
}

fn apply_material_theme(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    style.visuals = egui::Visuals::dark();
    style.visuals.override_text_color = Some(Color32::from_rgb(235, 235, 240));
    style.visuals.faint_bg_color = Color32::from_rgb(28, 28, 28);
    style.visuals.extreme_bg_color = Color32::from_rgb(14, 14, 18);
    style.visuals.window_fill = Color32::from_rgb(20, 20, 24);
    style.visuals.panel_fill = Color32::from_rgb(24, 24, 30);

    style.visuals.widgets.inactive.bg_fill = Color32::from_rgb(40, 40, 40);
    style.visuals.widgets.hovered.bg_fill = Color32::from_rgb(64, 80, 100);
    style.visuals.widgets.active.bg_fill = Color32::from_rgb(76, 96, 130);

    style.spacing.item_spacing = egui::vec2(8.0, 8.0);
    style.spacing.button_padding = egui::vec2(8.0, 6.0);
    style.spacing.window_margin = egui::Margin::same(8);
    ctx.set_style(style);
}

fn apply_amoled_theme(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    style.visuals = egui::Visuals::dark();
    style.visuals.override_text_color = Some(Color32::from_rgb(200, 200, 200));
    style.visuals.faint_bg_color = Color32::from_rgb(0, 0, 0);
    style.visuals.extreme_bg_color = Color32::from_rgb(0, 0, 0);
    style.visuals.window_fill = Color32::from_rgb(0, 0, 0);
    style.visuals.panel_fill = Color32::from_rgb(6, 6, 6);

    style.visuals.widgets.inactive.bg_fill = Color32::from_rgb(12, 12, 12);
    style.visuals.widgets.hovered.bg_fill = Color32::from_rgb(40, 40, 40);
    style.visuals.widgets.active.bg_fill = Color32::from_rgb(80, 80, 80);

    style.spacing.item_spacing = egui::vec2(8.0, 8.0);
    style.spacing.button_padding = egui::vec2(8.0, 6.0);
    style.spacing.window_margin = egui::Margin::same(8);
    ctx.set_style(style);
}

fn apply_catppuccin_mocha_theme(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    style.visuals = egui::Visuals {
        dark_mode: true,
        override_text_color: Some(Color32::from_rgb(205, 214, 244)), // Text
        faint_bg_color: Color32::from_rgb(30, 30, 46),               // Base
        extreme_bg_color: Color32::from_rgb(17, 17, 27),             // Crust
        window_fill: Color32::from_rgb(24, 24, 37),                  // Mantle
        panel_fill: Color32::from_rgb(30, 30, 46),                   // Base
        window_shadow: egui::epaint::Shadow {
            offset: [0, 8],
            blur: 16,
            spread: 0,
            color: Color32::from_black_alpha(120),
        },
        widgets: egui::style::Widgets {
            noninteractive: egui::style::WidgetVisuals {
                bg_fill: Color32::from_rgb(49, 50, 68), // Surface0
                bg_stroke: egui::Stroke::new(1.0, Color32::from_rgb(88, 91, 112)), // Surface2
                fg_stroke: egui::Stroke::new(1.0, Color32::from_rgb(205, 214, 244)),
                corner_radius: egui::CornerRadius::same(8),
                expansion: 0.0,
                weak_bg_fill: Color32::from_rgb(49, 50, 68),
            },
            inactive: egui::style::WidgetVisuals {
                bg_fill: Color32::from_rgb(49, 50, 68), // Surface0
                bg_stroke: egui::Stroke::new(1.0, Color32::from_rgb(116, 199, 236)), // Sky
                fg_stroke: egui::Stroke::new(1.0, Color32::from_rgb(205, 214, 244)),
                corner_radius: egui::CornerRadius::same(8),
                expansion: 0.0,
                weak_bg_fill: Color32::from_rgb(49, 50, 68),
            },
            hovered: egui::style::WidgetVisuals {
                bg_fill: Color32::from_rgb(88, 91, 112), // Surface2
                bg_stroke: egui::Stroke::new(1.5, Color32::from_rgb(137, 180, 250)), // Blue
                fg_stroke: egui::Stroke::new(1.5, Color32::from_rgb(205, 214, 244)),
                corner_radius: egui::CornerRadius::same(8),
                expansion: 0.0,
                weak_bg_fill: Color32::from_rgb(88, 91, 112),
            },
            active: egui::style::WidgetVisuals {
                bg_fill: Color32::from_rgb(137, 180, 250), // Blue
                bg_stroke: egui::Stroke::new(1.5, Color32::from_rgb(180, 190, 254)), // Lavender
                fg_stroke: egui::Stroke::new(1.5, Color32::from_rgb(30, 30, 46)),
                corner_radius: egui::CornerRadius::same(8),
                expansion: 0.0,
                weak_bg_fill: Color32::from_rgb(137, 180, 250),
            },
            open: egui::style::WidgetVisuals {
                bg_fill: Color32::from_rgb(69, 71, 90), // Surface1
                bg_stroke: egui::Stroke::new(1.0, Color32::from_rgb(137, 180, 250)),
                fg_stroke: egui::Stroke::new(1.0, Color32::from_rgb(205, 214, 244)),
                corner_radius: egui::CornerRadius::same(8),
                expansion: 0.0,
                weak_bg_fill: Color32::from_rgb(69, 71, 90),
            },
        },
        ..Default::default()
    };

    style.spacing.item_spacing = egui::vec2(8.0, 8.0);
    style.spacing.button_padding = egui::vec2(8.0, 6.0);
    style.spacing.window_margin = egui::Margin::same(8);
    ctx.set_style(style);
}

fn apply_neon_cyber_theme(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    style.visuals = egui::Visuals {
        dark_mode: true,
        override_text_color: Some(Color32::from_rgb(0, 255, 255)), // Neon cyan text
        faint_bg_color: Color32::from_rgb(10, 10, 25),             // Deep dark blue
        extreme_bg_color: Color32::from_rgb(5, 5, 15),
        window_fill: Color32::from_rgb(10, 10, 25),
        panel_fill: Color32::from_rgb(15, 15, 30),
        window_shadow: egui::epaint::Shadow {
            offset: [0, 10],
            blur: 20,
            spread: 2,
            color: Color32::from_rgba_premultiplied(0, 255, 255, 80), // Cyan glow
        },
        widgets: egui::style::Widgets {
            noninteractive: egui::style::WidgetVisuals {
                bg_fill: Color32::from_rgb(20, 20, 40),
                bg_stroke: egui::Stroke::new(2.0, Color32::from_rgb(0, 200, 255)),
                fg_stroke: egui::Stroke::new(1.0, Color32::from_rgb(0, 255, 255)),
                corner_radius: egui::CornerRadius::same(4),
                expansion: 0.0,
                weak_bg_fill: Color32::from_rgb(20, 20, 40),
            },
            inactive: egui::style::WidgetVisuals {
                bg_fill: Color32::from_rgb(20, 20, 40),
                bg_stroke: egui::Stroke::new(2.0, Color32::from_rgb(255, 0, 255)), // Magenta
                fg_stroke: egui::Stroke::new(1.0, Color32::from_rgb(0, 255, 255)),
                corner_radius: egui::CornerRadius::same(4),
                expansion: 0.0,
                weak_bg_fill: Color32::from_rgb(20, 20, 40),
            },
            hovered: egui::style::WidgetVisuals {
                bg_fill: Color32::from_rgb(30, 30, 60),
                bg_stroke: egui::Stroke::new(3.0, Color32::from_rgb(0, 255, 255)), // Bright cyan
                fg_stroke: egui::Stroke::new(2.0, Color32::from_rgb(255, 255, 255)),
                corner_radius: egui::CornerRadius::same(4),
                expansion: 1.0,
                weak_bg_fill: Color32::from_rgb(30, 30, 60),
            },
            active: egui::style::WidgetVisuals {
                bg_fill: Color32::from_rgb(0, 200, 255), // Neon cyan fill
                bg_stroke: egui::Stroke::new(3.0, Color32::from_rgb(255, 255, 255)),
                fg_stroke: egui::Stroke::new(2.0, Color32::from_rgb(10, 10, 25)),
                corner_radius: egui::CornerRadius::same(4),
                expansion: 1.0,
                weak_bg_fill: Color32::from_rgb(0, 200, 255),
            },
            open: egui::style::WidgetVisuals {
                bg_fill: Color32::from_rgb(25, 25, 50),
                bg_stroke: egui::Stroke::new(2.0, Color32::from_rgb(0, 255, 255)),
                fg_stroke: egui::Stroke::new(1.0, Color32::from_rgb(0, 255, 255)),
                corner_radius: egui::CornerRadius::same(4),
                expansion: 0.0,
                weak_bg_fill: Color32::from_rgb(25, 25, 50),
            },
        },
        ..Default::default()
    };

    style.spacing.item_spacing = egui::vec2(8.0, 8.0);
    style.spacing.button_padding = egui::vec2(8.0, 6.0);
    style.spacing.window_margin = egui::Margin::same(8);
    ctx.set_style(style);
}

fn apply_neon_purple_theme(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    style.visuals = egui::Visuals {
        dark_mode: true,
        override_text_color: Some(Color32::from_rgb(220, 160, 255)), // Neon purple text
        faint_bg_color: Color32::from_rgb(18, 10, 30),               // Deep purple-black
        extreme_bg_color: Color32::from_rgb(10, 5, 20),
        window_fill: Color32::from_rgb(18, 10, 30),
        panel_fill: Color32::from_rgb(25, 15, 40),
        window_shadow: egui::epaint::Shadow {
            offset: [0, 10],
            blur: 25,
            spread: 3,
            color: Color32::from_rgba_premultiplied(180, 0, 255, 100), // Purple glow
        },
        widgets: egui::style::Widgets {
            noninteractive: egui::style::WidgetVisuals {
                bg_fill: Color32::from_rgb(30, 20, 50),
                bg_stroke: egui::Stroke::new(2.0, Color32::from_rgb(150, 80, 255)),
                fg_stroke: egui::Stroke::new(1.0, Color32::from_rgb(220, 160, 255)),
                corner_radius: egui::CornerRadius::same(6),
                expansion: 0.0,
                weak_bg_fill: Color32::from_rgb(30, 20, 50),
            },
            inactive: egui::style::WidgetVisuals {
                bg_fill: Color32::from_rgb(30, 20, 50),
                bg_stroke: egui::Stroke::new(2.0, Color32::from_rgb(255, 0, 200)), // Hot pink
                fg_stroke: egui::Stroke::new(1.0, Color32::from_rgb(220, 160, 255)),
                corner_radius: egui::CornerRadius::same(6),
                expansion: 0.0,
                weak_bg_fill: Color32::from_rgb(30, 20, 50),
            },
            hovered: egui::style::WidgetVisuals {
                bg_fill: Color32::from_rgb(60, 30, 90),
                bg_stroke: egui::Stroke::new(3.0, Color32::from_rgb(200, 100, 255)),
                fg_stroke: egui::Stroke::new(2.0, Color32::from_rgb(255, 255, 255)),
                corner_radius: egui::CornerRadius::same(6),
                expansion: 2.0,
                weak_bg_fill: Color32::from_rgb(60, 30, 90),
            },
            active: egui::style::WidgetVisuals {
                bg_fill: Color32::from_rgb(180, 80, 255), // Bright neon purple
                bg_stroke: egui::Stroke::new(3.0, Color32::from_rgb(255, 150, 255)),
                fg_stroke: egui::Stroke::new(2.0, Color32::from_rgb(18, 10, 30)),
                corner_radius: egui::CornerRadius::same(6),
                expansion: 2.0,
                weak_bg_fill: Color32::from_rgb(180, 80, 255),
            },
            open: egui::style::WidgetVisuals {
                bg_fill: Color32::from_rgb(40, 25, 65),
                bg_stroke: egui::Stroke::new(2.0, Color32::from_rgb(150, 80, 255)),
                fg_stroke: egui::Stroke::new(1.0, Color32::from_rgb(220, 160, 255)),
                corner_radius: egui::CornerRadius::same(6),
                expansion: 0.0,
                weak_bg_fill: Color32::from_rgb(40, 25, 65),
            },
        },
        ..Default::default()
    };

    style.spacing.item_spacing = egui::vec2(8.0, 8.0);
    style.spacing.button_padding = egui::vec2(8.0, 6.0);
    style.spacing.window_margin = egui::Margin::same(8);
    ctx.set_style(style);
}

fn apply_matrix_green_theme(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    style.visuals = egui::Visuals {
        dark_mode: true,
        override_text_color: Some(Color32::from_rgb(0, 255, 65)), // Matrix green
        faint_bg_color: Color32::from_rgb(0, 0, 0),               // Pure black
        extreme_bg_color: Color32::from_rgb(0, 0, 0),
        window_fill: Color32::from_rgb(0, 0, 0),
        panel_fill: Color32::from_rgb(5, 10, 5),
        window_shadow: egui::epaint::Shadow {
            offset: [0, 8],
            blur: 20,
            spread: 2,
            color: Color32::from_rgba_premultiplied(0, 255, 65, 60), // Green glow
        },
        widgets: egui::style::Widgets {
            noninteractive: egui::style::WidgetVisuals {
                bg_fill: Color32::from_rgb(0, 15, 0),
                bg_stroke: egui::Stroke::new(1.5, Color32::from_rgb(0, 180, 50)),
                fg_stroke: egui::Stroke::new(1.0, Color32::from_rgb(0, 255, 65)),
                corner_radius: egui::CornerRadius::same(2),
                expansion: 0.0,
                weak_bg_fill: Color32::from_rgb(0, 15, 0),
            },
            inactive: egui::style::WidgetVisuals {
                bg_fill: Color32::from_rgb(0, 15, 0),
                bg_stroke: egui::Stroke::new(1.5, Color32::from_rgb(0, 255, 100)),
                fg_stroke: egui::Stroke::new(1.0, Color32::from_rgb(0, 255, 65)),
                corner_radius: egui::CornerRadius::same(2),
                expansion: 0.0,
                weak_bg_fill: Color32::from_rgb(0, 15, 0),
            },
            hovered: egui::style::WidgetVisuals {
                bg_fill: Color32::from_rgb(0, 30, 10),
                bg_stroke: egui::Stroke::new(2.0, Color32::from_rgb(0, 255, 65)),
                fg_stroke: egui::Stroke::new(1.5, Color32::from_rgb(150, 255, 150)),
                corner_radius: egui::CornerRadius::same(2),
                expansion: 0.0,
                weak_bg_fill: Color32::from_rgb(0, 30, 10),
            },
            active: egui::style::WidgetVisuals {
                bg_fill: Color32::from_rgb(0, 200, 50), // Bright green
                bg_stroke: egui::Stroke::new(2.5, Color32::from_rgb(100, 255, 100)),
                fg_stroke: egui::Stroke::new(2.0, Color32::from_rgb(0, 0, 0)),
                corner_radius: egui::CornerRadius::same(2),
                expansion: 0.0,
                weak_bg_fill: Color32::from_rgb(0, 200, 50),
            },
            open: egui::style::WidgetVisuals {
                bg_fill: Color32::from_rgb(0, 20, 5),
                bg_stroke: egui::Stroke::new(1.5, Color32::from_rgb(0, 255, 65)),
                fg_stroke: egui::Stroke::new(1.0, Color32::from_rgb(0, 255, 65)),
                corner_radius: egui::CornerRadius::same(2),
                expansion: 0.0,
                weak_bg_fill: Color32::from_rgb(0, 20, 5),
            },
        },
        ..Default::default()
    };

    style.spacing.item_spacing = egui::vec2(8.0, 8.0);
    style.spacing.button_padding = egui::vec2(8.0, 6.0);
    style.spacing.window_margin = egui::Margin::same(8);
    ctx.set_style(style);
}

fn apply_minimal_gray_theme(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    style.visuals = egui::Visuals {
        dark_mode: true,
        override_text_color: Some(Color32::from_rgb(230, 230, 230)), // Light gray text
        faint_bg_color: Color32::from_rgb(45, 45, 45),
        extreme_bg_color: Color32::from_rgb(35, 35, 35),
        window_fill: Color32::from_rgb(40, 40, 40),
        panel_fill: Color32::from_rgb(48, 48, 48),
        window_shadow: egui::epaint::Shadow {
            offset: [0, 4],
            blur: 10,
            spread: 0,
            color: Color32::from_black_alpha(80),
        },
        widgets: egui::style::Widgets {
            noninteractive: egui::style::WidgetVisuals {
                bg_fill: Color32::from_rgb(60, 60, 60),
                bg_stroke: egui::Stroke::new(1.0, Color32::from_rgb(80, 80, 80)),
                fg_stroke: egui::Stroke::new(1.0, Color32::from_rgb(230, 230, 230)),
                corner_radius: egui::CornerRadius::same(3),
                expansion: 0.0,
                weak_bg_fill: Color32::from_rgb(60, 60, 60),
            },
            inactive: egui::style::WidgetVisuals {
                bg_fill: Color32::from_rgb(65, 65, 65),
                bg_stroke: egui::Stroke::new(1.0, Color32::from_rgb(100, 100, 100)),
                fg_stroke: egui::Stroke::new(1.0, Color32::from_rgb(230, 230, 230)),
                corner_radius: egui::CornerRadius::same(3),
                expansion: 0.0,
                weak_bg_fill: Color32::from_rgb(65, 65, 65),
            },
            hovered: egui::style::WidgetVisuals {
                bg_fill: Color32::from_rgb(85, 85, 85),
                bg_stroke: egui::Stroke::new(1.0, Color32::from_rgb(140, 140, 140)),
                fg_stroke: egui::Stroke::new(1.0, Color32::from_rgb(255, 255, 255)),
                corner_radius: egui::CornerRadius::same(3),
                expansion: 0.0,
                weak_bg_fill: Color32::from_rgb(85, 85, 85),
            },
            active: egui::style::WidgetVisuals {
                bg_fill: Color32::from_rgb(120, 120, 120),
                bg_stroke: egui::Stroke::new(1.0, Color32::from_rgb(180, 180, 180)),
                fg_stroke: egui::Stroke::new(1.0, Color32::from_rgb(255, 255, 255)),
                corner_radius: egui::CornerRadius::same(3),
                expansion: 0.0,
                weak_bg_fill: Color32::from_rgb(120, 120, 120),
            },
            open: egui::style::WidgetVisuals {
                bg_fill: Color32::from_rgb(75, 75, 75),
                bg_stroke: egui::Stroke::new(1.0, Color32::from_rgb(100, 100, 100)),
                fg_stroke: egui::Stroke::new(1.0, Color32::from_rgb(230, 230, 230)),
                corner_radius: egui::CornerRadius::same(3),
                expansion: 0.0,
                weak_bg_fill: Color32::from_rgb(75, 75, 75),
            },
        },
        ..Default::default()
    };

    style.spacing.item_spacing = egui::vec2(8.0, 8.0);
    style.spacing.button_padding = egui::vec2(8.0, 6.0);
    style.spacing.window_margin = egui::Margin::same(8);
    ctx.set_style(style);
}

fn apply_forest_dark_theme(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    style.visuals = egui::Visuals {
        dark_mode: true,
        override_text_color: Some(Color32::from_rgb(200, 230, 180)), // Soft green-white
        faint_bg_color: Color32::from_rgb(20, 30, 20),               // Deep forest
        extreme_bg_color: Color32::from_rgb(10, 15, 10),
        window_fill: Color32::from_rgb(20, 30, 20),
        panel_fill: Color32::from_rgb(28, 40, 28),
        window_shadow: egui::epaint::Shadow {
            offset: [0, 6],
            blur: 14,
            spread: 0,
            color: Color32::from_black_alpha(120),
        },
        widgets: egui::style::Widgets {
            noninteractive: egui::style::WidgetVisuals {
                bg_fill: Color32::from_rgb(35, 50, 35),
                bg_stroke: egui::Stroke::new(1.0, Color32::from_rgb(60, 85, 60)),
                fg_stroke: egui::Stroke::new(1.0, Color32::from_rgb(200, 230, 180)),
                corner_radius: egui::CornerRadius::same(5),
                expansion: 0.0,
                weak_bg_fill: Color32::from_rgb(35, 50, 35),
            },
            inactive: egui::style::WidgetVisuals {
                bg_fill: Color32::from_rgb(35, 50, 35),
                bg_stroke: egui::Stroke::new(1.0, Color32::from_rgb(80, 140, 80)), // Medium green
                fg_stroke: egui::Stroke::new(1.0, Color32::from_rgb(200, 230, 180)),
                corner_radius: egui::CornerRadius::same(5),
                expansion: 0.0,
                weak_bg_fill: Color32::from_rgb(35, 50, 35),
            },
            hovered: egui::style::WidgetVisuals {
                bg_fill: Color32::from_rgb(50, 70, 50),
                bg_stroke: egui::Stroke::new(1.5, Color32::from_rgb(100, 180, 100)), // Bright green
                fg_stroke: egui::Stroke::new(1.5, Color32::from_rgb(220, 255, 200)),
                corner_radius: egui::CornerRadius::same(5),
                expansion: 0.0,
                weak_bg_fill: Color32::from_rgb(50, 70, 50),
            },
            active: egui::style::WidgetVisuals {
                bg_fill: Color32::from_rgb(70, 120, 70), // Forest green
                bg_stroke: egui::Stroke::new(2.0, Color32::from_rgb(120, 200, 120)),
                fg_stroke: egui::Stroke::new(2.0, Color32::from_rgb(20, 30, 20)),
                corner_radius: egui::CornerRadius::same(5),
                expansion: 0.0,
                weak_bg_fill: Color32::from_rgb(70, 120, 70),
            },
            open: egui::style::WidgetVisuals {
                bg_fill: Color32::from_rgb(40, 60, 40),
                bg_stroke: egui::Stroke::new(1.0, Color32::from_rgb(80, 140, 80)),
                fg_stroke: egui::Stroke::new(1.0, Color32::from_rgb(200, 230, 180)),
                corner_radius: egui::CornerRadius::same(5),
                expansion: 0.0,
                weak_bg_fill: Color32::from_rgb(40, 60, 40),
            },
        },
        ..Default::default()
    };

    style.spacing.item_spacing = egui::vec2(8.0, 8.0);
    style.spacing.button_padding = egui::vec2(8.0, 6.0);
    style.spacing.window_margin = egui::Margin::same(8);
    ctx.set_style(style);
}

fn apply_light_elegant_theme(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    style.visuals = egui::Visuals {
        dark_mode: false,
        override_text_color: Some(Color32::from_rgb(40, 40, 50)), // Dark blue-gray text
        faint_bg_color: Color32::from_rgb(248, 249, 252),         // Almost white
        extreme_bg_color: Color32::from_rgb(235, 238, 245),
        window_fill: Color32::from_rgb(255, 255, 255), // Pure white
        panel_fill: Color32::from_rgb(248, 249, 252),
        window_shadow: egui::epaint::Shadow {
            offset: [0, 2],
            blur: 8,
            spread: 0,
            color: Color32::from_rgba_premultiplied(0, 0, 50, 30), // Subtle shadow
        },
        widgets: egui::style::Widgets {
            noninteractive: egui::style::WidgetVisuals {
                bg_fill: Color32::from_rgb(240, 242, 248),
                bg_stroke: egui::Stroke::new(1.0, Color32::from_rgb(210, 215, 230)),
                fg_stroke: egui::Stroke::new(1.0, Color32::from_rgb(40, 40, 50)),
                corner_radius: egui::CornerRadius::same(6),
                expansion: 0.0,
                weak_bg_fill: Color32::from_rgb(240, 242, 248),
            },
            inactive: egui::style::WidgetVisuals {
                bg_fill: Color32::from_rgb(240, 242, 248),
                bg_stroke: egui::Stroke::new(1.0, Color32::from_rgb(120, 140, 200)), // Soft blue
                fg_stroke: egui::Stroke::new(1.0, Color32::from_rgb(40, 40, 50)),
                corner_radius: egui::CornerRadius::same(6),
                expansion: 0.0,
                weak_bg_fill: Color32::from_rgb(240, 242, 248),
            },
            hovered: egui::style::WidgetVisuals {
                bg_fill: Color32::from_rgb(230, 235, 245),
                bg_stroke: egui::Stroke::new(1.5, Color32::from_rgb(100, 130, 200)), // Medium blue
                fg_stroke: egui::Stroke::new(1.5, Color32::from_rgb(30, 30, 40)),
                corner_radius: egui::CornerRadius::same(6),
                expansion: 0.0,
                weak_bg_fill: Color32::from_rgb(230, 235, 245),
            },
            active: egui::style::WidgetVisuals {
                bg_fill: Color32::from_rgb(90, 120, 200), // Elegant blue
                bg_stroke: egui::Stroke::new(1.5, Color32::from_rgb(70, 100, 180)),
                fg_stroke: egui::Stroke::new(2.0, Color32::from_rgb(255, 255, 255)),
                corner_radius: egui::CornerRadius::same(6),
                expansion: 0.0,
                weak_bg_fill: Color32::from_rgb(90, 120, 200),
            },
            open: egui::style::WidgetVisuals {
                bg_fill: Color32::from_rgb(235, 238, 248),
                bg_stroke: egui::Stroke::new(1.0, Color32::from_rgb(120, 140, 200)),
                fg_stroke: egui::Stroke::new(1.0, Color32::from_rgb(40, 40, 50)),
                corner_radius: egui::CornerRadius::same(6),
                expansion: 0.0,
                weak_bg_fill: Color32::from_rgb(235, 238, 248),
            },
        },
        ..Default::default()
    };

    style.spacing.item_spacing = egui::vec2(8.0, 8.0);
    style.spacing.button_padding = egui::vec2(8.0, 6.0);
    style.spacing.window_margin = egui::Margin::same(8);
    ctx.set_style(style);
}
