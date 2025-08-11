use iced::{
    alignment, theme, widget::{button, column, container, row, scrollable, text, text_input},
    Command, Element, Length,
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::config::Config;

#[derive(Debug, Clone)]
pub enum Message {
    NavigateTo(PathBuf),
    GoUp,
    CreateFolder,
    CreateFile,
    UpdateNewItemName(String),
    ConfirmCreate,
    CancelCreate,
    SelectItem(usize),
    OpenItem(PathBuf),
    RefreshView,
    Close,
}

#[derive(Debug, Clone)]
pub struct FileItem {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub size: u64,
}

#[derive(Debug, Clone, PartialEq)]
enum CreateMode {
    None,
    File,
    Folder,
}

pub struct FileManager {
    config: Config,
    current_path: PathBuf,
    items: Vec<FileItem>,
    selected_index: Option<usize>,
    create_mode: CreateMode,
    new_item_name: String,
    error_message: Option<String>,
}

impl FileManager {
    pub fn new(config: Config) -> Self {
        let default_path = PathBuf::from(&config.applications.file_manager.default_path);
        let current_path = if default_path.exists() {
            default_path
        } else {
            dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"))
        };

        let mut manager = Self {
            config,
            current_path: current_path.clone(),
            items: Vec::new(),
            selected_index: None,
            create_mode: CreateMode::None,
            new_item_name: String::new(),
            error_message: None,
        };

        manager.load_directory(current_path.clone());
        manager
    }

    pub fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::NavigateTo(path) => {
                if path.is_dir() && path.exists() {
                    self.current_path = path.clone();
                    self.load_directory(path);
                    self.selected_index = None;
                    self.error_message = None;
                }
                Command::none()
            }
            Message::GoUp => {
                if let Some(parent) = self.current_path.parent() {
                    self.current_path = parent.to_path_buf();
                    let path = self.current_path.clone();
                    self.load_directory(path);
                    self.selected_index = None;
                    self.error_message = None;
                }
                Command::none()
            }
            Message::CreateFolder => {
                self.create_mode = CreateMode::Folder;
                self.new_item_name.clear();
                self.error_message = None;
                Command::none()
            }
            Message::CreateFile => {
                self.create_mode = CreateMode::File;
                self.new_item_name.clear();
                self.error_message = None;
                Command::none()
            }
            Message::UpdateNewItemName(name) => {
                self.new_item_name = name;
                Command::none()
            }
            Message::ConfirmCreate => {
                if !self.new_item_name.is_empty() {
                    let new_path = self.current_path.join(&self.new_item_name);

                    let result = match self.create_mode {
                        CreateMode::File => fs::write(&new_path, ""),
                        CreateMode::Folder => fs::create_dir(&new_path),
                        CreateMode::None => Ok(()),
                    };

                    match result {
                        Ok(_) => {
                            let path = self.current_path.clone();
                            self.load_directory(path);
                            self.create_mode = CreateMode::None;
                            self.new_item_name.clear();
                            self.error_message = None;
                        }
                        Err(e) => {
                            self.error_message = Some(format!("Error: {}", e));
                        }
                    }
                }
                Command::none()
            }
            Message::CancelCreate => {
                self.create_mode = CreateMode::None;
                self.new_item_name.clear();
                self.error_message = None;
                Command::none()
            }
            Message::SelectItem(index) => {
                self.selected_index = Some(index);
                Command::none()
            }
            Message::OpenItem(path) => {
                if path.is_dir() {
                    self.current_path = path.clone();
                    self.load_directory(path);
                    self.selected_index = None;
                }
                Command::none()
            }
            Message::RefreshView => {
                let path = self.current_path.clone();
                self.load_directory(path);
                Command::none()
            }
            Message::Close => {
                Command::none()
            }
        }
    }

    pub fn view(&self) -> Element<Message> {
        let header = self.create_header();
        let toolbar = self.create_toolbar();
        let content = self.create_content();
        let status_bar = self.create_status_bar();

        let main_content = column![
            header,
            toolbar,
            content,
            status_bar,
        ]
        .spacing(5)
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

        let path_display = text(self.current_path.display().to_string())
            .size(14)
            .style(theme::Text::Color(iced::Color::from_rgb(0.7, 0.7, 0.7)));

        row![
            text("📁 File Manager").size(18),
            row![].width(Length::Fill),
            path_display,
            close_btn,
        ]
        .spacing(10)
        .align_items(alignment::Alignment::Center)
        .into()
    }

    fn create_toolbar(&self) -> Element<Message> {
        let up_btn = button(text("⬆ Up"))
            .on_press(Message::GoUp)
            .padding(8);

        let refresh_btn = button(text("🔄 Refresh"))
            .on_press(Message::RefreshView)
            .padding(8);

        let new_folder_btn = button(text("📁+ New Folder"))
            .on_press(Message::CreateFolder)
            .padding(8);

        let new_file_btn = button(text("📄+ New File"))
            .on_press(Message::CreateFile)
            .padding(8);

        let mut toolbar = row![
            up_btn,
            refresh_btn,
            new_folder_btn,
            new_file_btn,
        ]
        .spacing(10);

        // Add create input if in create mode
        if self.create_mode != CreateMode::None {
            let placeholder = match self.create_mode {
                CreateMode::File => "Enter file name...",
                CreateMode::Folder => "Enter folder name...",
                CreateMode::None => "",
            };

            let input = text_input(placeholder, &self.new_item_name)
                .on_input(Message::UpdateNewItemName)
                .on_submit(Message::ConfirmCreate)
                .padding(5)
                .width(Length::Fixed(200.0));

            let confirm_btn = button(text("✓"))
                .on_press(Message::ConfirmCreate)
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
                });

            let cancel_btn = button(text("✗"))
                .on_press(Message::CancelCreate)
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
                });

            toolbar = toolbar.push(row![].width(Length::Fixed(20.0)));
            toolbar = toolbar.push(input);
            toolbar = toolbar.push(confirm_btn);
            toolbar = toolbar.push(cancel_btn);
        }

        container(toolbar)
            .padding(10)
            .style(|theme: &iced::Theme| {
                let palette = theme.extended_palette();
                container::Appearance {
                    background: Some(iced::Background::Color(palette.background.weak.color)),
                    border: iced::Border::with_radius(4),
                    ..Default::default()
                }
            })
            .into()
    }

    fn create_content(&self) -> Element<Message> {
        let mut items_column = column![].spacing(2);

        for (index, item) in self.items.iter().enumerate() {
            let icon = if item.is_dir { "📁" } else { "📄" };
            let size_text = if item.is_dir {
                String::new()
            } else {
                format_file_size(item.size)
            };

            let is_selected = self.selected_index == Some(index);

            let item_button = button(
                row![
                    text(format!("{} {}", icon, item.name)).size(14),
                    row![].width(Length::Fill),
                    text(size_text).size(12).style(theme::Text::Color(iced::Color::from_rgb(0.6, 0.6, 0.6))),
                ]
                .spacing(10)
                .align_items(alignment::Alignment::Center)
            )
            .on_press(Message::SelectItem(index))
            .width(Length::Fill)
            .padding(8)
            .style(move |theme: &iced::Theme, status| {
                let palette = theme.extended_palette();
                button::Appearance {
                    background: Some(iced::Background::Color(
                        if is_selected {
                            iced::Color::from_rgba8(100, 150, 255, 0.3)
                        } else if matches!(status, button::Status::Hovered) {
                            iced::Color::from_rgba8(255, 255, 255, 0.1)
                        } else {
                            iced::Color::TRANSPARENT
                        }
                    )),
                    border: iced::Border::with_radius(4),
                    text_color: palette.background.base.text,
                    ..Default::default()
                }
            });

            let item_row = if item.is_dir {
                button(
                    row![
                        text(format!("{} {}", icon, item.name)).size(14),
                        row![].width(Length::Fill),
                        text(size_text).size(12).style(theme::Text::Color(iced::Color::from_rgb(0.6, 0.6, 0.6))),
                    ]
                    .spacing(10)
                    .align_items(alignment::Alignment::Center)
                )
                .on_press(Message::OpenItem(item.path.clone()))
                .width(Length::Fill)
                .padding(8)
                .style(move |theme: &iced::Theme, status| {
                    let palette = theme.extended_palette();
                    button::Appearance {
                        background: Some(iced::Background::Color(
                            if is_selected {
                                iced::Color::from_rgba8(100, 150, 255, 0.3)
                            } else if matches!(status, button::Status::Hovered) {
                                iced::Color::from_rgba8(255, 255, 255, 0.1)
                            } else {
                                iced::Color::TRANSPARENT
                            }
                        )),
                        border: iced::Border::with_radius(4),
                        text_color: palette.background.base.text,
                        ..Default::default()
                    }
                })
            } else {
                item_button
            };

            items_column = items_column.push(item_row);
        }

        let scrollable_content = scrollable(items_column)
            .width(Length::Fill)
            .height(Length::Fill);

        container(scrollable_content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(5)
            .style(|theme: &iced::Theme| {
                let palette = theme.extended_palette();
                container::Appearance {
                    background: Some(iced::Background::Color(palette.background.weak.color)),
                    border: iced::Border {
                        color: palette.background.strong.color,
                        width: 1.0,
                        radius: 4.0.into(),
                    },
                    ..Default::default()
                }
            })
            .into()
    }

    fn create_status_bar(&self) -> Element<Message> {
        let status_text = if let Some(error) = &self.error_message {
            text(error).style(theme::Text::Color(iced::Color::from_rgb(1.0, 0.4, 0.4)))
        } else {
            text(format!("{} items", self.items.len()))
                .style(theme::Text::Color(iced::Color::from_rgb(0.7, 0.7, 0.7)))
        };

        container(status_text.size(12))
            .padding(5)
            .width(Length::Fill)
            .into()
    }

    fn load_directory(&mut self, path: PathBuf) {
        self.items.clear();

        if let Ok(entries) = fs::read_dir(&path) {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    let name = entry.file_name().to_string_lossy().to_string();

                    // Skip hidden files unless configured to show them
                    if name.starts_with('.') {
                        continue;
                    }

                    self.items.push(FileItem {
                        name,
                        path: entry.path(),
                        is_dir: metadata.is_dir(),
                        size: metadata.len(),
                    });
                }
            }
        }

        // Sort: directories first, then alphabetically
        self.items.sort_by(|a, b| {
            match (a.is_dir, b.is_dir) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            }
        });
    }
}

fn format_file_size(size: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut size = size as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", size as u64, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}
