//! Example — sash widget feature demo.
//!
//! Demonstrates panel sync, outer resizing with mode selection,
//! and cross-size (height) sync across linked SashH widgets.

use iced::Length::{self, Fill};
use iced::{Color, Element, Theme};
use iced::widget::{center, column, container, rule, scrollable, text};

use iced_sash::SashH;



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
    height: f32,
}

impl App {
    fn new() -> Self {
        Self {
            sizes: vec![100.0; 7],
            height: 25.0,
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    Resize(usize, f32),
}

impl App {
    fn title(&self) -> String {
        String::from("Sash Example")
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Resize(idx, val) => {
                iced_sash::resize(&mut self.sizes, idx, val, 10.0);
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let (header_data, body_data) =
            load_csv("assets/google.csv").unwrap_or_default();

        let header = table_header(&header_data, self.sizes.clone(), self.height);
        let body = table_body(&body_data, self.sizes.clone(), self.height);

        let table = column![header, body];

        center(container(table)
            .style(move|theme| {
                container::bordered_box(theme)
            })).into()
    }
}

fn table_header<'a>(header: &[String], sizes: Vec<f32>, height: f32) -> Element<'a, Message> {
    let sash: Element<'a, Message> = SashH::new(
        header.iter().map(|col| center(text(col.clone())).center(Fill).into()).collect(),
        sizes.clone(),
        height,
        6.0,
    )
    .min_size(10.0)
    .on_resize(|_id, idx, val| Message::Resize(idx, val))
    .sync_sashes(sizes.clone())
    .style(|theme, status| iced_sash::subtle(theme, status))
    .into();

    container(column![sash, container(rule::horizontal(6)).width(Length::Fixed(sizes.iter().sum()))])
        .style(|theme| container::rounded_box(theme))
        .into()
}

fn table_body<'a>(body: &[Vec<String>], sizes: Vec<f32>, height: f32) -> Element<'a, Message> {
    let rows: Vec<Element<'a, Message>> = body
        .iter()
        .enumerate()
        .map(|(i, row)| {
            let sash: Element<'a, Message> = SashH::new(
                row.iter().map(|cell| center(text(cell.clone()).size(14.0)).center(Fill).into()).collect(),
                sizes.clone(),
                height,
                6.0,
            )
            .min_size(10.0)
            .on_resize(|_id, idx, val| Message::Resize(idx, val))
            .sync_sashes(sizes.clone())
            .style(|theme, status| iced_sash::subtle(theme, status))
            .into();

            if i % 2 == 1 {
                container(sash)
                    .style(|theme: &Theme| {
                        let base = theme.palette().background.base.color;
                        let weak = theme.palette().background.weak.color;
                        let mid = Color {
                            r: (base.r + weak.r) / 2.0,
                            g: (base.g + weak.g) / 2.0,
                            b: (base.b + weak.b) / 2.0,
                            a: 1.0,
                        };
                        container::Style {
                            background: Some(mid.into()),
                            ..Default::default()
                        }
                    })
                    .into()
            } else {
                sash
            }
        })
        .collect();

    scrollable(column(rows)).height(600.0).into()
}

fn load_csv(path: &str) -> Result<(Vec<String>, Vec<Vec<String>>), csv::Error> {
    let mut reader = csv::Reader::from_path(path)?;

    let header: Vec<String> = reader
        .headers()?
        .iter()
        .map(|s| s.to_string())
        .collect();

    let body: Vec<Vec<String>> = reader
        .records()
        .map(|r| r.map(|rec| rec.iter().map(|s| s.to_string()).collect()))
        .collect::<Result<_, _>>()?;

    Ok((header, body))
}
