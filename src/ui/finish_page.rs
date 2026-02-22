use crate::ui::app::{AppMessage, Page};
use iced::widget::{button, column, container, text};
use iced::Length;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct FinishPage {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FinishPageMessage {
    Exit,
}

impl FinishPage {
    pub fn new() -> Self {
        Self {}
    }
}

impl Page for FinishPage {
    fn update(
        &mut self,
        message: AppMessage,
    ) -> (Option<Box<(dyn Page)>>, iced::Command<AppMessage>) {
        let command: iced::Command<AppMessage> = iced::Command::none();
        let page: Option<Box<dyn Page>> = None;
        if let AppMessage::Finish(msg) = message {
            match msg {
                FinishPageMessage::Exit => {
                    todo!("Implement app shutdown");
                }
            }
        }
        (page, command)
    }
    fn view(&self) -> iced::Element<AppMessage> {
        container(
            column![
                text("Finished download. You can flash the ISO now."),
                button("Exit").on_press(AppMessage::Finish(FinishPageMessage::Exit))
            ]
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
