use crate::ui::app::{AppMessage, Page};
use iced::Length;
use iced::widget::{button, column, container, text};
use iced::window::{self};

#[derive(Debug)]
pub enum FinishState {
    Clean,
    Error(anyhow::Error),
    Cancelled,
}

#[derive(Debug)]
pub struct FinishPage {
    state: FinishState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FinishPageMessage {
    Exit,
}

impl FinishPage {
    pub fn new(state: FinishState) -> Self {
        Self { state }
    }
}

impl Page for FinishPage {
    fn update(&mut self, message: AppMessage) -> (Option<Box<(dyn Page)>>, iced::Task<AppMessage>) {
        let mut command: iced::Task<AppMessage> = iced::Task::none();
        let page: Option<Box<dyn Page>> = None;
        if let AppMessage::Finish(msg) = message {
            match msg {
                FinishPageMessage::Exit => {
                    command = window::oldest().then(|id| window::close(id.unwrap()));
                }
            }
        }
        (page, command)
    }
    fn view(&self) -> iced::Element<AppMessage> {
        let mut col = column![
            text(match self.state {
                FinishState::Clean => "Finished Download",
                FinishState::Error(_) => "Download failed",
                FinishState::Cancelled => "Cancelled Download",
            })
            .size(24),
        ]
        .spacing(16);
        if let FinishState::Error(e) = &self.state {
            println!("{e:#}");
            col = col.push(text(e.to_string()));
        };
        container(
            col.push(button("Exit").on_press(AppMessage::Finish(FinishPageMessage::Exit)))
                .spacing(16),
        )
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
