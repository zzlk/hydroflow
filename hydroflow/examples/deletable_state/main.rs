use std::collections::{HashMap, HashSet};
use std::hash::Hash;

use hydroflow::hydroflow_syntax;
use hydroflow::util::Bag;
use lattices::map_union::{MapUnion, MapUnionHashMap, MapUnionSingletonMap};
use lattices::{DomPair, Max, Merge};

enum Persist<T> {
    Persist(T),
    Unpersist(T),
}

enum PersistKeyed<K, V> {
    Persist(K, V),
    Unpersist(K),
}

trait Persistence<T> {
    type Iter<'a>: Iterator
    where
        Self: 'a;

    fn persist(&mut self, item: T);

    fn iter(&self) -> Self::Iter<'_>;
}

trait PersistenceKeyed<K, V> {
    type Iter<'a>: Iterator
    where
        Self: 'a;

    fn persist(&mut self, k: K, v: V);

    fn iter(&self) -> Self::Iter<'_>;
}

trait PersistenceMut<T>: Persistence<T> {
    fn unpersist(&mut self, item: T);
}

trait PersistenceMutKeyed<K, V>: PersistenceKeyed<K, V> {
    fn unpersist(&mut self, k: K);
}

impl<T> Persistence<T> for Vec<T> {
    type Iter<'a> = std::slice::Iter<'a, T> where T: 'a;

    fn persist(&mut self, item: T) {
        self.push(item);
    }

    fn iter(&self) -> Self::Iter<'_> {
        self.as_slice().iter()
    }
}

impl<T: Hash + Eq> Persistence<T> for HashSet<T> {
    type Iter<'a> = std::collections::hash_set::Iter<'a, T> where T: 'a;

    fn persist(&mut self, item: T) {
        self.insert(item);
    }

    fn iter(&self) -> Self::Iter<'_> {
        HashSet::iter(self)
    }
}

impl<T: Hash + Eq> PersistenceMut<T> for HashSet<T> {
    fn unpersist(&mut self, item: T) {
        self.remove(&item);
    }
}

impl<T: Hash + Eq + Clone> Persistence<T> for Bag<T> {
    type Iter<'a> = hydroflow::util::Iter<'a, T> where T: 'a;

    fn persist(&mut self, item: T) {
        self.insert(item);
    }

    fn iter(&self) -> Self::Iter<'_> {
        Bag::iter(self)
    }
}

impl<T: Hash + Eq + Clone> PersistenceMut<T> for Bag<T> {
    fn unpersist(&mut self, item: T) {
        self.remove(&item);
    }
}

impl<T: Hash + Eq + Clone> Persistence<T> for MapUnionHashMap<T, Max<bool>> {
    type Iter<'a> = std::iter::Once<&'a MapUnionHashMap<T, Max<bool>>> where T: 'a;

    fn persist(&mut self, item: T) {
        Merge::merge(
            self,
            MapUnionSingletonMap::new_from((item, Max::new(false))),
        );
    }

    fn iter(&self) -> Self::Iter<'_> {
        std::iter::once(self)
    }
}

impl<T: Hash + Eq + Clone> PersistenceMut<T> for MapUnionHashMap<T, Max<bool>> {
    fn unpersist(&mut self, item: T) {
        Merge::merge(self, MapUnionSingletonMap::new_from((item, Max::new(true))));
    }
}

impl<T: Hash + Eq + Clone> Persistence<T> for MapUnionHashMap<T, DomPair<Max<bool>, Max<usize>>> {
    type Iter<'a> = std::iter::Once<&'a MapUnionHashMap<T, DomPair<Max<bool>, Max<usize>>>> where T: 'a;

    fn persist(&mut self, item: T) {
        let n = self.0.get(&item).map(|x| x.val.0).unwrap_or(0) + 1;

        Merge::merge(
            self,
            MapUnionSingletonMap::new_from((item, DomPair::new_from(Max::new(false), Max::new(n)))),
        );
    }

    fn iter(&self) -> Self::Iter<'_> {
        std::iter::once(self)
    }
}

impl<T: Hash + Eq + Clone> PersistenceMut<T>
    for MapUnionHashMap<T, DomPair<Max<bool>, Max<usize>>>
{
    fn unpersist(&mut self, item: T) {
        let n = self.0.get(&item).map(|x| x.val.0).unwrap_or(0) + 1;

        Merge::merge(
            self,
            MapUnionSingletonMap::new_from((item, DomPair::new_from(Max::new(true), Max::new(n)))),
        );
    }
}

impl<K: Hash + Eq + Clone, V: Clone> PersistenceKeyed<K, V> for HashMap<K, (V, usize)> {
    type Iter<'a> = HashMapIter<'a, K, V> where K: 'a, V: 'a;

    fn persist(&mut self, k: K, v: V) {
        self.entry(k).or_insert((v, 0)).1 += 1;
    }

    fn iter(&self) -> Self::Iter<'_> {
        HashMapIter {
            parent: HashMap::iter(self),
            repeat: None,
        }
    }
}

impl<K: Hash + Eq + Clone, V: Clone> PersistenceMutKeyed<K, V> for HashMap<K, (V, usize)> {
    fn unpersist(&mut self, k: K) {
        self.remove(&k);
    }
}

impl<K: Hash + Eq + Clone, V: Hash + Eq + Clone> PersistenceKeyed<K, V> for HashMap<K, HashSet<V>> {
    type Iter<'a> = KeyedHashSetIter<'a, K, V> where K: 'a, V: 'a;

    fn persist(&mut self, k: K, v: V) {
        self.entry(k).or_insert(HashSet::default()).insert(v);
    }

    fn iter(&self) -> Self::Iter<'_> {
        let x = self.iter().map(|(k, v)| v.iter().map(move |v| (k, v)));

        KeyedHashSetIter {
            parent: HashMap::iter(self),
            current: None,
        }
    }
}

impl<K: Hash + Eq + Clone, V: Hash + Eq + Clone> PersistenceMutKeyed<K, V>
    for HashMap<K, HashSet<V>>
{
    fn unpersist(&mut self, k: K) {
        self.remove(&k);
    }
}

struct KeyedHashSetIter<'a, K, V> {
    parent: std::collections::hash_map::Iter<'a, K, HashSet<V>>,
    current: Option<(K, std::collections::hash_set::Iter<'a, V>)>,
}

impl<'a, K: Clone, V: Clone> Iterator for KeyedHashSetIter<'a, K, V> {
    type Item = (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some((k, ref mut iter)) = &mut self.current {
            if let Some(v) = iter.next() {
                return Some((k.clone(), v.clone()));
            } else {
                self.current = None;
            }
        }

        if let Some((k, v)) = self.parent.next() {
            let mut iter = v.iter();
            let v = iter.next().unwrap();
            self.current = Some((k.clone(), iter));
            Some((k.clone(), v.clone()))
        } else {
            None
        }
    }
}

struct HashMapIter<'a, K, V> {
    parent: std::collections::hash_map::Iter<'a, K, (V, usize)>,
    repeat: Option<((&'a K, &'a V), usize)>,
}

impl<'a, K: Clone, V: Clone> Iterator for HashMapIter<'a, K, V> {
    type Item = (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(((k, v), ref mut count)) = self.repeat {
            *count -= 1;
            if *count > 0 {
                return Some((k.clone(), v.clone()));
            } else {
                self.repeat = None;
            }
        }

        if let Some((k, (v, count))) = self.parent.next() {
            self.repeat = Some(((k, v), *count));
            Some((k.clone(), v.clone()))
        } else {
            None
        }
    }
}

#[hydroflow::main]
pub async fn main() {
    println!("session example");
    let mut df = hydroflow_syntax! {
        source_iter([(("UserId1", "SessionData"), false), (("UserId2", "SessionData"), false)])
            -> sessions;
        deletes = source_iter([(("UserId1", ""), true)])
            -> next_tick()
            -> sessions;
        sessions = union()
            -> persist_mut_keyed::<HashMap<&str, (&str, usize)>>()
            -> [0]session_map;

        source_iter([("UserId1", ()), ("UserId2", ())])
            -> persist()
            -> [1]session_map;

        session_map = join::<'tick, 'tick>(); // Default join type is 'static

        session_map
            -> for_each(|x| println!("{x:?}"));
    };

    println!("pre-deletion");
    df.run_tick();
    println!("post-deletion");
    df.run_tick();

    println!("Persist into a Vec, this is what persist() does today. No support for deletions.");
    (hydroflow_syntax! {
        source_iter([1, 2, 1, 2, 1])
            -> fold::<'static>(Vec::default(), |mut a, b| { a.push(b); a })
            -> flat_map(|x| x.into_iter())
            -> fold(0, |a, b| a + b)
            -> for_each(|x| println!("1: {x:?}"));

        source_iter([1, 2, 1, 2, 1])
            -> persist()
            -> fold(0, |a, b| a + b)
            -> for_each(|x| println!("2: {x:?}"));
    })
    .run_available();

    println!("Persist into a hash set, no support for deletions.");
    (hydroflow_syntax! {
        source_iter([1, 2, 1, 2, 1])
            -> fold::<'static>(HashSet::default(), |mut a: HashSet<i32>, b| { a.insert(b); a })
            -> flat_map(|x| x.into_iter())
            -> fold(0, |a, b| a + b)
            -> for_each(|x| println!("1: {x:?}"));

        source_iter([1, 2, 1, 2, 1])
            -> persist::<HashSet<_>>()
            -> fold(0, |a, b| a + b)
            -> for_each(|x| println!("2: {x:?}"));
    })
    .run_available();

    println!("Persist into a bag, no support for deletions.");
    (hydroflow_syntax! {
        source_iter([1, 2, 1, 2, 1])
            -> fold::<'static>(Bag::default(), |mut a, b| { a.insert(b); a })
            -> flat_map(|x| x.into_iter())
            -> fold(0, |a, b| a + b)
            -> for_each(|x| println!("1: {x:?}"));

        source_iter([1, 2, 1, 2, 1])
            -> persist::<Bag<_>>()
            -> fold(0, |a, b| a + b)
            -> for_each(|x| println!("2: {x:?}"));
    })
    .run_available();

    println!("Vec does not implement PersistMut");
    // cannot impl

    println!("Persist into set with non-monitone deletions");
    (hydroflow_syntax! {
        source_iter([(1, false), (2, false), (1, false), (2, false), (1, true)])
            -> fold::<'static>(HashSet::default(), |mut a: HashSet<i32>, b| { if !b.1 { a.insert(b.0); a } else { a.remove(&b.0); a } })
            -> flat_map(|x| x.into_iter())
            -> fold(0, |a, b| a + b)
            -> for_each(|x| println!("1: {x:?}"));

        source_iter([(1, false), (2, false), (1, false), (2, false), (1, true)])
            -> persist_mut::<HashSet<_>>()
            -> fold(0, |a, b| a + b)
            -> for_each(|x| println!("2: {x:?}"));
    })
    .run_available();

    println!("Persist into bag with non-monitone deletions and 'delete all' behavior.");
    (hydroflow_syntax! {
        source_iter([(1, false), (2, false), (1, false), (2, false), (1, true)])
            -> fold::<'static>(Bag::default(), |mut a, b| { if !b.1 { a.insert(b.0); a } else { a.remove(&b.0); a } })
            -> flat_map(|x| x.into_iter())
            -> fold(0, |a, b| a + b)
            -> for_each(|x| println!("1: {x:?}"));

        source_iter([(1, false), (2, false), (1, false), (2, false), (1, true)])
            -> persist_mut() // default storage for persist_mut is a bag
            -> fold(0, |a, b| a + b)
            -> for_each(|x| println!("2: {x:?}"));
    })
    .run_available();

    println!("Persist into monitone tombstoned set");
    (hydroflow_syntax! {
        source_iter([(1, false), (2, false), (1, false), (2, false), (1, true)])
            -> fold::<'static>(MapUnionHashMap::<usize, Max<bool>>::default(), |a, b| { Merge::merge_owned(a, MapUnionSingletonMap::new_from((b.0, Max::new(b.1)))) })
            -> flat_map(|x| x.0.into_iter())
            -> filter_map(|x| if x.1.0 { None } else { Some(x.0) })
            -> fold(0, |a, b| a + b)
            -> for_each(|x| println!("1: {x:?}"));

        source_iter([(1, false), (2, false), (1, false), (2, false), (1, true)])
            -> persist_mut::<MapUnionHashMap<usize, Max<bool>>>()
            -> flat_map(|x| x.0.into_iter())
            -> filter_map(|x| if x.1.0 { None } else { Some(x.0) })
            -> fold(0, |a, b| a + b)
            -> for_each(|x| println!("2: {x:?}"));
    })
    .run_available();

    println!("Persist into monitone tombstoned bag");
    (hydroflow_syntax! {
        source_iter([(1, false), (2, false), (1, false), (2, false), (1, true)])
            -> fold::<'static>(MapUnionHashMap::<usize, DomPair<Max<bool>, Max<usize>>>::default(), |a, b| { let n = a.0.get(&b.0).map(|x| x.val.0).unwrap_or(0) + 1; Merge::merge_owned(a, MapUnionSingletonMap::new_from((b.0, DomPair::new_from(Max::new(b.1), Max::new(n))))) })
            -> flat_map(|x| x.0.into_iter())
            -> filter_map(|x| if x.1.key.0 { None } else { Some(std::iter::repeat(x.0).take(x.1.val.0)) })
            -> flatten()
            -> fold(0, |a, b| a + b)
            -> for_each(|x| println!("1: {x:?}"));

        source_iter([(1, false), (2, false), (1, false), (2, false), (1, true)])
            -> persist_mut::<MapUnionHashMap<usize, DomPair<Max<bool>, Max<usize>>>>()
            -> flat_map(|x| x.0.into_iter())
            -> filter_map(|x| if x.1.key.0 { None } else { Some(std::iter::repeat(x.0).take(x.1.val.0)) })
            -> flatten()
            -> fold(0, |a, b| a + b)
            -> for_each(|x| println!("2: {x:?}"));
    })
    .run_available();

    println!("Persist into keyed vec without deletions");
    (hydroflow_syntax! {
        source_iter([(0, 1), (1, 2), (0, 3), (1, 4)])
            -> fold::<'static>(HashMap::default(), |mut a: HashMap<i32, Vec<i32>>, b| { a.entry(b.0).or_insert(Vec::default()).push(b.1); a })
            -> flat_map(|x| x.into_iter().flat_map(|(k, v)| v.into_iter().map(move |v| (k.clone(), v))))
            -> fold_keyed(|| 0, |a: &mut i32, b| *a += b)
            -> fold(Vec::new(), |mut a, b| { a.push(b); a })
            -> for_each(|x| println!("1: {x:?}"));

        source_iter([(0, 1), (1, 2), (0, 3), (1, 4)])
            -> persist::<Vec<_>>()
            -> fold_keyed(|| 0, |a: &mut i32, b| *a += b)
            -> fold(Vec::new(), |mut a, b| { a.push(b); a })
            -> for_each(|x| println!("2: {x:?}"));
    })
    .run_available();

    println!("Persist into keyed set without deletions");
    (hydroflow_syntax! {
        source_iter([(0, 1), (1, 2), (0, 1), (1, 4)])
            -> fold::<'static>(HashMap::default(), |mut a: HashMap<i32, HashSet<i32>>, b| { a.entry(b.0).or_insert(HashSet::default()).insert(b.1); a })
            -> flat_map(|x| x.into_iter().flat_map(|(k, v)| v.into_iter().map(move |v| (k.clone(), v))))
            -> fold_keyed(|| 0, |a: &mut i32, b| *a += b)
            -> fold(Vec::new(), |mut a, b| { a.push(b); a })
            -> for_each(|x| println!("1: {x:?}"));

        source_iter([(0, 1), (1, 2), (0, 1), (1, 4)])
            -> persist::<HashSet<_>>()
            -> fold_keyed(|| 0, |a: &mut i32, b| *a += b)
            -> fold(Vec::new(), |mut a, b| { a.push(b); a })
            -> for_each(|x| println!("2: {x:?}"));
    })
    .run_available();

    println!("Persist into keyed bag without deletions");
    (hydroflow_syntax! {
        source_iter([(0, 1), (1, 2), (0, 1), (1, 4)])
            -> fold::<'static>(HashMap::default(), |mut a: HashMap<i32, Bag<i32>>, b| { a.entry(b.0).or_insert(Bag::default()).insert(b.1); a })
            -> flat_map(|x| x.into_iter().flat_map(|(k, v)| v.into_iter().map(move |v| (k.clone(), v))))
            -> fold_keyed(|| 0, |a: &mut i32, b| *a += b)
            -> fold(Vec::new(), |mut a, b| { a.push(b); a })
            -> for_each(|x| println!("1: {x:?}"));

        source_iter([(0, 1), (1, 2), (0, 1), (1, 4)])
            -> persist::<Bag<_>>()
            -> fold_keyed(|| 0, |a: &mut i32, b| *a += b)
            -> fold(Vec::new(), |mut a, b| { a.push(b); a })
            -> for_each(|x| println!("2: {x:?}"));
    })
    .run_available();

    println!("Vec does not implement PersistenceMutKeyed");
    // cannot impl

    println!("Persist into keyed set with non-monitone deletions");
    (hydroflow_syntax! {
        source_iter([((0, 1), false), ((0, 2), false), ((1, 3), false), ((1, 4), false), ((0, 0), true)])
            -> fold::<'static>(HashMap::default(), |mut a: HashMap<i32, HashSet<i32>>, b| { if !b.1 { a.entry(b.0.0).or_insert(HashSet::default()).insert(b.0.1); a } else { a.remove(&b.0.0); a } })
            -> flat_map(|x| x.into_iter().flat_map(|(k, v)| v.into_iter().map(move |v| (k.clone(), v))))
            -> fold_keyed(|| 0, |a: &mut i32, b| *a += b)
            -> fold(Vec::new(), |mut a, b| { a.push(b); a })
            -> for_each(|x| println!("1: {x:?}"));

        source_iter([((0, 1), false), ((0, 2), false), ((1, 3), false), ((1, 4), false), ((0, 0), true)])
            -> persist_mut_keyed::<HashMap<_, HashSet<_>>>()
            -> fold_keyed(|| 0, |a: &mut i32, b| *a += b)
            -> fold(Vec::new(), |mut a, b| { a.push(b); a })
            -> for_each(|x| println!("2: {x:?}"));
    })
    .run_available();
}

// // println!("{}", df2.meta_graph().unwrap().to_mermaid());
