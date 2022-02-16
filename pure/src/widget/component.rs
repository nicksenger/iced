use super::cache::{Cache, CacheBuilder};
use crate::widget::tree::{self, Tree};
use crate::widget::{Element, Widget};

use iced_native::event;
use iced_native::layout;
use iced_native::mouse;
use iced_native::renderer;
use iced_native::{Clipboard, Hasher, Layout, Length, Point, Rectangle, Shell};

use ouroboros::self_referencing;
use std::cell::RefCell;
use std::marker::PhantomData;

pub trait Component<Message, Renderer> {
    type Event;
    type T;

    fn state(&self) -> Self::T;

    fn update(
        &self,
        state: &mut Self::T,
        event: Self::Event,
    ) -> Option<Message>;

    fn view(&self, state: &Self::T) -> Element<Self::Event, Renderer>;
}

pub fn view<'a, C, Message, Renderer, T>(
    component: C,
) -> Element<'a, Message, Renderer>
where
    C: Component<Message, Renderer, T = T> + 'a,
    T: 'static,
    Message: Clone + 'static,
    Renderer: iced_native::Renderer + 'static,
{
    Element::new(Instance {
        state: RefCell::new(Some(
            StateBuilder {
                component: Box::new(component),
                message: PhantomData,
                cache_builder: |_| {
                    Some(
                        CacheBuilder {
                            element: None,
                            overlay_builder: |_| None,
                        }
                        .build(),
                    )
                },
            }
            .build(),
        )),
    })
}

struct Instance<'a, Message, Renderer, Event, T> {
    state: RefCell<Option<State<'a, Message, Renderer, Event, T>>>,
}

#[self_referencing]
struct State<'a, Message: 'a, Renderer: 'a, Event: 'a, T: 'a> {
    component: Box<dyn Component<Message, Renderer, Event = Event, T = T> + 'a>,
    message: PhantomData<Message>,

    #[borrows(component)]
    #[covariant]
    cache: Option<Cache<'this, Event, Renderer>>,
}

impl<'a, Message, Renderer, Event, T>
    Instance<'a, Message, Renderer, Event, T>
{
    fn with_element<U>(
        &self,
        f: impl FnOnce(&Element<'_, Event, Renderer>) -> U,
    ) -> U {
        self.with_element_mut(|element| f(element))
    }

    fn with_element_mut<U>(
        &self,
        f: impl FnOnce(&mut Element<'_, Event, Renderer>) -> U,
    ) -> U {
        self.state
            .borrow_mut()
            .as_mut()
            .unwrap()
            .with_cache_mut(|cache| {
                let mut element =
                    cache.take().unwrap().into_heads().element.unwrap();
                let result = f(&mut element);

                *cache = Some(
                    CacheBuilder {
                        element: Some(element),
                        overlay_builder: |_| None,
                    }
                    .build(),
                );

                result
            })
    }
}

impl<'a, Message, Renderer, Event, T: 'static> Widget<Message, Renderer>
    for Instance<'a, Message, Renderer, Event, T>
where
    Message: 'static + Clone,
    Renderer: 'static + iced_native::Renderer,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<T>()
    }

    fn state(&self) -> tree::State {
        let component = self
            .state
            .borrow_mut()
            .take()
            .unwrap()
            .into_heads()
            .component;

        let x = component.state();

        *self.state.borrow_mut() = Some(
            StateBuilder {
                component,
                message: PhantomData,
                cache_builder: |state| {
                    Some(
                        CacheBuilder {
                            element: Some(state.view(&x)),
                            overlay_builder: |_| None,
                        }
                        .build(),
                    )
                },
            }
            .build(),
        );

        tree::State::new(x)
    }

    fn children(&self) -> Vec<Tree> {
        self.with_element(|el| vec![Tree::new(el)])
    }

    fn diff(&self, tree: &mut Tree) {
        let component = self
            .state
            .borrow_mut()
            .take()
            .unwrap()
            .into_heads()
            .component;

        *self.state.borrow_mut() = Some(
            StateBuilder {
                component,
                message: PhantomData,
                cache_builder: |state| {
                    Some(
                        CacheBuilder {
                            element: Some(
                                state.view(tree.state.downcast_mut()),
                            ),
                            overlay_builder: |_| None,
                        }
                        .build(),
                    )
                },
            }
            .build(),
        );

        self.with_element(|el| tree.diff_children(std::slice::from_ref(el)))
    }

    fn width(&self) -> Length {
        self.with_element(|element| element.as_widget().width())
    }

    fn height(&self) -> Length {
        self.with_element(|element| element.as_widget().height())
    }

    fn layout(
        &self,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.with_element(|element| {
            element.as_widget().layout(renderer, limits)
        })
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: iced_native::Event,
        layout: Layout<'_>,
        cursor_position: Point,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) -> event::Status {
        let mut local_messages = Vec::new();
        let mut local_shell = Shell::new(&mut local_messages);

        let event_status = self.with_element_mut(|element| {
            element.as_widget_mut().on_event(
                &mut tree.children[0],
                event,
                layout,
                cursor_position,
                renderer,
                clipboard,
                &mut local_shell,
            )
        });

        local_shell.revalidate_layout(|| shell.invalidate_layout());

        if !local_messages.is_empty() {
            let component = self
                .state
                .borrow_mut()
                .take()
                .unwrap()
                .into_heads()
                .component;

            for message in local_messages.into_iter().filter_map(|message| {
                component.update(tree.state.downcast_mut(), message)
            }) {
                shell.publish(message);
            }

            *self.state.borrow_mut() = Some(
                StateBuilder {
                    component,
                    message: PhantomData,
                    cache_builder: |state| {
                        Some(
                            CacheBuilder {
                                element: Some(
                                    state.view(tree.state.downcast_ref()),
                                ),
                                overlay_builder: |_| None,
                            }
                            .build(),
                        )
                    },
                }
                .build(),
            );

            shell.invalidate_layout();
        }

        event_status
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
    ) {
        self.with_element(|element| {
            element.as_widget().draw(
                &tree.children[0],
                renderer,
                style,
                layout,
                cursor_position,
                viewport,
            );
        });
    }

    fn hash_layout(&self, state: &mut Hasher) {
        self.with_element(|element| {
            element.as_widget().hash_layout(state);
        });
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.with_element(|element| {
            element.as_widget().mouse_interaction(
                &tree.children[0],
                layout,
                cursor_position,
                viewport,
                renderer,
            )
        })
    }
}
