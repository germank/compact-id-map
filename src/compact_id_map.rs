use std::{borrow::Borrow, fmt::Debug, hash::Hash};

use hashbrown::HashMap;
use increment::Incrementable;
use serde::{Deserialize, Serialize};

pub type ID = usize;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CompactIdBiMap<K>
    where K: Eq + Hash
{
    ids: HashMap<K, ID>,
    keys: HashMap<ID, K>,
    recycle_bin: Vec<ID>,
    next_new_id: ID,
}

impl<K> CompactIdBiMap<K> where
K: Hash + Eq
{
    pub fn new() -> Self {
        Self {
            ids: HashMap::new(),
            keys: HashMap::new(),
            recycle_bin: Vec::new(),
            next_new_id: 0,
        }
    }

    pub fn get_or_insert(&mut self, k: K) -> ID
    where
        K: Clone + Debug,
    {
        self.get(&k).unwrap_or_else(||self.insert(k))
    }

    pub fn get<Q: ?Sized>(&self, key: &Q) -> Option<ID>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + Debug
    {
        self.ids.get(key).copied()
    }

    pub fn get_key(&self, id: ID) -> Option<&K> {
        self.keys.get(&id)
    }

    pub fn insert(&mut self, k: K) -> ID
    where
        K: Clone,
    {
        let id = self.get_fresh_id();
        debug_assert!(!self.ids.contains_key(&k));
        self.ids.insert(k.clone(), id);
        self.keys.insert(id, k);
        id
    }

    fn get_fresh_id(&mut self) -> ID {
        if self.recycle_bin.is_empty() {
            let id = self.next_new_id;
            self.next_new_id += 1;
            id
        } else {
            self.recycle_bin.pop().unwrap()
        }
    }

    pub fn remove<Q: ?Sized>(&mut self, k: &Q) -> Option<ID>
    where
        K: Borrow<Q>,
        Q: Hash + Eq
    {
        self.ids.remove(k).map(|id| {
            self.recycle_bin.push(id);
            self.keys.remove(&id);
            id
        })
    }

    pub fn remove_id(&mut self, id: ID) -> Option<K> {
        self.keys.remove(&id).map(|k| {
            self.recycle_bin.push(id);
            self.ids.remove(&k);
            k
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CompactIdMap<I: Eq + Hash, K>
{
    keys: HashMap<I, K>,
    recycle_bin: Vec<I>,
    next_new_id: I,
}

impl<I, K> CompactIdMap<I, K> where
I: Hash + Incrementable + Default + Eq + Copy
{
    pub fn new() -> Self {
        Self {
            keys: HashMap::new(),
            recycle_bin: Vec::new(),
            next_new_id: Default::default(),
        }
    }

    pub fn get(&self, id: I) -> Option<&K> {
        self.keys.get(&id)
    }

    pub fn get_mut(&mut self, id: I) -> Option<&mut K> {
        self.keys.get_mut(&id)
    }

    pub fn insert(&mut self, k: K) -> I
    where
        K: Clone,
    {
        let id = self.fresh_id();
        self.keys.insert(id, k);
        id
    }

    fn fresh_id(&mut self) -> I {
        if self.recycle_bin.is_empty() {
            let id = self.next_new_id;
            self.next_new_id = self.next_new_id.increment().unwrap();
            id
        } else {
            self.recycle_bin.pop().unwrap()
        }
    }

    pub fn remove_id(&mut self, id: I) -> Option<K> {
        self.keys.remove(&id).map(|k| {
            self.recycle_bin.push(id);
            k
        })
    }
}

#[cfg(test)]
mod test_compact_id_alloc {
    use super::*;

    #[test]
    fn test_ids() {
        let mut ids = CompactIdBiMap::new();
        assert_eq!(0, ids.insert(String::from("hello")));
        assert_eq!(1, ids.insert(String::from("how")));
        assert_eq!(Some(0), ids.get("hello"));
        assert_eq!(Some(&String::from("hello")), ids.get_key(0));
        assert_eq!(Some(0), ids.remove("hello"));
        assert_eq!(None, ids.get("hello"));
        assert_eq!(None, ids.remove("hello"));
        assert_eq!(0, ids.insert(String::from("are")));
        assert_eq!(2, ids.insert(String::from("you")));
        assert_eq!(Some(0), ids.get("are"));
        assert_eq!(Some(&String::from("are")), ids.get_key(0));
        assert_eq!(Some(2), ids.get("you"));
        assert_eq!(Some(&String::from("you")), ids.get_key(2));
    }
}