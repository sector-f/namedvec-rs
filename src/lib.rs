use std::collections::hash_map::HashMap;
use std::ops::{Index, Range, RangeFrom, RangeFull, RangeTo};

/// Vector where each element has an associated name.
///
/// Elements must implement the [`Named`](trait.Named.html) trait so that they can be accessed
/// by name. Each element's name must be unique; calling [`push()`](#method.push) will update
/// an existing element rather than add a new one if the new element's name is in use by
/// an existing element.
///
/// Internally, a `NamedVec<T>` is a wrapper around a `Vec<T>`, with names
/// and their corresponding indices stored as a `HashMap<String, usize>`.
#[derive(Debug, PartialEq)]
pub struct NamedVec<T: Named> {
    map: HashMap<String, usize>,
    items: Vec<T>,
}

impl<T: Named> NamedVec<T> {
    /// Creates an empty `NamedVec<T>`.
    pub fn new() -> Self {
        NamedVec {
            map: HashMap::new(),
            items: Vec::new(),
        }
    }

    /// Creates an empty `NamedVec<T>` with the specified capacity.
    ///
    /// The vector will be able to hold exactly `capacity` elements without
    /// relocating. If `capacity` is 0, the vector will not allocate.
    pub fn with_capacity(capacity: usize) -> Self {
        NamedVec {
            map: HashMap::with_capacity(capacity),
            items: Vec::with_capacity(capacity),
        }
    }

    /// Appends an element to the back of the collection,
    /// or replaces an element with the same name if one exists.
    pub fn push(&mut self, item: T) {
        match self.map.get(item.name()).map(|n| n.clone()) {
            Some(i) => {
                self.items[i] = item;
            },
            None => {
                self.map.insert(item.name().to_owned(), self.items.len());
                self.items.push(item);
            },
        }
    }

    /// Returns the number of elements the vector can hold without reallocating.
    pub fn capacity(&self) -> usize {
        self.items.capacity()
    }

    /// Reserves capacity for at least `additional` more elements to be inserted in
    /// the `NamedVec`. The collection may reserve more space to avoid frequent
    /// reallocations.
    ///
    /// # Panics
    ///
    /// Panics if the new allocation size overflows `usize`.
    pub fn reserve(&mut self, additional: usize) {
        self.items.reserve(additional);
        self.map.reserve(additional);
    }

    /// Shrinks the capacity as much as possible.
    pub fn shrink_to_fit(&mut self) {
        self.items.shrink_to_fit();
        self.map.shrink_to_fit();
    }

    /// Shortens the vector, keeping the first len elements and dropping the rest.
    ///
    /// If `len` is greater than the vector's current length, this has no effect.
    pub fn truncate(&mut self, len: usize) {
        if len < self.len() {
            let max = self.map.values().max().map(|n| n.clone()).unwrap();
            for item in self.items[len..max+1].iter() {
                let name = item.name();
                self.map.remove(name);
            }
            self.items.truncate(len);
        }
    }

    /// Clears the vector, removing all values.
    pub fn clear(&mut self) {
        self.map.clear();
        self.items.clear();
    }

    /// Returns `true` if the vector contains no elements.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns a reference to an element.
    ///
    /// This function's argument can be a `usize`, e.g. `named_vec.get(0)`,
    /// or a `&str`, e.g. `named_vec.get("foo")`.
    /// These will access elements by position or name, respectively.
    ///
    /// Returns `None` if a `usize` argument is out of bounds or if
    /// a `&str` argument refers to a nonexistent element.
    pub fn get<'a, A: 'a>(&self, lookup: A) -> Option<&T> where A: Into<Lookup<'a>> {
        self.index_from_lookup(lookup.into()).and_then(|i| self.items.get(i))
    }

    /// Returns a mutable reference to an element.
    ///
    /// See [`get()`](#method.get) for more information.
    pub fn get_mut <'a, A: 'a>(&mut self, lookup: A) -> Option<&mut T>
    where A: Into<Lookup<'a>> {
        self.index_from_lookup(lookup.into()).and_then(move |i| self.items.get_mut(i))
    }

    /// Swaps two elements.
    ///
    /// Each element can be either a `usize` or a `&str`.
    /// See [`get()`](#method.get) for more information on arguments.
    ///
    /// # Panics
    ///
    /// * Panics if a `usize` argument is out of bounds.
    /// * Panics if a `&str` argument is an invalid name.
    pub fn swap<'a, 'b, A: 'a, B: 'b>(&mut self, first: A, second: B)
    where A: Into<Lookup<'a>> + Copy, B: Into<Lookup<'b>> + Copy {
        let old_i1 = self.index_from_lookup(first.into()).unwrap();
        let old_i2 = self.index_from_lookup(second.into()).unwrap();

        // Don't bother swapping (and allocating Strings!) if the two items are the same
        if old_i1 == old_i2 {
            return;
        }

        let old_s1 = self.name_from_lookup(first.into()).unwrap();
        let old_s2 = self.name_from_lookup(second.into()).unwrap();

        self.map.insert(old_s1, old_i2);
        self.map.insert(old_s2, old_i1);
        self.items.swap(old_i1, old_i2);
    }

    /// Returns the number of elements in the vector.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Removes the last element from the vector and returns in, or `None` if it is empty.
    pub fn pop(&mut self) -> Option<T> {
        if self.items.len() == 0 {
            None
        } else {
            let last_item = self.items.pop().unwrap();
            self.map.remove(last_item.name());
            Some(last_item)
        }
    }

    fn index_from_lookup(&self, lookup: Lookup) -> Option<usize> {
        match lookup {
            Lookup::Name(name) => {
                self.map.get(name).cloned()
            },
            Lookup::Index(index) => {
                Some(index)
            },
        }
    }

    fn name_from_lookup(&self, lookup: Lookup) -> Option<String> {
        match lookup {
            Lookup::Name(name) => {
                Some(name.to_owned())
            },
            Lookup::Index(index) => {
                self.items.get(index).and_then(|s| Some(String::from(s.name())))
            },
        }
    }
}

///////////
// Index //
///////////

impl<'a, T: Named> Index<&'a str> for NamedVec<T> {
    type Output = T;

    fn index(&self, index: &str) -> &T {
        self.get(index).unwrap()
    }
}

impl<T: Named> Index<usize> for NamedVec<T> {
    type Output = T;

    fn index(&self, index: usize) -> &T {
        &self.items[index]
    }
}

impl<T: Named> Index<Range<usize>> for NamedVec<T> {
    type Output = [T];

    fn index(&self, index: Range<usize>) -> &[T] {
        &self.items[index]
    }
}

impl<T: Named> Index<RangeTo<usize>> for NamedVec<T> {
    type Output = [T];

    fn index(&self, index: RangeTo<usize>) -> &[T] {
        &self.items[index]
    }
}

impl<T: Named> Index<RangeFrom<usize>> for NamedVec<T> {
    type Output = [T];

    fn index(&self, index: RangeFrom<usize>) -> &[T] {
        &self.items[index]
    }
}

impl<T: Named> Index<RangeFull> for NamedVec<T> {
    type Output = [T];

    fn index(&self, _index: RangeFull) -> &[T] {
        &self.items
    }
}

///////////
// Named //
///////////

pub trait Named {
    fn name(&self) -> &str;
}

////////////
// Lookup //
////////////

/// Used to refer to elements in a `NamedVec`.
///
/// However, `NamedVec`'s methods
/// are designed to avoid making the user have to create a `Lookup`.
/// In other words, prefer `named_vec.get("foo")` to `named_vec.get(Lookup::Name("foo"))`.
pub enum Lookup<'a> {
    Name(&'a str),
    Index(usize),
}

impl<'a> From<&'a str> for Lookup<'a> {
    fn from(s: &'a str) -> Self {
        Lookup::Name(s)
    }
}

impl<'a> From<usize> for Lookup<'a> {
    fn from(i: usize) -> Self {
        Lookup::Index(i)
    }
}

/////////////////
// MultiLookup //
/////////////////

// This won't be useful until std::slice::SliceIndex is stable
enum MultiLookup<'a> {
    Name(&'a str),
    Index(usize),
    Range(Range<usize>),
    RangeFrom(RangeFrom<usize>),
    RangeTo(RangeTo<usize>),
    RangeFull(RangeFull),
}

impl<'a> From<&'a str> for MultiLookup<'a> {
    fn from(s: &'a str) -> Self {
        MultiLookup::Name(s)
    }
}

impl<'a> From<usize> for MultiLookup<'a> {
    fn from(i: usize) -> Self {
        MultiLookup::Index(i)
    }
}

impl<'a> From<Range<usize>> for MultiLookup<'a> {
    fn from(i: Range<usize>) -> Self {
        MultiLookup::Range(i)
    }
}

impl<'a> From<RangeFrom<usize>> for MultiLookup<'a> {
    fn from(i: RangeFrom<usize>) -> Self {
        MultiLookup::RangeFrom(i)
    }
}

impl<'a> From<RangeTo<usize>> for MultiLookup<'a> {
    fn from(i: RangeTo<usize>) -> Self {
        MultiLookup::RangeTo(i)
    }
}

impl<'a> From<RangeFull> for MultiLookup<'a> {
    fn from(i: RangeFull) -> Self {
        MultiLookup::RangeFull(i)
    }
}
