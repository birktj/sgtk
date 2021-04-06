use crate::bitset::{Bitset, Bitset16};
use std::mem::MaybeUninit;

pub type Map16<T> = Map<T, Bitset16, 16>;

pub struct Map<T, B: Bitset, const N: usize> {
    occupied: B,
    values: [MaybeUninit<T>; N],
}

impl<T, B: Bitset, const N: usize> Map<T, B, N> {
    const UNINIT: MaybeUninit<T> = MaybeUninit::uninit();

    pub fn new() -> Self {
        Self {
            occupied: B::new(),
            values: [Self::UNINIT; N],
        }
    }

    pub fn push(&mut self, val: T) -> usize {
        let i = self.occupied.invert().smallest().expect("No space left in Map16");
        self.occupied.set(i);
        self.values[i] = MaybeUninit::new(val);
        i
    }

    pub fn pop(&mut self) -> Option<(usize, T)> {
        let i = self.occupied.smallest()?;
        Some((i, self.take(i)?))
    }

    pub fn take(&mut self, i: usize) -> Option<T> {
        assert!(self.occupied.get(i));
        self.occupied.clear(i);
        let res = std::mem::replace(&mut self.values[i], MaybeUninit::uninit());

        unsafe {
            Some(res.assume_init())
        }
    }

    pub fn insert(&mut self, i: usize, val: T) {
        assert!(!self.occupied.get(i));
        self.occupied.set(i);
        self.values[i] = MaybeUninit::new(val);
    }

    pub fn swap(&mut self, i: usize, j: usize) {
        self.occupied.swap(i, j);
        self.values.swap(i, j);
    }

    pub fn is_empty(&self) -> bool {
        self.occupied.is_empty()
    }

    pub fn count(&self) -> usize {
        self.occupied.count()
    }
}

impl<T, B: Bitset + Clone, const N: usize> Map<T, B, N> {
    pub fn iter(&self) -> IterMap<T, B, N> {
        IterMap {
            map: &self,
            elems: self.occupied.clone(),
        }
    }
}

impl<T: Clone, B: Bitset + Clone, const N: usize> Clone for Map<T, B, N> {
    fn clone(&self) -> Self {
        let mut values = [Self::UNINIT; N];
        for (i, v) in self.iter() {
            values[i] = MaybeUninit::new(v.clone());
        }
        Self {
            occupied: self.occupied.clone(),
            values,
        }
    }
}

impl<T, B: Bitset, const N: usize> Drop for Map<T, B, N> {
    fn drop(&mut self) {
        // Easiest way to call `assume_init` for all values and drop them
        while let Some(_) = self.pop() {}
    }
}

impl<T, B: Bitset, const N: usize> std::ops::Index<usize> for Map<T, B, N> {
    type Output = T;

    fn index(&self, i: usize) -> &T {
        assert!(self.occupied.get(i));
        unsafe {
            &*self.values[i].as_ptr()
        }
    }
}

impl<T> std::ops::IndexMut<usize> for Map16<T> {
    fn index_mut(&mut self, i: usize) -> &mut T {
        assert!(self.occupied.get(i));
        unsafe {
            &mut *self.values[i].as_mut_ptr()
        }
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for Map16<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map().entries(self.iter()).finish()
    }
}

impl<T> IntoIterator for Map16<T> {
    type Item = (usize, T);
    type IntoIter = IntoIterMap16<T>;

    fn into_iter(self) -> IntoIterMap16<T> {
        IntoIterMap16 {
            map: self
        }
    }
}

impl<'a, T, B: Bitset + Clone, const N: usize> IntoIterator for &'a Map<T, B, N> {
    type Item = (usize, &'a T);
    type IntoIter = IterMap<'a, T, B, N>;

    fn into_iter(self) -> IterMap<'a, T, B, N> {
        self.iter()
    }
}

impl<T> std::iter::FromIterator<T> for Map16<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut map = Map16::new();
        for val in iter {
            map.push(val);
        }
        map
    }
}

pub struct IterMap<'a, T, B: Bitset, const N: usize> {
    map: &'a Map<T, B, N>,
    elems: B,
}

impl<'a, T, B: Bitset, const N: usize> Iterator for IterMap<'a, T, B, N> {
    type Item = (usize, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        let i = self.elems.smallest()?;
        self.elems.clear(i);
        Some((i, &self.map[i]))
    }
}

/*
pub struct IterMutMap16<'a, T> {
    map: &'a mut Map16<T>,
    elems: Bitset16,
}

impl<'a, T> Iterator for IterMutMap16<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<&'a mut T> {
        let i = self.elems.smallest()?;
        self.elems.clear(i);
        Some(&mut self.map[i])
    }
}
*/

pub struct IntoIterMap16<T> {
    map: Map16<T>,
}

impl<T> Iterator for IntoIterMap16<T> {
    type Item = (usize, T);

    fn next(&mut self) -> Option<(usize, T)> {
        self.map.pop()
    }
}
