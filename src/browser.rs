use iced::{
    alignment, theme, widget::{button, column, container, row, text, text_input},
    Command, Element, Length,
};
use reqwest;
use std::sync::Arc;

use crate::config::Config;

#[derive(Debug, Clone)]
pub enum Message {
    Navigate(String),
    UpdateUrl(String),
    GoBack,
    GoForward,
    Refresh,
    LoadComplete(Result<String, String>),
    Close,
}

#[derive(Debug, Clone)]
struct WebPage {
    url: String,
    content: String,
}

pub struct Browser {
    config: Config,
    current_url: String,
    url_input: String,
    content: String,
    loading: bool,
    error: Option<String>,
    history: Vec<String>,
    history_index: usize,
}

impl Browser {
    pub fn new(config: Config) -> Self {
        let homepage = config.applications.browser.homepage.clone();
        Self {
            config,
            current_url: homepage.clone(),
            url_input: homepage.clone(),
            content: String::from("Welcome to MinDesk Browser\n\nEnter a URL above to start browsing."),
            loading: false,
            error: None,
            history: vec![homepage],
            history_index: 0,
        }
    }

    pub fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::UpdateUrl(url) => {
                self.url_input = url;
                Command::none()
            }
            Message::Navigate(url) => {
                let url = if !url.starts_with("http://") && !url.starts_with("https://") {
                    format!("https://{}", url)
                } else {
                    url
                };

                self.current_url = url.clone();
                self.url_input = url.clone();
                self.loading = true;
                self.error = None;

                // Update history
                if self.history_index < self.history.len() - 1 {
                    self.history.truncate(self.history_index + 1);
                }
                self.history.push(url.clone());
                self.history_index = self.history.len() - 1;

                Command::perform(fetch_page(url), Message::LoadComplete)
            }
            Message::GoBack => {
                if self.history_index > 0 {
                    self.history_index -= 1;
                    let url = self.history[self.history_index].clone();
                    self.current_url = url.clone();
                    self.url_input = url.clone();
                    self.loading = true;
                    Command::perform(fetch_page(url), Message::LoadComplete)
                } else {
                    Command::none()
                }
            }
            Message::GoForward => {
                if self.history_index < self.history.len() - 1 {
                    self.history_index += 1;
                    let url = self.history[self.history_index].clone();
                    self.current_url = url.clone();
                    self.url_input = url.clone();
                    self.loading = true;
                    Command::perform(fetch_page(url), Message::LoadComplete)
                } else {
                    Command::none()
                }
            }
            Message::Refresh => {
                self.loading = true;
                self.error = None;
                Command::perform(fetch_page(self.current_url.clone()), Message::LoadComplete)
            }
            Message::LoadComplete(result) => {
                self.loading = false;
                match result {
                    Ok(content) => {
                        self.content = content;
                        self.error = None;
                    }
                    Err(error) => {
                        self.error = Some(error);
                        self.content = String::new();
                    }
                }
                Command::none()
            }
            Message::Close => {
                Command::none()
            }
        }
    }

    pub fn view(&self) -> Element<Message> {
        let header = self.create_header();
        let navigation = self.create_navigation();
        let content = self.create_content();

        let main_content = column![
            header,
            navigation,
            content,
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

        row![
            text("🌐 Browser").size(18),
            row![].width(Length::Fill),
            close_btn,
        ]
        .spacing(10)
        .align_items(alignment::Alignment::Center)
        .into()
    }

    fn create_navigation(&self) -> Element<Message> {
        let back_btn = button(text("◀").size(16))
            .on_press(Message::GoBack)
            .padding(8)
            .style(|theme: &iced::Theme, status| {
                let palette = theme.extended_palette();
                let can_go_back = true; // You might want to check history here
                button::Appearance {
                    background: Some(iced::Background::Color(
                        if matches!(status, button::Status::Hovered) && can_go_back {
                            palette.background.weak.color
                        } else {
                            iced::Color::TRANSPARENT
                        }
                    )),
                    border: iced::Border::with_radius(4),
                    text_color: if can_go_back {
                        palette.background.base.text
                    } else {
                        iced::Color::from_rgba8(100, 100, 100, 0.5)
                    },
                    ..Default::default()
                }
            });

        let forward_btn = button(text("▶").size(16))
            .on_press(Message::GoForward)
            .padding(8)
            .style(|theme: &iced::Theme, status| {
                let palette = theme.extended_palette();
                let can_go_forward = true; // You might want to check history here
                button::Appearance {
                    background: Some(iced::Background::Color(
                        if matches!(status, button::Status::Hovered) && can_go_forward {
                            palette.background.weak.color
                        } else {
                            iced::Color::TRANSPARENT
                        }
                    )),
                    border: iced::Border::with_radius(4),
                    text_color: if can_go_forward {
                        palette.background.base.text
                    } else {
                        iced::Color::from_rgba8(100, 100, 100, 0.5)
                    },
                    ..Default::default()
                }
            });

        let refresh_btn = button(text("🔄").size(16))
            .on_press(Message::Refresh)
            .padding(8);

        let url_input = text_input(
            "Enter URL...",
            &self.url_input,
        )
        .on_input(Message::UpdateUrl)
        .on_submit(Message::Navigate(self.url_input.clone()))
        .padding(8)
        .size(14)
        .width(Length::Fill);

        let go_btn = button(text("Go").size(14))
            .on_press(Message::Navigate(self.url_input.clone()))
            .padding(8)
            .style(|theme: &iced::Theme, _| {
                button::Appearance {
                    background: Some(iced::Background::Color(
                        iced::Color::from_rgb(0.2, 0.4, 0.8)
                    )),
                    border: iced::Border::with_radius(4),
                    text_color: iced::Color::WHITE,
                    ..Default::default()
                }
            });

        container(
            row![
                back_btn,
                forward_btn,
                refresh_btn,
                url_input,
                go_btn,
            ]
            .spacing(5)
            .align_items(alignment::Alignment::Center)
        )
        .padding(5)
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
        if self.loading {
            return container(
                column![
                    text("Loading...").size(16),
                    text(&self.current_url)
                        .size(12)
                        .style(theme::Text::Color(iced::Color::from_rgb(0.6, 0.6, 0.6)))
                ]
                .spacing(10)
                .align_items(alignment::Alignment::Center)
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into();
        }

        if let Some(error) = &self.error {
            return container(
                column![
                    text("Error loading page").size(18)
                        .style(theme::Text::Color(iced::Color::from_rgb(1.0, 0.4, 0.4))),
                    text(error)
                        .size(14)
                        .style(theme::Text::Color(iced::Color::from_rgb(0.8, 0.3, 0.3))),
                    button(text("Retry").size(14))
                        .on_press(Message::Refresh)
                        .padding(10)
                ]
                .spacing(15)
                .align_items(alignment::Alignment::Center)
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into();
        }

        // Display content in a simple text format (minimal mode)
        let content_display = if self.config.applications.browser.minimal_mode {
            // In minimal mode, show plain text version
            container(
                iced::widget::scrollable(
                    container(
                        text(&self.content)
                            .size(14)
                            .style(theme::Text::Color(iced::Color::from_rgb(0.9, 0.9, 0.9)))
                    )
                    .padding(20)
                )
                .width(Length::Fill)
                .height(Length::Fill)
            )
        } else {
            // Normal mode would show rendered HTML (not implemented in minimal version)
            container(
                text("Full HTML rendering not available in minimal mode")
                    .size(14)
                    .style(theme::Text::Color(iced::Color::from_rgb(0.6, 0.6, 0.6)))
            )
            .center_x()
            .center_y()
        };

        container(content_display)
            .width(Length::Fill)
            .height(Length::Fill)
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
}

async fn fetch_page(url: String) -> Result<String, String> {
    // Create a client with minimal settings for Alpine compatibility
    let client = reqwest::Client::builder()
        .user_agent("MinDesk/1.0")
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch page: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("HTTP {}: {}",
            response.status().as_u16(),
            response.status().canonical_reason().unwrap_or("Unknown error")
        ));
    }

    let html = response
        .text()
        .await
        .map_err(|e| format!("Failed to read response: {}", e))?;

    // Extract text content from HTML (very basic extraction)
    Ok(extract_text_from_html(&html))
}

fn extract_text_from_html(html: &str) -> String {
    use html5ever::parse_document;
    use html5ever::tendril::TendrilSink;
    use markup5ever_rcdom::{Handle, NodeData, RcDom};

    let dom = parse_document(RcDom::default(), Default::default())
        .from_utf8()
        .read_from(&mut html.as_bytes())
        .unwrap();

    let mut text = String::new();
    extract_text_from_node(&dom.document, &mut text);
    text.trim().to_string()
}

fn extract_text_from_node(handle: &Handle, output: &mut String) {
    match handle.data {
        NodeData::Text { ref contents } => {
            let text = contents.borrow();
            if !text.trim().is_empty() {
                output.push_str(&text);
                output.push('\n');
            }
        }
        NodeData::Element { ref name, .. } => {
            // Skip script and style elements
            if name.local.as_ref() != "script" && name.local.as_ref() != "style" {
                for child in handle.children.borrow().iter() {
                    extract_text_from_node(child, output);
                }
            }
        }
        _ => {
            for child in handle.children.borrow().iter() {
                extract_text_from_node(child, output);
            }
        }
    }
}
