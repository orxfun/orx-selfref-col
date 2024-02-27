use super::node_data::NodeData;

/// Node data storage with eager closure.
/// In other words, the value is stored as it is and closing of a node means removing it from the vector.
#[derive(Clone)]
pub struct NodeDataEagerClose<T>(T);

impl<T> NodeData<T> for NodeDataEagerClose<T> {
    #[inline(always)]
    fn active(value: T) -> Self {
        Self(value)
    }

    #[inline(always)]
    fn get(&self) -> Option<&T> {
        Some(&self.0)
    }

    #[inline(always)]
    fn get_mut(&mut self) -> Option<&mut T> {
        Some(&mut self.0)
    }

    #[inline(always)]
    fn swap_data(&mut self, new_value: T) -> T {
        let mut storage = new_value;
        std::mem::swap(&mut self.0, &mut storage);
        storage
    }
}

impl<T> From<T> for NodeDataEagerClose<T> {
    #[inline(always)]
    fn from(value: T) -> Self {
        Self(value)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn active() {
        let data = NodeDataEagerClose::active(42);
        assert_eq!(42, data.0);
    }

    #[test]
    fn from() {
        let data: NodeDataEagerClose<_> = 'x'.into();
        assert_eq!('x', data.0);
    }

    #[test]
    fn get() {
        let data = NodeDataEagerClose::active(42);
        assert_eq!(Some(&42), data.get());
    }

    #[test]
    fn get_mut() {
        let mut data = NodeDataEagerClose::active(42);
        *data.get_mut().unwrap() = 7;
        assert_eq!(Some(&7), data.get());
    }

    #[test]
    fn swap_data() {
        let mut data = NodeDataEagerClose::active(42);
        let old = data.swap_data(7);
        assert_eq!(Some(&7), data.get());
        assert_eq!(42, old);
    }
}
