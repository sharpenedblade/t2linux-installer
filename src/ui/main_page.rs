use crate::ui::{
    app::{AppMessage, Page},
    download_page,
};
use crate::{distro::Distro, install::InstallSettings};
use iced::{
    widget::{button, column, container, radio, scrollable, text},
    Length,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MainPageMessage {
    PickDistro(usize),
    StartInstall,
}

pub struct MainPage {
    distro_index: Option<usize>,
}

impl MainPage {
    pub fn new() -> Self {
        Self { distro_index: None }
    }
}

impl Page for MainPage {
    fn update(&mut self, message: AppMessage) -> (Option<Box<dyn Page>>, iced::Task<AppMessage>) {
        let mut page: Option<Box<dyn Page>> = None;
        if let AppMessage::Main(msg) = message {
            match msg {
                MainPageMessage::PickDistro(distro_index) => self.distro_index = Some(distro_index),
                MainPageMessage::StartInstall => {
                    let install_settings = InstallSettings::new(
                        Distro::get_all()
                            .unwrap()
                            .get(self.distro_index.unwrap())
                            .unwrap()
                            .clone(),
                    );
                    page = Some(Box::new(download_page::DownloadPage::new(install_settings)))
                }
            }
        }
        (page, iced::Task::none())
    }
    fn view(&self) -> iced::Element<AppMessage> {
        let mut distro_list = column![].spacing(16);
        let distros = Distro::get_all().unwrap();
        for (i, distro) in distros.iter().enumerate() {
            distro_list =
                distro_list.push(radio(distro.name.clone(), i, self.distro_index, |_| {
                    AppMessage::Main(MainPageMessage::PickDistro(i))
                }));
        }

        container(
            column![
                text("Choose a distro").size(24),
                scrollable(distro_list).height(400).width(400),
                button("Next").on_press(AppMessage::Main(MainPageMessage::StartInstall))
            ]
            .spacing(16)
            .padding(8),
        )
        .align_x(iced::alignment::Horizontal::Center)
        .align_y(iced::alignment::Vertical::Center)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    fn subscription(&self) -> iced::Subscription<AppMessage> {
        iced::Subscription::none()
    }
}
