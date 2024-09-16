use super::refs::Refs;

/// Zero-sized no-reference.0
#[derive(Clone, Debug)]
pub struct RefsNone;

impl Refs for RefsNone {
    fn empty() -> Self {
        Self
    }

    fn is_empty(&self) -> bool {
        true
    }

    fn clear(&mut self) {}
}
