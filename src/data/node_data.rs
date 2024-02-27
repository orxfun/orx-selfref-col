/// Trait representing how the node data will be stored in a self referential collection.
pub trait NodeData<T> {
    /// Creates a new active node data with the given `value`.
    fn active(value: T) -> Self;

    /// Returns a reference to the stored value; returns None if the node is not active.
    fn get(&self) -> Option<&T>;

    /// Returns a mutable reference to the stored value; returns None if the node is not active.
    fn get_mut(&mut self) -> Option<&mut T>;

    /// Updates the node data with the `new_value` and returns back the old value.
    fn swap_data(&mut self, new_value: T) -> T;
}
