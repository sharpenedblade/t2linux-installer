use iced::{
    widget::{button, checkbox, column, radio, text},
    Sandbox, Settings,
};

struct App {
    distro: Option<Distro>,
    auto_partitioning: bool,
    target_disk: Option<String>,
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Copy)]
enum Distro {
    Arch,
    Fedora,
    Ubuntu,
}

#[derive(Debug, Clone, Eq, PartialEq)]
enum AppMessage {
    PickDistro(Distro),
    ToggleAutoPartitioning(bool),
    PickDisk(String),
}

impl Sandbox for App {
    type Message = AppMessage;

    fn new() -> Self {
        Self {
            distro: None,
            auto_partitioning: true,
            target_disk: None,
        }
    }

    fn title(&self) -> String {
        String::from("t2linux Installer")
    }

    fn update(&mut self, _message: Self::Message) {
        match _message {
            AppMessage::PickDistro(distro) => self.distro = Some(distro),
            AppMessage::ToggleAutoPartitioning(b) => self.auto_partitioning = b,
            AppMessage::PickDisk(s) => self.target_disk = Some(s),
        }
    }

    fn view(&self) -> iced::Element<Self::Message> {
        let mut disk_list = column![text("Choose instalation disk").size(24)].spacing(4);
        for disk in ["disk0", "disk2"] {
            disk_list = disk_list.push(radio(
                disk,
                &disk.to_string(),
                self.target_disk.as_ref(),
                |s| AppMessage::PickDisk(s.into()),
            ));
        }
        column![
            column![
                text("Choose a distro").size(24),
                radio("Arch", Distro::Arch, self.distro, AppMessage::PickDistro),
                radio(
                    "Fedora",
                    Distro::Fedora,
                    self.distro,
                    AppMessage::PickDistro
                ),
                radio(
                    "Ubuntu",
                    Distro::Ubuntu,
                    self.distro,
                    AppMessage::PickDistro
                )
            ]
            .spacing(4),
            disk_list,
            checkbox("Automatic partitioning", self.auto_partitioning)
                .on_toggle(AppMessage::ToggleAutoPartitioning),
            button("Begin installation")
        ]
        .spacing(16)
        .padding(8)
        .into()
    }
}

fn main() -> iced::Result {
    App::run(Settings::default())
}
