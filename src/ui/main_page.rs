use crate::ui::{
    app::{AppMessage, Page},
    download_page,
};
use crate::{distro::Distro, install::DownloadSettings};
use iced::widget::{button, column, radio, text};

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
    fn update(
        &mut self,
        message: AppMessage,
    ) -> (Option<Box<dyn Page>>, iced::Command<AppMessage>) {
        let mut page: Option<Box<dyn Page>> = None;
        if let AppMessage::MainPage(msg) = message {
            match msg {
                MainPageMessage::PickDistro(distro_index) => self.distro_index = Some(distro_index),
                MainPageMessage::StartInstall => {
                    let install_settings = DownloadSettings::new(
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
        (page, iced::Command::none())
    }
    fn view(&self) -> iced::Element<AppMessage> {
        let mut distro_list = column![text("Choose a distro").size(24),].spacing(8);
        let distros = Distro::get_all().unwrap();
        for (i, distro) in distros.iter().enumerate() {
            distro_list =
                distro_list.push(radio(distro.name.clone(), i, self.distro_index, |_| {
                    AppMessage::MainPage(MainPageMessage::PickDistro(i))
                }));
        }

        column![
            distro_list,
            button("Begin installation")
                .on_press(AppMessage::MainPage(MainPageMessage::StartInstall))
        ]
        .spacing(16)
        .padding(8)
        .into()
    }

    fn subscription(&self) -> iced::Subscription<AppMessage> {
        iced::Subscription::none()
    }
}
