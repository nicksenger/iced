use iced::widget::button;
use iced::{Element, Sandbox, Settings};

use counter::ViewCounter;

pub fn main() -> iced::Result {
    Component::run(Settings::default())
}

#[derive(Default)]
struct Component;

#[derive(Debug, Clone, Copy)]
enum Message {
    ReView,
}

impl Sandbox for Component {
    type Message = Message;

    fn new() -> Self {
        Self::default()
    }

    fn title(&self) -> String {
        String::from("Nested Component - Iced")
    }

    fn update(&mut self, _message: Message) {}

    fn view(&self) -> Element<Message> {
        ViewCounter::new(|| {
            ViewCounter::new(|| {
                ViewCounter::new(|| {
                    ViewCounter::new(|| {
                        ViewCounter::new(|| {
                            ViewCounter::new(|| {
                                ViewCounter::new(|| {
                                    ViewCounter::new(|| {
                                        ViewCounter::new(|| {
                                            ViewCounter::new(|| {
                                                ViewCounter::new(|| {
                                                    ViewCounter::new(|| {
                                                        button("Re-View")
                                                            .on_press(
                                                                Message::ReView,
                                                            )
                                                            .into()
                                                    })
                                                    .into()
                                                })
                                                .into()
                                            })
                                            .into()
                                        })
                                        .into()
                                    })
                                    .into()
                                })
                                .into()
                            })
                            .into()
                        })
                        .into()
                    })
                    .into()
                })
                .into()
            })
            .into()
        })
        .into()
    }
}

mod counter {
    use std::cell::RefCell;

    use iced::Element;
    use iced_lazy::{self, Component};
    use iced_native::widget::{column, text};

    pub struct ViewCounter<'a, Message, Renderer> {
        content: fn() -> Element<'a, Message, Renderer>,
    }

    pub struct State(RefCell<usize>);

    impl State {
        fn count(&self) -> usize {
            *self.0.borrow()
        }

        fn increment(&self) {
            *self.0.borrow_mut() += 1;
        }
    }

    impl Default for State {
        fn default() -> Self {
            Self(RefCell::new(0))
        }
    }

    impl<'a, Message, Renderer> ViewCounter<'a, Message, Renderer> {
        pub fn new(content: fn() -> Element<'a, Message, Renderer>) -> Self {
            Self { content }
        }
    }

    impl<'a, Message, Renderer> Component<Message, Renderer>
        for ViewCounter<'a, Message, Renderer>
    where
        Renderer: iced_native::text::Renderer + 'static,
        Renderer::Theme: iced_native::widget::text::StyleSheet,
    {
        type State = State;
        type Event = Message;

        fn update(
            &mut self,
            _state: &mut Self::State,
            message: Message,
        ) -> Option<Message> {
            Some(message)
        }

        fn view(&self, state: &Self::State) -> Element<Message, Renderer> {
            state.increment();

            column(vec![
                text(format!("viewed {} times", state.count())).into(),
                (self.content)(),
            ])
            .into()
        }
    }

    impl<'a, Message, Renderer> From<ViewCounter<'a, Message, Renderer>>
        for Element<'a, Message, Renderer>
    where
        Message: 'a,
        Renderer: iced_native::text::Renderer + 'static,
        Renderer::Theme: iced_native::widget::text::StyleSheet,
    {
        fn from(view_counter: ViewCounter<'a, Message, Renderer>) -> Self {
            iced_lazy::component(view_counter)
        }
    }
}
