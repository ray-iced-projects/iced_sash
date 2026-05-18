//! Example — two linked SashH widgets.
//!
//! Dragging a handle in either sash updates both simultaneously,
//! demonstrating `sync_sizes` + `on_resize` coordination.

use iced::{Color, Element, Length, Theme};
use iced::widget::{center, checkbox, column, container, row, text};

use iced_sash::{Id, SashH, SashV};

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
}

impl App {
    fn new() -> Self {
        Self { 
            sizes: vec![200.0, 300.0, 200.0],
            sync_sashes: false,
            is_checked: false,
         }
    }
}

#[derive(Debug, Clone)]
enum Message {
    OnChecked(bool),
    ResizedH(Id, usize, f32),
    ResizedV(Id, usize, f32),
    // Released(Id, usize),
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
            },
            Message::ResizedH(_id, index, size) => {
                if self.sync_sashes {
                    // Needed when the sync is used
                    iced_sash::resize(&mut self.sizes, index, size, 50.0);
                }
            }
            // Message::Released(_, _) => {}
            Message::ResizedV(id, index, value) => {
                println!("{:?}, {}, {}", id, index, value)
            },
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
                200.0,
                4.0,
            )
            .min_size(50.0)
            
            .on_resize(Message::ResizedH);
            
            let sh = if self.sync_sashes {
                sh.sync_sashes(self.sizes.clone())
            } else { sh };

            sh.into()
        };

        let sashv = SashV::new(
                vec![
                    panel("Top",   Color::from_rgb(0.25, 0.35, 0.55)),
                    panel("Center", Color::from_rgb(0.30, 0.50, 0.35)),
                    panel("Bottom",  Color::from_rgb(0.55, 0.35, 0.25)),
                ],
                self.sizes.clone(),
                200.0,
                4.0,
            )
            .min_size(50.0)
            .on_resize(Message::ResizedV)
            .style(move|theme, status| {
                iced_sash::primary(theme, status)
            })
            .into();

        let chk = 
            checkbox(self.is_checked)
            .label("Set Sashes to Sync")
            .on_toggle(Message::OnChecked)
            .into();

        let col = 
            column(vec![make_sash(), make_sash(), chk]).spacing(20.0).into();

        row(vec![col, sashv]).spacing(20.0).into()
    
    }
}

