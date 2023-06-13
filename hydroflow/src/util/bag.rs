use std::collections::hash_map;
use std::iter::repeat;

use rustc_hash::FxHashMap;

/// TODO:
#[derive(Default, Clone)]
pub struct Bag<T> {
    items: FxHashMap<T, usize>,
    len: usize,
}

impl<T: Eq + std::hash::Hash + Clone> Bag<T> {
    /// TODO:
    pub fn insert(&mut self, item: T) {
        *self.items.entry(item).or_insert(0) += 1;
        self.len += 1;
    }

    /// TODO:
    pub fn remove_one(&mut self, item: T) {
        match self.items.entry(item) {
            std::collections::hash_map::Entry::Occupied(mut entry) => {
                if *entry.get() == 1 {
                    entry.remove();
                    self.len -= 1;
                } else {
                    *entry.get_mut() -= 1;
                    self.len -= 1;
                }
            }
            std::collections::hash_map::Entry::Vacant(_) => {}
        }
    }

    /// TODO:
    pub fn remove(&mut self, item: &T) {
        if let Some(num_items) = self.items.remove(item) {
            self.len -= num_items;
        }
    }

    /// TODO:
    pub fn len(&self) -> usize {
        self.len
    }

    /// TODO:
    pub fn extend(&mut self, iter: impl Iterator<Item = T>) {
        for item in iter {
            self.insert(item);
        }
    }

    /// TODO:
    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            parent: self.items.iter(),
            repeat: None,
        }
    }

    /// TODO:
    pub fn into_iter(self) -> impl Iterator<Item = T> {
        self.items.into_iter().flat_map(|x| repeat(x.0).take(x.1))
    }
}

/// TODO:
pub struct Iter<'a, T>
where
    T: 'a + Clone,
{
    parent: hash_map::Iter<'a, T, usize>,
    repeat: Option<(&'a T, usize)>,
}

impl<'a, T: Clone> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some((item, ref mut count)) = self.repeat {
            *count -= 1;
            if *count > 0 {
                return Some(item);
            } else {
                self.repeat = None;
            }
        }

        if let Some((element, count)) = self.parent.next() {
            self.repeat = Some((element, *count));
            Some(element)
        } else {
            None
        }
    }
}
