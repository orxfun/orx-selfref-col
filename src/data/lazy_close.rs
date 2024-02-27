use super::node_data::NodeData;

/// Node data storage with lazy closure.
/// In other words, the value is stored as it is and closing of a node means setting it to None.
#[derive(Clone)]
pub struct NodeDataLazyClose<T>(Option<T>);

impl<T> NodeData<T> for NodeDataLazyClose<T> {
    #[inline(always)]
    fn active(value: T) -> Self {
        Self(Some(value))
    }

    #[inline(always)]
    fn get(&self) -> Option<&T> {
        self.0.as_ref()
    }

    #[inline(always)]
    fn get_mut(&mut self) -> Option<&mut T> {
        self.0.as_mut()
    }

    #[inline(always)]
    fn swap_data(&mut self, new_value: T) -> T {
        let output = self.0.take();
        self.0 = Some(new_value);
        output.expect("NodeDataLazyClose must be `is_active` to be able to `swap_data`")
    }
}

impl<T> NodeDataLazyClose<T> {
    /// Creates a new closed node data.
    #[inline(always)]
    pub fn closed() -> Self {
        Self(None)
    }

    /// Returns whether or not the node data is active.
    #[inline(always)]
    pub fn is_active(&self) -> bool {
        self.0.is_some()
    }

    /// Returns whether or not the node data is closed.
    #[inline(always)]
    pub fn is_closed(&self) -> bool {
        self.0.is_none()
    }

    /// Closes the node storage and returns the internally stored value.
    #[inline(always)]
    pub fn close(&mut self) -> Option<T> {
        self.0.take()
    }
}

impl<T> Default for NodeDataLazyClose<T> {
    #[inline(always)]
    fn default() -> Self {
        Self::closed()
    }
}

impl<T> From<T> for NodeDataLazyClose<T> {
    #[inline(always)]
    fn from(value: T) -> Self {
        Self(Some(value))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn active() {
        let data = NodeDataLazyClose::active(42);
        assert_eq!(Some(42), data.0);
    }

    #[test]
    fn from() {
        let data: NodeDataLazyClose<_> = 'x'.into();
        assert_eq!(Some('x'), data.0);
    }

    #[test]
    fn get() {
        let data = NodeDataLazyClose::active(42);
        assert_eq!(Some(&42), data.get());
    }

    #[test]
    fn get_mut() {
        let mut data = NodeDataLazyClose::active(42);
        *data.get_mut().unwrap() = 7;
        assert_eq!(Some(&7), data.get());
    }

    #[test]
    fn swap_data() {
        let mut data = NodeDataLazyClose::active(42);
        let old = data.swap_data(7);
        assert_eq!(Some(&7), data.get());
        assert_eq!(42, old);
    }

    #[test]
    #[should_panic]
    fn swap_data_of_closed() {
        let mut data = NodeDataLazyClose::closed();
        let _old = data.swap_data(7);
    }

    #[test]
    fn closed() {
        let data = NodeDataLazyClose::<char>::closed();
        assert_eq!(None, data.0);

        let data = NodeDataLazyClose::<char>::default();
        assert_eq!(None, data.0);
    }

    #[test]
    fn is_active() {
        let data = NodeDataLazyClose::active(42);
        assert!(data.is_active());

        let data = NodeDataLazyClose::<char>::closed();
        assert!(!data.is_active());
    }

    #[test]
    fn is_closed() {
        let data = NodeDataLazyClose::active(42);
        assert!(!data.is_closed());

        let data = NodeDataLazyClose::<char>::closed();
        assert!(data.is_closed());
    }

    #[test]
    fn close() {
        let mut data = NodeDataLazyClose::active(42);
        assert_eq!(Some(42), data.close());
        assert!(data.is_closed());

        let mut data = NodeDataLazyClose::<char>::closed();
        assert_eq!(None, data.close());
    }
}
