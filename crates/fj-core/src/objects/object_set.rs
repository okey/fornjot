use std::{collections::BTreeSet, fmt::Debug, slice, vec};

use itertools::Itertools;

use crate::storage::Handle;

/// An ordered set of objects
///
/// This is the data structure used by all objects that reference multiple
/// objects of the same type. It is a set, not containing any duplicate
/// elements, and it maintains the insertion order of those elements.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct ObjectSet<T> {
    // This is supposed to be a set data structure, so what is that `Vec` doing
    // here? Well, it's here because we need it to preserve insertion order, but
    // that doesn't explain why it is here *alone*.
    //
    // If you look closely, you'll notice that this is an immutable data
    // structure (since it is used in objects, and objects themselves are
    // immutable). We need to make sure there are no duplicates when this is
    // constructed (see the constructor below), but after that, we're fine.
    inner: Vec<Handle<T>>,
}

impl<T> ObjectSet<T> {
    /// Create an instances of `ObjectSet` from an iterator over `Handle<T>`
    ///
    /// # Panics
    ///
    /// Panics, if the iterator contains duplicate `Handle`s.
    pub fn new(handles: impl IntoIterator<Item = Handle<T>>) -> Self
    where
        T: Debug + Ord,
    {
        let mut added = BTreeSet::new();
        let mut inner = Vec::new();

        for handle in handles {
            if added.contains(&handle) {
                panic!(
                    "Constructing `ObjectSet` with duplicate handle: {:?}",
                    handle
                );
            }

            added.insert(handle.clone());
            inner.push(handle);
        }

        Self { inner }
    }

    /// Return the number of objects in this set
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Indicate whether the set is empty
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Indicate whether the set contains the provided object
    pub fn contains(&self, object: &Handle<T>) -> bool {
        self.index_of(object).is_some()
    }

    /// Return the only item
    ///
    /// # Panics
    ///
    /// Panics, if there is more than one item.
    pub fn only(&self) -> &Handle<T> {
        let mut iter = self.inner.iter();
        let item = iter
            .next()
            .expect("Requested only item, but no items available");

        assert!(
            iter.next().is_none(),
            "Requested only item, but more than one available"
        );

        item
    }

    /// Return the first item
    ///
    /// # Panics
    ///
    /// Panics, if there are no items.
    pub fn first(&self) -> &Handle<T> {
        self.inner
            .first()
            .expect("Requested first item, but no items available")
    }

    /// Return the n-th item
    pub fn nth(&self, index: usize) -> Option<&Handle<T>> {
        self.inner.get(index)
    }

    /// Return the n-th item, treating the index space as circular
    ///
    /// If the length of `ObjectSet` is `i`, then retrieving the i-th edge using
    /// this method, is the same as retrieving the 0-th one, and so on.
    ///
    /// # Panics
    ///
    /// Panics, if `ObjectSet` is empty.
    pub fn nth_circular(&self, index: usize) -> &Handle<T> {
        assert!(!self.is_empty(), "`ObjectSet` must not be empty");

        let index = index % self.len();
        self.nth(index)
            .expect("Index must be valid, due to modulo above")
    }

    /// Return the index of the item, if available
    pub fn index_of(&self, handle: &Handle<T>) -> Option<usize> {
        self.inner.iter().position(|h| h.id() == handle.id())
    }

    /// Access the item after the provided one
    ///
    /// Returns `None`, if the provided item is not in this iterator.
    pub fn after(&self, handle: &Handle<T>) -> Option<&Handle<T>> {
        self.index_of(handle)
            .map(|index| self.nth_circular(index + 1))
    }

    /// Access an iterator over the objects
    pub fn iter(&self) -> slice::Iter<Handle<T>> {
        self.inner.iter()
    }

    /// Access an iterator over the neighboring pairs of all contained objects
    pub fn pairs(&self) -> impl Iterator<Item = (&Handle<T>, &Handle<T>)> {
        self.iter().circular_tuple_windows()
    }

    /// Create a new instance in which the provided object has been replaced
    ///
    /// Returns `None`, if the provided item is not present.
    ///
    /// # Panics
    ///
    /// Panics, if the update results in a duplicate item.
    #[must_use]
    pub fn replace<const N: usize>(
        &self,
        original: &Handle<T>,
        replacements: [Handle<T>; N],
    ) -> Option<Self>
    where
        T: Debug + Ord,
    {
        let mut iter = self.iter().cloned().peekable();

        // Collect all items before the item we want to update.
        let mut before = Vec::new();
        loop {
            let next = match iter.next() {
                Some(handle) => handle,
                None => {
                    // We went through the whole iterator without finding the
                    // item we were looking for.
                    return None;
                }
            };

            if next.id() == original.id() {
                break;
            }

            before.push(next.clone());
        }

        // What's left in the iterator is what comes after the replaced item.
        // Let's make that a bit more explicit by renaming the variable.
        let after = iter;

        Some(
            before
                .into_iter()
                .chain(replacements)
                .chain(after)
                .collect(),
        )
    }
}

impl<O> FromIterator<Handle<O>> for ObjectSet<O>
where
    O: Debug + Ord,
{
    fn from_iter<T: IntoIterator<Item = Handle<O>>>(handles: T) -> Self {
        Self::new(handles)
    }
}

impl<T> IntoIterator for ObjectSet<T> {
    type Item = Handle<T>;
    type IntoIter = vec::IntoIter<Handle<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

impl<'r, T> IntoIterator for &'r ObjectSet<T> {
    // You might wonder why we're returning references to handles here, when
    // `Handle` already is kind of reference, and easily cloned.
    //
    // Most of the time that doesn't make a difference, but there are use cases
    // where dealing with owned `Handle`s is inconvenient, for example when
    // using iterator adapters. You can't return a reference to the argument of
    // an adapter's closure, if you own that argument. You can, if you just
    // reference the argument.
    type Item = &'r Handle<T>;
    type IntoIter = slice::Iter<'r, Handle<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
