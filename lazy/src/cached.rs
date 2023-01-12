use iced_native::event;
use iced_native::layout::{self, Layout};
use iced_native::mouse;
use iced_native::overlay;
use iced_native::renderer;
use iced_native::widget::tree::{self, Tree};
use iced_native::widget::{self, Widget};
use iced_native::Element;
use iced_native::{Clipboard, Hasher, Length, Point, Rectangle, Shell, Size};

use ouroboros::self_referencing;
use std::cell::RefCell;
use std::hash::{Hash, Hasher as H};
use std::rc::Rc;

#[allow(missing_debug_implementations)]
pub struct Cached<'a, Message, Renderer, Dependency, Output, View> {
    dependency: Dependency,
    computation: Box<dyn Fn(&Dependency, Option<Output>) -> Output + 'a>,
    output: RefCell<Option<Rc<RefCell<Option<Output>>>>>,
    view: Box<dyn Fn(&Output) -> View + 'a>,
    element: Rc<RefCell<Option<Element<'a, Message, Renderer>>>>,
}

impl<'a, Message, Renderer, Dependency, Output, View>
    Cached<'a, Message, Renderer, Dependency, Output, View>
where
    Dependency: Hash + 'a,
    Output: 'static,
    View: Into<Element<'a, Message, Renderer>> + 'a,
    Message: 'a,
    Renderer: 'a,
{
    pub fn new(
        dependency: Dependency,
        computation: impl Fn(&Dependency, Option<Output>) -> Output + 'a,
        view: impl Fn(&Output) -> View + 'a,
    ) -> Self {
        Self {
            dependency,
            computation: Box::new(computation),
            output: RefCell::new(None),
            view: Box::new(view),
            element: Rc::new(RefCell::new(None)),
        }
    }

    fn with_element<T>(
        &self,
        f: impl FnOnce(&Element<Message, Renderer>) -> T,
    ) -> T {
        f(self.element.borrow().as_ref().unwrap())
    }

    fn with_element_mut<T>(
        &self,
        f: impl FnOnce(&mut Element<Message, Renderer>) -> T,
    ) -> T {
        f(self.element.borrow_mut().as_mut().unwrap())
    }
}

struct Internal<Output> {
    output: Rc<RefCell<Option<Output>>>,
    hash: u64,
}

impl<'a, Message, Renderer, Dependency, Output, View> Widget<Message, Renderer>
    for Cached<'a, Message, Renderer, Dependency, Output, View>
where
    Output: 'static,
    View: Into<Element<'a, Message, Renderer>> + 'a,
    Dependency: Hash + 'a,
    Message: 'a,
    Renderer: iced_native::Renderer + 'a,
{
    fn tag(&self) -> tree::Tag {
        struct Tag<T>(T);
        tree::Tag::of::<Tag<Output>>()
    }

    fn state(&self) -> tree::State {
        let mut hasher = Hasher::default();
        self.dependency.hash(&mut hasher);
        let hash = hasher.finish();

        let output = (self.computation)(&self.dependency, None);
        let element = (self.view)(&output);
        (*self.output.borrow_mut()) = Some(Rc::new(RefCell::new(Some(output))));
        (*self.element.borrow_mut()) = Some(element.into());

        tree::State::new(Internal {
            output: self.output.borrow().as_ref().unwrap().clone(),
            hash,
        })
    }

    fn children(&self) -> Vec<Tree> {
        self.with_element(|element| vec![Tree::new(element.as_widget())])
    }

    fn diff(&self, tree: &mut Tree) {
        let current = tree.state.downcast_mut::<Internal<Output>>();

        let mut hasher = Hasher::default();
        self.dependency.hash(&mut hasher);
        let new_hash = hasher.finish();

        if current.hash != new_hash {
            current.hash = new_hash;

            let current_output = current.output.borrow_mut().take().unwrap();
            let output =
                (self.computation)(&self.dependency, Some(current_output));
            let element = (self.view)(&output);
            current.output = Rc::new(RefCell::new(Some(output)));
            (*self.output.borrow_mut()) = Some(current.output.clone());
            (*self.element.borrow_mut()) = Some(element.into());

            self.with_element(|element| {
                tree.diff_children(std::slice::from_ref(&element.as_widget()))
            });
        } else {
            (*self.output.borrow_mut()) = Some(current.output.clone());
            (*self.element.borrow_mut()) = Some(
                (self.view)(current.output.borrow().as_ref().unwrap()).into(),
            );

            self.with_element(|element| {
                tree.diff_children(std::slice::from_ref(&element.as_widget()))
            });
        }
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

    fn operate(
        &self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn widget::Operation<Message>,
    ) {
        self.with_element(|element| {
            element.as_widget().operate(
                &mut tree.children[0],
                layout,
                renderer,
                operation,
            );
        });
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
        self.with_element_mut(|element| {
            element.as_widget_mut().on_event(
                &mut tree.children[0],
                event,
                layout,
                cursor_position,
                renderer,
                clipboard,
                shell,
            )
        })
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

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Renderer::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
    ) {
        self.with_element(|element| {
            element.as_widget().draw(
                &tree.children[0],
                renderer,
                theme,
                style,
                layout,
                cursor_position,
                viewport,
            )
        })
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
    ) -> Option<overlay::Element<'_, Message, Renderer>> {
        let overlay = Overlay(Some(
            InnerBuilder {
                cell: self.element.clone(),
                element: self.element.borrow_mut().take().unwrap(),
                tree: &mut tree.children[0],
                overlay_builder: |element, tree| {
                    element.as_widget_mut().overlay(tree, layout, renderer)
                },
            }
            .build(),
        ));

        let has_overlay = overlay
            .with_overlay_maybe(|overlay| overlay::Element::position(overlay));

        has_overlay
            .map(|position| overlay::Element::new(position, Box::new(overlay)))
    }
}

#[self_referencing]
struct Inner<'a, 'b: 'a, Message, Renderer>
where
    Message: 'a,
    Renderer: 'a,
{
    cell: Rc<RefCell<Option<Element<'b, Message, Renderer>>>>,
    element: Element<'b, Message, Renderer>,
    tree: &'a mut Tree,

    #[borrows(mut element, mut tree)]
    #[covariant]
    overlay: Option<overlay::Element<'this, Message, Renderer>>,
}

struct Overlay<'a, 'b: 'a, Message, Renderer>(
    Option<Inner<'a, 'b, Message, Renderer>>,
);

impl<'a, 'b, Message, Renderer> Drop for Overlay<'a, 'b, Message, Renderer> {
    fn drop(&mut self) {
        let heads = self.0.take().unwrap().into_heads();
        (*heads.cell.borrow_mut()) = Some(heads.element);
    }
}

impl<'a, 'b, Message, Renderer> Overlay<'a, 'b, Message, Renderer> {
    fn with_overlay_maybe<T>(
        &self,
        f: impl FnOnce(&overlay::Element<'_, Message, Renderer>) -> T,
    ) -> Option<T> {
        self.0.as_ref().unwrap().borrow_overlay().as_ref().map(f)
    }

    fn with_overlay_mut_maybe<T>(
        &mut self,
        f: impl FnOnce(&mut overlay::Element<'_, Message, Renderer>) -> T,
    ) -> Option<T> {
        self.0
            .as_mut()
            .unwrap()
            .with_overlay_mut(|overlay| overlay.as_mut().map(f))
    }
}

impl<'a, 'b, Message, Renderer> overlay::Overlay<Message, Renderer>
    for Overlay<'a, 'b, Message, Renderer>
where
    Renderer: iced_native::Renderer,
{
    fn layout(
        &self,
        renderer: &Renderer,
        bounds: Size,
        position: Point,
    ) -> layout::Node {
        self.with_overlay_maybe(|overlay| {
            let vector = position - overlay.position();

            overlay.layout(renderer, bounds).translate(vector)
        })
        .unwrap_or_default()
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Renderer::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor_position: Point,
    ) {
        let _ = self.with_overlay_maybe(|overlay| {
            overlay.draw(renderer, theme, style, layout, cursor_position);
        });
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.with_overlay_maybe(|overlay| {
            overlay.mouse_interaction(
                layout,
                cursor_position,
                viewport,
                renderer,
            )
        })
        .unwrap_or_default()
    }

    fn on_event(
        &mut self,
        event: iced_native::Event,
        layout: Layout<'_>,
        cursor_position: Point,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) -> event::Status {
        self.with_overlay_mut_maybe(|overlay| {
            overlay.on_event(
                event,
                layout,
                cursor_position,
                renderer,
                clipboard,
                shell,
            )
        })
        .unwrap_or(iced_native::event::Status::Ignored)
    }
}

impl<'a, Message, Renderer, Dependency, Output, View>
    From<Cached<'a, Message, Renderer, Dependency, Output, View>>
    for Element<'a, Message, Renderer>
where
    Output: 'static,
    View: Into<Element<'a, Message, Renderer>> + 'a,
    Renderer: iced_native::Renderer + 'a,
    Message: 'a,
    Dependency: Hash + 'a,
{
    fn from(
        cached: Cached<'a, Message, Renderer, Dependency, Output, View>,
    ) -> Self {
        Self::new(cached)
    }
}
