use std::sync::Arc;

use crate::ui::app::{AppMessage, Page};
use iced::Length;
use iced::widget::{button, column, container, row, text};
use iced::window::{self};

use super::main_page::MainPage;

#[derive(Debug)]
pub enum FinishState {
    Clean,
    Error(Arc<anyhow::Error>),
    Cancelled,
}

#[derive(Debug)]
pub struct FinishPage {
    state: FinishState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FinishPageMessage {
    Exit,
    Retry,
}

impl FinishPage {
    pub fn new(state: FinishState) -> Self {
        Self { state }
    }
}

impl Page for FinishPage {
    fn update(&mut self, message: AppMessage) -> (Option<Box<dyn Page>>, iced::Task<AppMessage>) {
        let mut command: iced::Task<AppMessage> = iced::Task::none();
        let mut page: Option<Box<dyn Page>> = None;
        if let AppMessage::Finish(msg) = message {
            match msg {
                FinishPageMessage::Exit => {
                    command = window::oldest().then(|id| window::close(id.unwrap()));
                }
                FinishPageMessage::Retry => {
                    page = Some(Box::new(MainPage::new()));
                    command = MainPage::init_tasks();
                }
            }
        }
        (page, command)
    }
    fn view(&self) -> iced::Element<'_, AppMessage> {
        let (glyph, title, subtitle) = match self.state {
            FinishState::Clean => (
                "✓",
                "Download Complete",
                "The Linux image is ready for the next step.",
            ),
            FinishState::Error(_) => (
                "!",
                "Download Failed",
                "The image could not be downloaded. Check details below and try again.",
            ),
            FinishState::Cancelled => (
                "×",
                "Download Cancelled",
                "No changes were made after cancellation.",
            ),
        };

        let mut col = column![
            text(glyph).size(56),
            text(title).size(34),
            text(subtitle).size(18),
        ]
        .spacing(14);
        if let FinishState::Error(e) = &self.state {
            println!("{e:#}");
            col = col.push(text(e.to_string()).size(14));
        };
        let mut row1 = row![].spacing(14);
        match self.state {
            FinishState::Clean => {}
            FinishState::Error(_) | FinishState::Cancelled => {
                row1 = row1.push(button("Try Again").on_press(AppMessage::Finish(
                    FinishPageMessage::Retry,
                )))
            }
        }
        row1 = row1.push(button("Done").on_press(AppMessage::Finish(FinishPageMessage::Exit)));
        col = col.push(row1);
        container(col.spacing(20).padding(30).max_width(620))
            .align_x(iced::alignment::Horizontal::Center)
            .align_y(iced::alignment::Vertical::Center)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn subscription(&self) -> iced::Subscription<AppMessage> {
        iced::Subscription::batch(vec![])
    }
}
