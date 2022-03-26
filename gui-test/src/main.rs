use iced::{button, Align, Button, Column, Element, Sandbox, Settings, Text,Length};

pub fn main() -> iced::Result {
    let mut settings = Settings::default();
    settings.window.size = (512,512);
    settings.window.transparent = true;
    settings.window.always_on_top = true;
    Counter::run(settings)
}

#[derive(Default)]
struct Counter {
    value: i32,
    increment_button: button::State,
    decrement_button: button::State,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    IncrementPressed,
    DecrementPressed,
}

impl Sandbox for Counter {
    type Message = Message;

    fn new() -> Self {
        Self::default()
    }

    fn title(&self) -> String {
        String::from("Counter - Iced")
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::IncrementPressed => {
                self.value += 1;
            }
            Message::DecrementPressed => {
                self.value -= 1;
            }
        }
    }

    fn view(&mut self) -> Element<Message> {
        Column::new()
            .padding(20)
            .align_items(Align::Center)
            .push(
                Button::new(&mut self.increment_button, Text::new("Increment"))
                    .on_press(Message::IncrementPressed).width(Length::Fill),
            )
            .push(Text::new(self.value.to_string()).size(50).width(Length::Fill).horizontal_alignment(iced::HorizontalAlignment::Center))
            .push(
                Button::new(&mut self.decrement_button, Text::new("Decrement"))
                    .on_press(Message::DecrementPressed).width(Length::Fill),
            )
            .height(Length::Fill)
            .width(Length::Fill)
            .into()
    }
}
