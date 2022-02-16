use iced::pure::container;
use iced::pure::{Element, Sandbox};
use iced::{Length, Settings};
use numeric_input::NumericInput;

pub fn main() -> iced::Result {
    Component::run(Settings::default())
}

#[derive(Default)]
struct Component;

#[derive(Debug, Clone, Copy)]
enum Message {}

impl Sandbox for Component {
    type Message = Message;

    fn new() -> Self {
        Self::default()
    }

    fn title(&self) -> String {
        String::from("Component - Iced")
    }

    fn update(&mut self, _: Message) {}

    fn view(&self) -> Element<Message> {
        container(NumericInput::new())
            .padding(20)
            .height(Length::Fill)
            .center_y()
            .into()
    }
}

mod numeric_input {
    use iced::pure::widget::{Element, Row};
    use iced::pure::{button, text_input};
    use iced_lazy::pure_component::{self, PureComponent};
    use iced_native::alignment::{self, Alignment};
    use iced_native::text;
    use iced_native::widget::Text;
    use iced_native::Length;

    pub struct NumericInput {}

    #[derive(Default)]
    pub struct State {
        value: Option<u32>,
    }

    #[derive(Debug, Clone)]
    pub enum Event {
        InputChanged(String),
        IncrementPressed,
        DecrementPressed,
    }

    impl<'a> NumericInput {
        pub fn new() -> Self {
            Self {}
        }
    }

    impl<'a, Message, Renderer> PureComponent<Message, Renderer> for NumericInput
    where
        Renderer: 'static + text::Renderer,
    {
        type Event = Event;
        type State = State;

        fn state(&self) -> State {
            State::default()
        }

        fn update(&self, state: &mut State, event: Event) -> Option<Message> {
            match event {
                Event::IncrementPressed => {
                    if let Some(value) = state.value.as_mut() {
                        *value = value.saturating_add(1);
                    }
                }
                Event::DecrementPressed => {
                    if let Some(value) = state.value.as_mut() {
                        *value = value.saturating_sub(1);
                    }
                }
                Event::InputChanged(value) => {
                    if value.is_empty() {
                        state.value = None;
                    } else if let Some(value) = value.parse().ok() {
                        state.value = Some(value);
                    }
                }
            }

            None
        }

        fn view(&self, state: &State) -> Element<Event, Renderer> {
            let button = |label, on_press| {
                button(
                    Text::new(label)
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .horizontal_alignment(alignment::Horizontal::Center)
                        .vertical_alignment(alignment::Vertical::Center),
                )
                .width(Length::Units(50))
                .on_press(on_press)
            };

            Row::with_children(vec![
                button("-", Event::DecrementPressed).into(),
                text_input(
                    "Type a number",
                    state
                        .value
                        .as_ref()
                        .map(u32::to_string)
                        .as_ref()
                        .map(String::as_str)
                        .unwrap_or(""),
                    Event::InputChanged,
                )
                .padding(10)
                .into(),
                button("+", Event::IncrementPressed).into(),
            ])
            .align_items(Alignment::Fill)
            .spacing(10)
            .into()
        }
    }

    impl<'a, Message, Renderer> From<NumericInput>
        for Element<'a, Message, Renderer>
    where
        Message: Clone + 'static,
        Renderer: text::Renderer + 'static,
    {
        fn from(numeric_input: NumericInput) -> Self {
            pure_component::view(numeric_input)
        }
    }
}
