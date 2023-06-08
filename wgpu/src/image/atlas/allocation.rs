use crate::core::Size;
use crate::image::atlas::{self, allocator};

#[derive(Debug)]
pub enum Allocation {
    Partial {
        layer: usize,
        region: allocator::Region,
    },
    Full {
        layer: usize,
    },
}

impl Allocation {
    pub fn position(&self) -> (u32, u32) {
        match self {
            Allocation::Partial { region, .. } => region.position(),
            Allocation::Full { .. } => (0, 0),
        }
    }

    pub fn size(&self) -> Size<u32> {
        match self {
            Allocation::Partial { region, .. } => region.size(),
            Allocation::Full { .. } => Size::new(atlas::SIZE, atlas::SIZE),
        }
    }

    pub fn z_position(&self) -> usize {
        match self {
            Allocation::Partial { layer, .. } => *layer + 1,
            Allocation::Full { layer } => *layer + 1,
        }
    }
}
