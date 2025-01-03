use super::refs::Refs;

/// Zero-sized no-reference.0
#[derive(Clone, Debug)]
pub struct RefsNone;

impl Refs for RefsNone {
    #[inline(always)]
    fn empty() -> Self {
        Self
    }

    #[inline(always)]
    fn is_empty(&self) -> bool {
        true
    }

    #[inline(always)]
    fn clear(&mut self) {}

    #[inline(always)]
    fn remove_at(&mut self, _: usize) {}

    #[inline(always)]
    fn remove(&mut self, _: usize) -> Option<usize> {
        None
    }
}
