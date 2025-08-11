use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub desktop: DesktopConfig,
    pub applications: ApplicationsConfig,
    pub packages_to_install: Vec<String>,
    pub system: SystemConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DesktopConfig {
    pub wallpaper: String,
    pub font_name: String,
    pub font_size: u16,
    pub theme: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ApplicationsConfig {
    pub file_manager: AppConfig,
    pub package_manager: PackageManagerConfig,
    pub browser: BrowserConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AppConfig {
    pub enabled: bool,
    pub icon: String,
    pub default_path: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PackageManagerConfig {
    pub enabled: bool,
    pub icon: String,
    pub backend: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BrowserConfig {
    pub enabled: bool,
    pub icon: String,
    pub homepage: String,
    pub minimal_mode: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SystemConfig {
    pub dpi: u32,
    pub vsync: bool,
    pub compositor: bool,
}

impl Config {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = "/etc/min-desk/config.json";

        // Try system config first
        if Path::new(config_path).exists() {
            let contents = fs::read_to_string(config_path)?;
            let config: Config = serde_json::from_str(&contents)?;
            return Ok(config);
        }

        // Try local config
        if Path::new("config.json").exists() {
            let contents = fs::read_to_string("config.json")?;
            let config: Config = serde_json::from_str(&contents)?;
            return Ok(config);
        }

        // Return default config
        Ok(Config::default())
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            desktop: DesktopConfig {
                wallpaper: "/usr/share/backgrounds/default.png".to_string(),
                font_name: "Inter".to_string(),
                font_size: 12,
                theme: "dark".to_string(),
            },
            applications: ApplicationsConfig {
                file_manager: AppConfig {
                    enabled: true,
                    icon: "📁".to_string(),
                    default_path: "/home".to_string(),
                },
                package_manager: PackageManagerConfig {
                    enabled: true,
                    icon: "📦".to_string(),
                    backend: "apk".to_string(),
                },
                browser: BrowserConfig {
                    enabled: true,
                    icon: "🌐".to_string(),
                    homepage: "https://start.duckduckgo.com".to_string(),
                    minimal_mode: true,
                },
            },
            packages_to_install: vec![
                "firefox-esr".to_string(),
                "ttf-liberation".to_string(),
                "mesa-gl".to_string(),
                "mesa-dri-gallium".to_string(),
                "xf86-video-vesa".to_string(),
            ],
            system: SystemConfig {
                dpi: 96,
                vsync: true,
                compositor: false,
            },
        }
    }
}
