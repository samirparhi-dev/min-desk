use iced::{
    alignment, theme, widget::{button, column, container, row, scrollable, text, text_input},
    Command, Element, Length,
};
use std::process::Command as ProcessCommand;
use tokio::process::Command as TokioCommand;

use crate::config::Config;

#[derive(Debug, Clone)]
pub enum Message {
    Search(String),
    Install(String),
    Remove(String),
    UpdateCache,
    SearchResults(Vec<Package>),
    OperationComplete(String),
    OperationError(String),
    Close,
}

#[derive(Debug, Clone)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub description: String,
    pub installed: bool,
}

pub struct PackageManager {
    config: Config,
    search_query: String,
    packages: Vec<Package>,
    loading: bool,
    message: Option<String>,
    error: Option<String>,
}

impl PackageManager {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            search_query: String::new(),
            packages: Vec::new(),
            loading: false,
            message: None,
            error: None,
        }
    }

    pub fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Search(query) => {
                self.search_query = query.clone();
                if !query.is_empty() {
                    self.loading = true;
                    self.error = None;
                    Command::perform(search_packages(query), |result| {
                        match result {
                            Ok(packages) => Message::SearchResults(packages),
                            Err(e) => Message::OperationError(e),
                        }
                    })
                } else {
                    self.packages.clear();
                    Command::none()
                }
            }
            Message::SearchResults(packages) => {
                self.packages = packages;
                self.loading = false;
                Command::none()
            }
            Message::Install(package_name) => {
                self.loading = true;
                self.error = None;
                self.message = Some(format!("Installing {}...", package_name));
                Command::perform(install_package(package_name.clone()), |result| {
                    match result {
                        Ok(msg) => Message::OperationComplete(msg),
                        Err(e) => Message::OperationError(e),
                    }
                })
            }
            Message::Remove(package_name) => {
                self.loading = true;
                self.error = None;
                self.message = Some(format!("Removing {}...", package_name));
                Command::perform(remove_package(package_name.clone()), |result| {
                    match result {
                        Ok(msg) => Message::OperationComplete(msg),
                        Err(e) => Message::OperationError(e),
                    }
                })
            }
            Message::UpdateCache => {
                self.loading = true;
                self.error = None;
                self.message = Some("Updating package cache...".to_string());
                Command::perform(update_cache(), |result| {
                    match result {
                        Ok(msg) => Message::OperationComplete(msg),
                        Err(e) => Message::OperationError(e),
                    }
                })
            }
            Message::OperationComplete(msg) => {
                self.loading = false;
                self.message = Some(msg);
                self.error = None;
                // Refresh search after operation
                if !self.search_query.is_empty() {
                    Command::perform(search_packages(self.search_query.clone()), |result| {
                        match result {
                            Ok(packages) => Message::SearchResults(packages),
                            Err(e) => Message::OperationError(e),
                        }
                    })
                } else {
                    Command::none()
                }
            }
            Message::OperationError(error) => {
                self.loading = false;
                self.error = Some(error);
                self.message = None;
                Command::none()
            }
            Message::Close => {
                Command::none()
            }
        }
    }

    pub fn view(&self) -> Element<Message> {
        let header = self.create_header();
        let search_bar = self.create_search_bar();
        let content = self.create_content();
        let status_bar = self.create_status_bar();

        let main_content = column![
            header,
            search_bar,
            content,
            status_bar,
        ]
        .spacing(10)
        .padding(10);

        container(main_content)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|theme: &iced::Theme| {
                let palette = theme.extended_palette();
                container::Appearance {
                    background: Some(iced::Background::Color(palette.background.base.color)),
                    border: iced::Border::with_radius(0),
                    ..Default::default()
                }
            })
            .into()
    }

    fn create_header(&self) -> Element<Message> {
        let close_btn = button(text("✕").size(20))
            .on_press(Message::Close)
            .padding(5)
            .style(|theme: &iced::Theme, status| {
                let palette = theme.extended_palette();
                button::Appearance {
                    background: Some(iced::Background::Color(
                        if matches!(status, button::Status::Hovered) {
                            iced::Color::from_rgba8(255, 100, 100, 0.8)
                        } else {
                            iced::Color::TRANSPARENT
                        }
                    )),
                    border: iced::Border::with_radius(4),
                    text_color: palette.background.base.text,
                    ..Default::default()
                }
            });

        let update_btn = button(text("🔄 Update Cache"))
            .on_press(Message::UpdateCache)
            .padding(8);

        row![
            text("📦 Package Manager").size(18),
            row![].width(Length::Fill),
            update_btn,
            close_btn,
        ]
        .spacing(10)
        .align_items(alignment::Alignment::Center)
        .into()
    }

    fn create_search_bar(&self) -> Element<Message> {
        let search_input = text_input(
            "Search for packages...",
            &self.search_query,
        )
        .on_input(Message::Search)
        .padding(10)
        .size(16);

        container(search_input)
            .width(Length::Fill)
            .padding(5)
            .into()
    }

    fn create_content(&self) -> Element<Message> {
        if self.loading {
            return container(
                text("Loading...").size(16)
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into();
        }

        let mut packages_column = column![].spacing(5);

        for package in &self.packages {
            let install_btn = if package.installed {
                button(text("Remove").size(12))
                    .on_press(Message::Remove(package.name.clone()))
                    .padding(5)
                    .style(|theme: &iced::Theme, _| {
                        button::Appearance {
                            background: Some(iced::Background::Color(
                                iced::Color::from_rgb(0.6, 0.2, 0.2)
                            )),
                            border: iced::Border::with_radius(4),
                            text_color: iced::Color::WHITE,
                            ..Default::default()
                        }
                    })
            } else {
                button(text("Install").size(12))
                    .on_press(Message::Install(package.name.clone()))
                    .padding(5)
                    .style(|theme: &iced::Theme, _| {
                        button::Appearance {
                            background: Some(iced::Background::Color(
                                iced::Color::from_rgb(0.2, 0.6, 0.2)
                            )),
                            border: iced::Border::with_radius(4),
                            text_color: iced::Color::WHITE,
                            ..Default::default()
                        }
                    })
            };

            let status_indicator = if package.installed {
                text("✓").style(theme::Text::Color(iced::Color::from_rgb(0.2, 0.8, 0.2)))
            } else {
                text("")
            };

            let package_row = container(
                row![
                    status_indicator.width(Length::Fixed(20.0)),
                    column![
                        text(&package.name).size(14),
                        text(&package.description)
                            .size(12)
                            .style(theme::Text::Color(iced::Color::from_rgb(0.6, 0.6, 0.6))),
                    ].width(Length::Fill),
                    text(&package.version)
                        .size(12)
                        .style(theme::Text::Color(iced::Color::from_rgb(0.5, 0.5, 0.5))),
                    install_btn,
                ]
                .spacing(10)
                .align_items(alignment::Alignment::Center)
            )
            .padding(10)
            .style(|theme: &iced::Theme| {
                let palette = theme.extended_palette();
                container::Appearance {
                    background: Some(iced::Background::Color(palette.background.weak.color)),
                    border: iced::Border::with_radius(4),
                    ..Default::default()
                }
            });

            packages_column = packages_column.push(package_row);
        }

        if self.packages.is_empty() && !self.search_query.is_empty() {
            packages_column = packages_column.push(
                container(
                    text("No packages found").size(14)
                        .style(theme::Text::Color(iced::Color::from_rgb(0.6, 0.6, 0.6)))
                )
                .width(Length::Fill)
                .center_x()
                .padding(20)
            );
        }

        let scrollable_content = scrollable(packages_column)
            .width(Length::Fill)
            .height(Length::Fill);

        container(scrollable_content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn create_status_bar(&self) -> Element<Message> {
        let status_text = if let Some(error) = &self.error {
            text(error).style(theme::Text::Color(iced::Color::from_rgb(1.0, 0.4, 0.4)))
        } else if let Some(message) = &self.message {
            text(message).style(theme::Text::Color(iced::Color::from_rgb(0.4, 0.8, 0.4)))
        } else {
            text(format!("{} packages found", self.packages.len()))
                .style(theme::Text::Color(iced::Color::from_rgb(0.7, 0.7, 0.7)))
        };

        container(status_text.size(12))
            .padding(5)
            .width(Length::Fill)
            .into()
    }
}

async fn search_packages(query: String) -> Result<Vec<Package>, String> {
    let output = TokioCommand::new("apk")
        .args(&["search", "-v", &query])
        .output()
        .await
        .map_err(|e| format!("Failed to search packages: {}", e))?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut packages = Vec::new();

    for line in stdout.lines() {
        if let Some((name_version, _)) = line.split_once(" - ") {
            let (name, version) = if let Some(pos) = name_version.rfind('-') {
                (&name_version[..pos], &name_version[pos + 1..])
            } else {
                (name_version, "")
            };

            // Check if installed
            let installed = check_if_installed(name);

            packages.push(Package {
                name: name.to_string(),
                version: version.to_string(),
                description: line.split(" - ").nth(1).unwrap_or("").to_string(),
                installed,
            });
        }
    }

    Ok(packages)
}

async fn install_package(package_name: String) -> Result<String, String> {
    let output = TokioCommand::new("sudo")
        .args(&["apk", "add", &package_name])
        .output()
        .await
        .map_err(|e| format!("Failed to install package: {}", e))?;

    if output.status.success() {
        Ok(format!("Successfully installed {}", package_name))
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

async fn remove_package(package_name: String) -> Result<String, String> {
    let output = TokioCommand::new("sudo")
        .args(&["apk", "del", &package_name])
        .output()
        .await
        .map_err(|e| format!("Failed to remove package: {}", e))?;

    if output.status.success() {
        Ok(format!("Successfully removed {}", package_name))
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

async fn update_cache() -> Result<String, String> {
    let output = TokioCommand::new("sudo")
        .args(&["apk", "update"])
        .output()
        .await
        .map_err(|e| format!("Failed to update cache: {}", e))?;

    if output.status.success() {
        Ok("Package cache updated successfully".to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

fn check_if_installed(package_name: &str) -> bool {
    ProcessCommand::new("apk")
        .args(&["info", "-e", package_name])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}
