//! Module containing the [`SetUnion`] lattice and aliases for different datastructures.

use std::cmp::Ordering::{self, *};
use std::collections::{BTreeSet, HashSet};
use std::hash::Hash;

use crate::cc_traits::{Iter, Len, Set};
use crate::collections::{ArraySet, SingletonSet};
use crate::set_union::SetUnion;
use crate::{ConvertFrom, LatticeOrd, Merge};

/// Set-union lattice.
///
/// Merging set-union lattices is done by unioning the keys.
#[derive(Clone, Debug, Default)]
// #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SetUnionWithTombstone<Set, TombstoneSet> {
    pub set: SetUnion<Set>,
    pub tombstones: SetUnion<TombstoneSet>,
}

impl<Item> SetUnionWithTombstone<Item> {
    /// Create a new `SetUnion` from a `Set`.
    pub fn new(set: HashSet<Item>, tombstones: HashSet<Item>) -> Self {
        Self { set, tombstones }
    }

    /// Create a new `SetUnion` from an `Into<Set>`.
    pub fn new_from(
        set: impl Into<HashSet<Item>>,
        tombstone_val: impl Into<HashSet<Item>>,
    ) -> Self {
        Self::new(set.into(), tombstone_val.into())
    }
}

impl<Item> Merge<SetUnionWithTombstone<Item>> for SetUnionWithTombstone<Item>
where
    Item: Hash + Eq,
{
    fn merge(&mut self, other: SetUnionWithTombstone<Item>) -> bool {
        let set_old_len = {
            let old_len = self.set.len();
            self.set.extend(other.set.into_iter());
            self.set.len() > old_len
        };

        let tombstone_set_old_len = {
            let old_len = self.tombstones.len();
            self.tombstones.extend(other.tombstones.into_iter());
            self.tombstones.len() > old_len
        };

        set_old_len || tombstone_set_old_len
    }
}

impl<Item> ConvertFrom<SetUnionWithTombstone<Item>> for SetUnionWithTombstone<Item>
where
    Item: Hash + Eq,
{
    fn from(other: SetUnionWithTombstone<Item>) -> Self {
        Self {
            set: other.set.into_iter().collect(),
            tombstones: other.tombstones.into_iter().collect(),
        }
    }
}

impl<Item> PartialOrd<SetUnionWithTombstone<Item>> for SetUnionWithTombstone<Item>
where
    Item: Hash + Eq,
{
    fn partial_cmp(&self, other: &SetUnionWithTombstone<Item>) -> Option<Ordering> {
        match self.0.len().cmp(&other.0.len()) {
            Greater => {
                if other.0.iter().all(|key| self.0.contains(&*key)) {
                    Some(Greater)
                } else {
                    None
                }
            }
            Equal => {
                if self.0.iter().all(|key| other.0.contains(&*key)) {
                    Some(Equal)
                } else {
                    None
                }
            }
            Less => {
                if self.0.iter().all(|key| other.0.contains(&*key)) {
                    Some(Less)
                } else {
                    None
                }
            }
        }
    }
}
impl<Item> LatticeOrd<SetUnionWithTombstone<Item>> for SetUnionWithTombstone<Item> where
    Item: Hash + Eq
{
}

impl<SetSelf, SetOther, Item> PartialEq<SetUnionWithTombstone<SetOther>>
    for SetUnionWithTombstone<SetSelf>
where
    SetSelf: Set<Item, Item = Item> + Iter,
    SetOther: Set<Item, Item = Item> + Iter,
{
    fn eq(&self, other: &SetUnionWithTombstone<SetOther>) -> bool {
        if self.0.len() != other.0.len() {
            return false;
        }

        self.0.iter().all(|key| other.0.contains(&*key))
    }
}
impl<SetSelf> Eq for SetUnionWithTombstone<SetSelf> where Self: PartialEq {}

/// [`std::collections::HashSet`]-backed [`SetUnion`] lattice.
pub type SetUnionHashSet<Item> = SetUnionWithTombstone<HashSet<Item>>;

/// [`std::collections::BTreeSet`]-backed [`SetUnion`] lattice.
pub type SetUnionBTreeSet<Item> = SetUnionWithTombstone<BTreeSet<Item>>;

/// [`Vec`]-backed [`SetUnion`] lattice.
pub type SetUnionVec<Item> = SetUnionWithTombstone<Vec<Item>>;

/// [`crate::collections::ArraySet`]-backed [`SetUnion`] lattice.
pub type SetUnionArray<Item, const N: usize> = SetUnionWithTombstone<ArraySet<Item, N>>;

/// [`crate::collections::SingletonSet`]-backed [`SetUnion`] lattice.
pub type SetUnionSingletonSet<Item> = SetUnionWithTombstone<SingletonSet<Item>>;

/// [`Option`]-backed [`SetUnion`] lattice.
pub type SetUnionOption<Item> = SetUnionWithTombstone<Option<Item>>;

// #[cfg(test)]
// mod test {
//     use super::*;
//     use crate::collections::SingletonSet;
//     use crate::test::check_all;

//     #[test]
//     fn test_set_union() {
//         let mut my_set_a = SetUnionHashSet::<&str>::new(HashSet::new());
//         let my_set_b = SetUnionBTreeSet::<&str>::new(BTreeSet::new());
//         let my_set_c = SetUnionSingletonSet::new(SingletonSet("hello world"));

//         assert_eq!(Some(Equal), my_set_a.partial_cmp(&my_set_a));
//         assert_eq!(Some(Equal), my_set_a.partial_cmp(&my_set_b));
//         assert_eq!(Some(Less), my_set_a.partial_cmp(&my_set_c));
//         assert_eq!(Some(Equal), my_set_b.partial_cmp(&my_set_a));
//         assert_eq!(Some(Equal), my_set_b.partial_cmp(&my_set_b));
//         assert_eq!(Some(Less), my_set_b.partial_cmp(&my_set_c));
//         assert_eq!(Some(Greater), my_set_c.partial_cmp(&my_set_a));
//         assert_eq!(Some(Greater), my_set_c.partial_cmp(&my_set_b));
//         assert_eq!(Some(Equal), my_set_c.partial_cmp(&my_set_c));

//         assert!(!my_set_a.merge(my_set_b));
//         assert!(my_set_a.merge(my_set_c));
//     }

//     #[test]
//     fn test_singleton_example() {
//         let mut my_hash_set = SetUnionHashSet::<&str>::default();
//         let my_delta_set = SetUnionSingletonSet::new_from("hello world");
//         let my_array_set = SetUnionArray::new_from(["hello world", "b", "c", "d"]);

//         assert_eq!(Some(Equal), my_delta_set.partial_cmp(&my_delta_set));
//         assert_eq!(Some(Less), my_delta_set.partial_cmp(&my_array_set));
//         assert_eq!(Some(Greater), my_array_set.partial_cmp(&my_delta_set));
//         assert_eq!(Some(Equal), my_array_set.partial_cmp(&my_array_set));

//         assert!(my_hash_set.merge(my_array_set)); // Changes
//         assert!(!my_hash_set.merge(my_delta_set)); // No changes
//     }

//     #[test]
//     fn consistency() {
//         check_all(&[
//             SetUnionHashSet::new_from([]),
//             SetUnionHashSet::new_from([0]),
//             SetUnionHashSet::new_from([1]),
//             SetUnionHashSet::new_from([0, 1]),
//         ]);
//     }
// }
