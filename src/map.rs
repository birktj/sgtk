use std::collections::HashMap;
use crate::bitset::{Bitset, Bitset16, Bitset32, Bitset64, Bitset128};
use std::mem::MaybeUninit;
use std::ops::Index;
use std::ops::IndexMut;

pub type Map16<T> = Map<T, Bitset16, 16>;
pub type Map32<T> = Map<T, Bitset32, 32>;
pub type Map64<T> = Map<T, Bitset64, 64>;
pub type Map128<T> = Map<T, Bitset128, 128>;

#[derive(Debug, Copy, Clone)]
pub struct FullMapError;

pub trait Slotmap: Index<usize> + IndexMut<usize> where Self::Output: Sized, for<'a> &'a Self: IntoIterator<Item = (usize, &'a Self::Output)> {
    fn new() -> Self;

    fn push(&mut self, val: Self::Output) -> Result<usize, FullMapError>;

    fn pop(&mut self) -> Option<(usize, Self::Output)>;

    fn take(&mut self, i: usize) -> Option<Self::Output>;

    fn insert(&mut self, i: usize, val: Self::Output);

    fn is_empty(&self) -> bool;

    fn count(&self) -> usize;
}

pub struct Map<T, B: Bitset, const N: usize> {
    occupied: B,
    values: [MaybeUninit<T>; N],
}

impl<T, B: Bitset, const N: usize> Slotmap for Map<T, B, N> {
    #[inline]
    fn new() -> Self {
        let values = unsafe { MaybeUninit::uninit().assume_init() };
        Self {
            occupied: B::new(),
            values,
        }
    }

    #[inline]
    fn push(&mut self, val: T) -> Result<usize, FullMapError> {
        let i = self.occupied.invert().smallest().ok_or(FullMapError)?;
        self.occupied.set(i);
        self.values[i] = MaybeUninit::new(val);
        Ok(i)
    }

    #[inline]
    fn pop(&mut self) -> Option<(usize, T)> {
        let i = self.occupied.smallest()?;
        Some((i, self.take(i)?))
    }

    #[inline]
    fn take(&mut self, i: usize) -> Option<T> {
        if self.occupied.get(i) {
            self.occupied.clear(i);
            let res = std::mem::replace(&mut self.values[i], MaybeUninit::uninit());

            unsafe {
                Some(res.assume_init())
            }
        } else {
            None
        }
    }

    #[inline]
    fn insert(&mut self, i: usize, val: T) {
        let _ = self.take(i);
        self.occupied.set(i);
        self.values[i] = MaybeUninit::new(val);
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.occupied.is_empty()
    }

    #[inline]
    fn count(&self) -> usize {
        self.occupied.count()
    }
}

impl<T, B: Bitset, const N: usize> Map<T, B, N> {
    pub fn iter(&self) -> IterMap<T, B, N> {
        IterMap {
            map: &self,
            elems: self.occupied.clone(),
        }
    }
}

impl<T: Clone, B: Bitset, const N: usize> Clone for Map<T, B, N> {
    fn clone(&self) -> Self {
        let mut clone = Self::new();
        clone.occupied = self.occupied.clone();
        for (i, v) in self.iter() {
            clone.values[i] = MaybeUninit::new(v.clone());
        }
        clone
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

impl<T, B: Bitset, const N: usize> std::ops::IndexMut<usize> for Map<T, B, N> {
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

impl<T, B: Bitset, const N: usize> IntoIterator for Map<T, B, N> {
    type Item = (usize, T);
    type IntoIter = IntoIterMap<T, B, N>;

    fn into_iter(self) -> IntoIterMap<T, B, N> {
        IntoIterMap {
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

impl<T, B: Bitset, const N: usize> std::iter::FromIterator<T> for Map<T, B, N> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut map = Map::new();
        for val in iter {
            map.push(val).unwrap();
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

pub struct IntoIterMap<T, B: Bitset, const N: usize> {
    map: Map<T, B, N>,
}

impl<T, B: Bitset, const N: usize> Iterator for IntoIterMap<T, B, N> {
    type Item = (usize, T);

    fn next(&mut self) -> Option<(usize, T)> {
        self.map.pop()
    }
}

#[derive(Clone)]
pub struct DynMap<T> {
    counter: usize,
    values: HashMap<usize, T>,
}

impl<T> Slotmap for DynMap<T> {
    fn new() -> Self {
        Self {
            counter: 0,
            values: HashMap::new(),
        }
    }

    fn push(&mut self, val: T) -> Result<usize, FullMapError> {
        let i = self.counter;
        self.counter += 1;
        self.values.insert(i, val);
        Ok(i)
    }

    fn pop(&mut self) -> Option<(usize, T)> {
        let i = *self.values.keys().next()?;
        Some((i, self.take(i)?))
    }

    fn take(&mut self, i: usize) -> Option<T> {
        self.values.remove(&i)
    }

    fn insert(&mut self, i: usize, val: T) {
        if self.counter <= i {
            self.counter = i + 1;
        }
        self.values.insert(i, val);
    }

    fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    fn count(&self) -> usize {
        self.values.len()
    }

}

impl<T> DynMap<T> {
    pub fn iter(&self) -> IterDynMap<T> {
        IterDynMap {
            iter: self.values.iter(),
        }
    }
}

impl<T> std::ops::Index<usize> for DynMap<T> {
    type Output = T;

    fn index(&self, i: usize) -> &T {
        &self.values[&i]
    }
}

impl<T> std::ops::IndexMut<usize> for DynMap<T> {
    fn index_mut(&mut self, i: usize) -> &mut T {
        self.values.get_mut(&i).unwrap()
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for DynMap<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map().entries(self.iter()).finish()
    }
}

impl<T> IntoIterator for DynMap<T> {
    type Item = (usize, T);
    type IntoIter = std::collections::hash_map::IntoIter<usize, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.values.into_iter()
    }
}

impl<'a, T> IntoIterator for &'a DynMap<T> {
    type Item = (usize, &'a T);
    type IntoIter = IterDynMap<'a, T>;

    fn into_iter(self) -> IterDynMap<'a, T> {
        self.iter()
    }
}

impl<T> std::iter::FromIterator<T> for DynMap<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut map = DynMap::new();
        for val in iter {
            map.push(val).unwrap();
        }
        map
    }
}

pub struct IterDynMap<'a, T> {
    iter: std::collections::hash_map::Iter<'a, usize, T>,
}

impl<'a, T> Iterator for IterDynMap<'a, T> {
    type Item = (usize, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        let (i, val) = self.iter.next()?;
        Some((*i, val))
    }
}
