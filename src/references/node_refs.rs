use crate::{variants::variant::Variant, Node};

/// Trait defining how the references of a self referential collection node will be stored.
pub trait NodeRefs<'a, V, T>: Default + Clone
where
    V: Variant<'a, T>,
{
    /// Type of the underlying references.
    type References;

    /// Creates a new node references for the given `references`.
    fn new(references: Self::References) -> Self;

    /// Creates empty references.
    fn empty() -> Self {
        Self::default()
    }

    /// Returns a reference to the underlying references.
    fn get(&self) -> &Self::References;

    /// Returns a mutable reference to the underlying references.
    fn get_mut(&mut self) -> &mut Self::References;

    /// Updates this reference so that all internal references to the `prior_reference` are updated as `new_reference`.
    /// Does nothing if this `NodeRefs` does not hold a reference to the `prior_reference`.
    fn update_reference(
        &mut self,
        prior_reference: &'a Node<'a, V, T>,
        new_reference: &'a Node<'a, V, T>,
    );

    /// Returns an iterator to the referenced nodes.
    fn referenced_nodes(&self) -> impl Iterator<Item = &'a Node<'a, V, T>>
    where
        V: 'a,
        T: 'a;
}
