use std::collections::HashMap;

#[derive(Clone)]
pub struct SlotMap<T> {
    counter: usize,
    values: HashMap<usize, T>,
}

impl<T> SlotMap<T> {
    pub fn new() -> Self {
        Self {
            counter: 0,
            values: HashMap::new(),
        }
    }

    pub fn push(&mut self, val: T) -> usize {
        let i = self.counter;
        self.counter += 1;
        self.values.insert(i, val);
        i
    }

    pub fn pop(&mut self) -> Option<(usize, T)> {
        let i = *self.values.keys().next()?;
        Some((i, self.take(i)?))
    }

    pub fn take(&mut self, i: usize) -> Option<T> {
        self.values.remove(&i)
    }

    pub fn insert(&mut self, i: usize, val: T) {
        assert!(!self.values.contains_key(&i));
        if self.counter <= i {
            self.counter = i + 1;
        }
        self.values.insert(i, val);
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    pub fn count(&self) -> usize {
        self.values.len()
    }

    pub fn iter(&self) -> IterSlotMap<T> {
        IterSlotMap {
            iter: self.values.iter(),
        }
    }
}

impl<T> std::ops::Index<usize> for SlotMap<T> {
    type Output = T;

    fn index(&self, i: usize) -> &T {
        &self.values[&i]
    }
}

impl<T> std::ops::IndexMut<usize> for SlotMap<T> {
    fn index_mut(&mut self, i: usize) -> &mut T {
        self.values.get_mut(&i).unwrap()
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for SlotMap<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map().entries(self.iter()).finish()
    }
}

impl<T> IntoIterator for SlotMap<T> {
    type Item = (usize, T);
    type IntoIter = std::collections::hash_map::IntoIter<usize, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.values.into_iter()
    }
}

impl<'a, T: 'a> IntoIterator for &'a SlotMap<T> {
    type Item = (usize, &'a T);
    type IntoIter = IterSlotMap<'a, T>;

    fn into_iter(self) -> IterSlotMap<'a, T> {
        self.iter()
    }
}

impl<T> std::iter::FromIterator<T> for SlotMap<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut map = SlotMap::new();
        for val in iter {
            map.push(val);
        }
        map
    }
}

pub struct IterSlotMap<'a, T> {
    iter: std::collections::hash_map::Iter<'a, usize, T>,
}

impl<'a, T: 'a> Iterator for IterSlotMap<'a, T> {
    type Item = (usize, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        let (i, val) = self.iter.next()?;
        Some((*i, val))
    }
}
