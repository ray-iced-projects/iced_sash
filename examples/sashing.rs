//! Example — sash widget feature demo.
//!
//! Demonstrates panel sync, outer resizing with mode selection,
//! and cross-size (height) sync across linked SashH widgets.

use iced::Length::Fill;
use iced::{Color, Element, Length, Theme};
use iced::widget::{center, checkbox, column, container, radio, row, text, text_input};

use iced_sash::{Id, OuterResizeMode, SashH, SashV};

pub fn main() -> iced::Result {
    iced::application(
        App::new,
        App::update,
        App::view)
        .title(App::title)
        .theme(Theme::TokyoNight)
        .centered()
        .run()
}

struct App {
    sizes: Vec<f32>,
    sync_sashes: bool,
    is_checked: bool,
    height: f32,
    sync_cross_sashes: bool,
    cross_is_checked: bool,
    outer_mode: Option<OuterResizeMode>,
    min_size: f32,
    max_size: Option<f32>,
    min_size_text: String,
    max_size_text: String,
    min_cross_size: f32,
    max_cross_size: Option<f32>,
    min_cross_size_text: String,
    max_cross_size_text: String,
}

impl App {
    fn new() -> Self {
        Self {
            sizes: vec![150.0, 200.0, 150.0],
            sync_sashes: false,
            is_checked: false,
            height: 200.0,
            sync_cross_sashes: false,
            cross_is_checked: false,
            outer_mode: None,
            min_size: 0.0,
            max_size: None,
            min_size_text: String::new(),
            max_size_text: String::new(),
            min_cross_size: 0.0,
            max_cross_size: None,
            min_cross_size_text: String::new(),
            max_cross_size_text: String::new(),
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    OnChecked(bool),
    OnCrossChecked(bool),
    ResizedH(Id, usize, f32),
    ResizedV(Id, usize, f32),
    ResizedCrossH(Id, f32),
    OuterResized(Id, f32),
    OnOuterMode(OuterResizeMode),
    
    SetMinText(String),
    SetMin,
    SetMaxText(String),
    SetMax,

    SetCrossMinText(String),
    SetCrossMin,
    SetCrossMaxText(String),
    SetCrossMax,
}

impl App {
    fn title(&self) -> String {
        String::from("Sash Example")
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::OnChecked(checked) => {
                self.is_checked = checked;
                self.sync_sashes = checked;
            }
            Message::OnCrossChecked(checked) => {
                self.cross_is_checked = checked;
                self.sync_cross_sashes = checked;
            }
            Message::ResizedH(_id, index, size) => {
                if self.sync_sashes {
                    iced_sash::resize(&mut self.sizes, index, size, self.min_size);
                }
            }
            Message::ResizedCrossH(_id, size) => {
                if self.sync_cross_sashes {
                    self.height = size;
                }
            }
            Message::OuterResized(_id, new_total) => {
                if self.sync_sashes {
                    iced_sash::apply_outer_resize(
                        &mut self.sizes,
                        new_total,
                        self.outer_mode.unwrap_or_default(),
                        50.0,
                    );
                }
            }
            Message::ResizedV(id, index, value) => {
                println!("{:?}, {}, {}", id, index, value)
            }
            Message::OnOuterMode(mode) => {
                self.outer_mode = Some(mode);
            }
            Message::SetMinText(value) => {
                self.min_size_text = value;
            }
            Message::SetMin => {
                self.min_size = self.min_size_text.parse().unwrap_or(0.0);
            }
            Message::SetMaxText(value) => {
                self.max_size_text = value;
            }
            Message::SetMax => {
                let max = self.max_size_text.parse().unwrap_or(0.0);
                self.max_size = if max > 0.0 { Some(max) } else { None };
            }
            Message::SetCrossMinText(value) => {
                self.min_cross_size_text = value;
            }
            Message::SetCrossMin => {
                self.min_cross_size = self.min_cross_size_text.parse().unwrap_or(0.0);
            }
            Message::SetCrossMaxText(value) => {
                self.max_cross_size_text = value;
            }
            Message::SetCrossMax => {
                let max = self.max_cross_size_text.parse().unwrap_or(0.0);
                self.max_cross_size = if max > 0.0 { Some(max) } else { None };
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {

        let panel = |label: &'static str, color: Color| -> Element<'_, Message> {
            container(center(text(label)))
                .width(Length::Fill)
                .height(Length::Fill)
                .style(move |_theme| iced::widget::container::Style {
                    background: Some(color.into()),
                    ..Default::default()
                })
                .into()
        };

        let make_sash = || -> Element<'_, Message> {
            let sh = SashH::new(
                vec![
                    panel("Left",   Color::from_rgb(0.25, 0.35, 0.55)),
                    panel("Center", Color::from_rgb(0.30, 0.50, 0.35)),
                    panel("Right",  Color::from_rgb(0.55, 0.35, 0.25)),
                ],
                self.sizes.clone(),
                self.height,
                4.0,
            )
            .min_size(self.min_size)
            .max_size_maybe(self.max_size)
            .min_cross_size(self.min_cross_size)
            .max_cross_size_maybe(self.max_cross_size)
            .on_resize(Message::ResizedH)
            .outer_handle(6.0)
            .outer_resize_mode(self.outer_mode.unwrap_or_default())
            .on_outer_resize(Message::OuterResized)
            .cross_handle(4.0)
            .on_cross_resize(Message::ResizedCrossH);

            let sh = if self.sync_sashes {
                sh.sync_sashes(self.sizes.clone())
            } else { sh };

            let sh = if self.sync_cross_sashes {
                sh.sync_cross_sashes(self.height)
            } else { sh };

            sh.into()
        };

        let sashv = SashV::new(
            vec![
                panel("Top",    Color::from_rgb(0.25, 0.35, 0.55)),
                panel("Center", Color::from_rgb(0.30, 0.50, 0.35)),
                panel("Bottom", Color::from_rgb(0.55, 0.35, 0.25)),
            ],
            self.sizes.clone(),
            200.0,
            4.0,
        )
        .min_size(50.0)
        .on_resize(Message::ResizedV)
        .style(|theme, status| iced_sash::primary(theme, status))
        .cross_handle(4.0)
        .into();

        let chk = checkbox(self.is_checked)
            .label("Sync SashesH")
            .on_toggle(Message::OnChecked)
            .into();

        let chk_cross = checkbox(self.cross_is_checked)
            .label("Sync Cross Sashes")
            .on_toggle(Message::OnCrossChecked)
            .into();

        let outer_mode_rads = column(vec![
            text("Outer resize mode:").into(),
            radio("Last Only",    OuterResizeMode::LastOnly,     self.outer_mode, Message::OnOuterMode).into(),
            radio("Uniform",      OuterResizeMode::Uniform,      self.outer_mode, Message::OnOuterMode).into(),
            radio("Proportional", OuterResizeMode::Proportional, self.outer_mode, Message::OnOuterMode).into(),
        ])
        .spacing(8.0)
        .into();

        let sashes = 
            column(vec![make_sash(), make_sash()])
            .height(450.0)
            .spacing(20.0)
            .into();
        
        
        let min_max = row(
            vec![
                text("Set min size").into(),
                text_input("", &self.min_size_text)
                    .on_input(Message::SetMinText)
                    .on_submit(Message::SetMin)
                    .width(100.0)
                    .into(),
                text("Set max size").into(),
                text_input("", &self.max_size_text)
                    .on_input(Message::SetMaxText)
                    .on_submit(Message::SetMax)
                    .width(100.0)
                    .into(),
                text("Press Enter to submit").into()
                ]
        ).spacing(10.0).into();

        let cross_min_max = row(
            vec![
                text("Set cross min size").into(),
                text_input("", &self.min_cross_size_text)
                    .on_input(Message::SetCrossMinText)
                    .on_submit(Message::SetCrossMin)
                    .width(100.0)
                    .into(),
                text("Set cross max size").into(),
                text_input("", &self.max_cross_size_text)
                    .on_input(Message::SetCrossMaxText)
                    .on_submit(Message::SetCrossMax)
                    .width(100.0)
                    .into(),
                text("Press Enter to submit").into()
                ]
        ).spacing(10.0).into();

        let controls = column(vec![
            sashes,
            chk,
            chk_cross,
            outer_mode_rads,
            min_max,
            cross_min_max,
        ])
        .spacing(10.0)
        .width(750.0)
        .height(Fill)
        .into();

        row(vec![controls, sashv])
            .spacing(20.0)
            .padding(20.0)
            .into()
    }
}

