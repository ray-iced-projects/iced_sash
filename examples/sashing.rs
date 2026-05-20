//! Example — sash widget feature demo.
//!
//! Demonstrates panel sync, outer resizing with mode selection,
//! and cross-size (height) sync across linked SashH widgets.

use iced::Length::Fill;
use iced::{Color, Element, Length, Theme};
use iced::widget::{center, checkbox, column, container, radio, row, text};

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
}

impl App {
    fn new() -> Self {
        Self {
            sizes: vec![200.0, 300.0, 200.0],
            sync_sashes: false,
            is_checked: false,
            height: 200.0,
            sync_cross_sashes: false,
            cross_is_checked: false,
            outer_mode: None,
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
                    iced_sash::resize(&mut self.sizes, index, size, 50.0);
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
            .min_size(50.0)
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
            .height(500.0)
            .spacing(20.0)
            .into();

        let controls = column(vec![
            sashes,
            chk,
            chk_cross,
            outer_mode_rads,
        ])
        .spacing(20.0)
        .width(750.0)
        .height(Fill)
        .into();

        row(vec![controls, sashv])
            .spacing(20.0)
            .padding(20.0)
            .into()
    }
}

