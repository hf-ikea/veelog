use iced::widget::text;

#[derive(Debug, Clone, Copy)]
pub enum Message {}

pub struct Logger;

impl Default for Logger {
    fn default() -> Self {
        Self {}
    }
}

impl Logger {
    pub fn title(&self) -> String {
        format!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
    }

    pub fn update(&mut self, message: Message) {
        match message {}
    }

    pub fn view(&self) -> iced::Element<'_, Message> {
        text("hello iced!").into()
    }
}

fn main() -> iced::Result {
    iced::application(Logger::title, Logger::update, Logger::view)
        .centered()
        .run()
}

#[cfg(test)]
mod tests {
    #[test]
    pub fn test() {}
}
