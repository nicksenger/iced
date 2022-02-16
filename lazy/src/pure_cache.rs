use iced_pure::overlay;
use iced_pure::widget::Element;

use ouroboros::self_referencing;

#[self_referencing(pub_extras)]
pub struct PureCache<'a, Message: 'a, Renderer: 'a> {
    pub element: Option<Element<'a, Message, Renderer>>,

    #[borrows(mut element)]
    #[covariant]
    pub overlay: Option<overlay::Element<'this, Message, Renderer>>,
}
