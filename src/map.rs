use crate::bitset::Bitset16;
use std::mem::MaybeUninit;

pub struct Map16<T> {
    occupied: Bitset16,
    values: [MaybeUninit<T>; 16],
}

impl<T> Map16<T> {
    const UNINIT: MaybeUninit<T> = MaybeUninit::uninit();

    pub const fn new() -> Self {
        Self {
            occupied: Bitset16::new(),
            values: [Self::UNINIT; 16],
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

    pub fn iter(&self) -> IterMap16<T> {
        IterMap16 {
            map: &self,
            elems: self.occupied,
        }
    }
}

impl<T: Clone> Clone for Map16<T> {
    fn clone(&self) -> Map16<T> {
        let mut values = [Self::UNINIT; 16];
        for (i, v) in self.iter() {
            values[i] = MaybeUninit::new(v.clone());
        }
        Map16 {
            occupied: self.occupied,
            values,
        }
    }
}

impl<T> Drop for Map16<T> {
    fn drop(&mut self) {
        // Easiest way to call `assume_init` for all values and drop them
        while let Some(_) = self.pop() {}
    }
}

impl<T> std::ops::Index<usize> for Map16<T> {
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

impl<'a, T> IntoIterator for &'a Map16<T> {
    type Item = (usize, &'a T);
    type IntoIter = IterMap16<'a, T>;

    fn into_iter(self) -> IterMap16<'a, T> {
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

pub struct IterMap16<'a, T> {
    map: &'a Map16<T>,
    elems: Bitset16,
}

impl<'a, T> Iterator for IterMap16<'a, T> {
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
