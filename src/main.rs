use iced::{
    alignment, executor, font, theme,
    widget::{button, column, container, row, text},
    Application, Command, Element, Length, Settings, Subscription, Theme,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

mod config;
mod file_manager;
mod package_manager;
mod browser;

use config::Config;
use file_manager::FileManager;
use package_manager::PackageManager;
use browser::Browser;

fn main() -> iced::Result {
    env_logger::init();
    MinDesk::run(Settings {
        window: iced::window::Settings {
            size: (1280, 720),
            decorations: false,
            transparent: true,
            ..Default::default()
        },
        antialiasing: true,
        ..Default::default()
    })
}

#[derive(Debug, Clone)]
pub enum Message {
    OpenFileManager,
    OpenPackageManager,
    OpenBrowser,
    FileManagerMessage(file_manager::Message),
    PackageManagerMessage(package_manager::Message),
    BrowserMessage(browser::Message),
    CloseApp(AppView),
    Tick,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppView {
    Desktop,
    FileManager,
    PackageManager,
    Browser,
}

pub struct MinDesk {
    config: Config,
    current_view: AppView,
    file_manager: FileManager,
    package_manager: PackageManager,
    browser: Browser,
}

impl Application for MinDesk {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        let config = Config::load().unwrap_or_default();

        (
            Self {
                config: config.clone(),
                current_view: AppView::Desktop,
                file_manager: FileManager::new(config.clone()),
                package_manager: PackageManager::new(config.clone()),
                browser: Browser::new(config.clone()),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("MinDesk")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::OpenFileManager => {
                self.current_view = AppView::FileManager;
                Command::none()
            }
            Message::OpenPackageManager => {
                self.current_view = AppView::PackageManager;
                Command::none()
            }
            Message::OpenBrowser => {
                self.current_view = AppView::Browser;
                Command::none()
            }
            Message::FileManagerMessage(msg) => {
                self.file_manager.update(msg).map(Message::FileManagerMessage)
            }
            Message::PackageManagerMessage(msg) => {
                self.package_manager.update(msg).map(Message::PackageManagerMessage)
            }
            Message::BrowserMessage(msg) => {
                self.browser.update(msg).map(Message::BrowserMessage)
            }
            Message::CloseApp(_) => {
                self.current_view = AppView::Desktop;
                Command::none()
            }
            Message::Tick => Command::none(),
        }
    }

    fn view(&self) -> Element<Message> {
        let content = match self.current_view {
            AppView::Desktop => self.desktop_view(),
            AppView::FileManager => self.file_manager.view()
                .map(Message::FileManagerMessage),
            AppView::PackageManager => self.package_manager.view()
                .map(Message::PackageManagerMessage),
            AppView::Browser => self.browser.view()
                .map(Message::BrowserMessage),
        };

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|theme: &Theme| {
                let palette = theme.extended_palette();
                container::Appearance {
                    background: Some(iced::Background::Color(palette.background.base.color)),
                    border: iced::Border::with_radius(0),
                    ..Default::default()
                }
            })
            .into()
    }

    fn theme(&self) -> Theme {
        if self.config.desktop.theme == "dark" {
            Theme::Dark
        } else {
            Theme::Light
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        iced::time::every(std::time::Duration::from_secs(1))
            .map(|_| Message::Tick)
    }
}

impl MinDesk {
    fn desktop_view(&self) -> Element<Message> {
        let title = text("MinDesk")
            .size(32)
            .style(theme::Text::Color(iced::Color::WHITE));

        let file_manager_btn = button(
            row![
                text(&self.config.applications.file_manager.icon).size(48),
                text("Files").size(16)
            ]
            .spacing(10)
            .align_items(alignment::Alignment::Center)
        )
        .on_press(Message::OpenFileManager)
        .padding(20)
        .style(|theme: &Theme, status| {
            let palette = theme.extended_palette();
            button::Appearance {
                background: Some(iced::Background::Color(
                    if matches!(status, button::Status::Hovered) {
                        iced::Color::from_rgba8(255, 255, 255, 0.1)
                    } else {
                        iced::Color::TRANSPARENT
                    }
                )),
                border: iced::Border::with_radius(8),
                text_color: palette.background.base.text,
                ..Default::default()
            }
        });

        let package_manager_btn = button(
            row![
                text(&self.config.applications.package_manager.icon).size(48),
                text("Packages").size(16)
            ]
            .spacing(10)
            .align_items(alignment::Alignment::Center)
        )
        .on_press(Message::OpenPackageManager)
        .padding(20)
        .style(|theme: &Theme, status| {
            let palette = theme.extended_palette();
            button::Appearance {
                background: Some(iced::Background::Color(
                    if matches!(status, button::Status::Hovered) {
                        iced::Color::from_rgba8(255, 255, 255, 0.1)
                    } else {
                        iced::Color::TRANSPARENT
                    }
                )),
                border: iced::Border::with_radius(8),
                text_color: palette.background.base.text,
                ..Default::default()
            }
        });

        let browser_btn = button(
            row![
                text(&self.config.applications.browser.icon).size(48),
                text("Browser").size(16)
            ]
            .spacing(10)
            .align_items(alignment::Alignment::Center)
        )
        .on_press(Message::OpenBrowser)
        .padding(20)
        .style(|theme: &Theme, status| {
            let palette = theme.extended_palette();
            button::Appearance {
                background: Some(iced::Background::Color(
                    if matches!(status, button::Status::Hovered) {
                        iced::Color::from_rgba8(255, 255, 255, 0.1)
                    } else {
                        iced::Color::TRANSPARENT
                    }
                )),
                border: iced::Border::with_radius(8),
                text_color: palette.background.base.text,
                ..Default::default()
            }
        });

        let apps = row![
            file_manager_btn,
            package_manager_btn,
            browser_btn
        ]
        .spacing(30);

        let content = column![
            title,
            apps
        ]
        .spacing(50)
        .align_items(alignment::Alignment::Center);

        // Simple dark background for now
        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .style(|theme: &Theme| {
                let palette = theme.extended_palette();
                container::Appearance {
                    background: Some(iced::Background::Color(iced::Color::from_rgb(0.1, 0.1, 0.15))),
                    ..Default::default()
                }
            })
            .into()
    }
}
