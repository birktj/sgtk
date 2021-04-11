use std::collections::HashSet;
use crate::seq::*;
use crate::permutation::{Permutation, SmallPerm};

pub trait Intset {
    fn new() -> Self;

    fn is_empty(&self) -> bool;

    fn get(&self, i: usize) -> bool;

    fn set_val(&mut self, i: usize, v: bool);

    fn set(&mut self, i: usize) {
        self.set_val(i, true);
    }

    fn clear(&mut self, i: usize) {
        self.set_val(i, false);
    }

    fn smallest(&self) -> Option<usize>;

    fn count(&self) -> usize;
}

pub trait Bitset: Intset + Eq + Clone {
    const SIZE: usize;

    type Enumerate: Iterator<Item = Self>;
    type Perm: Permutation;
    type Iter: Iterator<Item = usize> + DoubleEndedIterator + Clone;

    fn swap(&mut self, i: usize, j: usize);

    fn mask_le(n: usize) -> Self;

    fn mask_ge(n: usize) -> Self;

    fn union(&self, other: &Self) -> Self;

    fn intersection(&self, other: &Self) -> Self;

    fn difference(&self, other: &Self) -> Self;

    fn invert(&self) -> Self;

    fn is_superset(&self, other: &Self) -> bool;

    fn enumerate(maxn: usize) -> Self::Enumerate;

    fn shuffle(&mut self, permutation: &Self::Perm);

    fn iter(&self) -> Self::Iter;
}

macro_rules! bit_set {
    ($name:ident, $size:expr, $type:ty, $from_ty:ident, $to_ty:ident, $iter:ident, $iter_enum:ident) => (
        #[derive(Copy, Clone, Default, Eq, PartialEq, Hash, Ord, PartialOrd)]
        pub struct $name {
            bitset: $type,
        }

        impl $name {
            pub fn $from_ty(bitset: $type) -> Self {
                Self {
                    bitset
                }
            }

            pub fn $to_ty(&self) -> $type {
                self.bitset
            }
        }

        impl Intset for $name {
            fn new() -> Self {
                Self {
                    bitset: 0
                }
            }

            fn is_empty(&self) -> bool {
                self.bitset == 0
            }

            fn get(&self, i: usize) -> bool {
                (self.bitset & (1 << i)) > 0
            }

            fn set(&mut self, i: usize) {
                self.bitset |= 1 << i;
            }

            fn set_val(&mut self, i: usize, v: bool) {
                self.bitset |= if v { 1 << i } else { 0 };
            }

            fn clear(&mut self, i: usize) {
                self.bitset &= !(1 << i);
            }

            fn smallest(&self) -> Option<usize> {
                (*self).into_iter().next()
            }

            fn count(&self) -> usize {
                self.bitset.count_ones() as usize
            }
        }

        impl Bitset for $name {
            const SIZE: usize = $size;

            type Iter = $iter;

            type Enumerate = $iter_enum;

            type Perm = SmallPerm<$size>;

            fn swap(&mut self, i: usize, j: usize) {
                let vi = (self.bitset >> i) & 1;
                let vj = (self.bitset >> j) & 1;
                self.bitset = (self.bitset & !((1 << i) | (1 << j))) | (vi << i) | (vj << j);
            }

            fn mask_le(n: usize) -> Self {
                Self {
                    bitset: (1 as $type << n).wrapping_sub(1),
                }
            }

            fn mask_ge(n: usize) -> Self {
                Self::mask_le(n).invert()
            }

            fn union(&self, other: &Self) -> Self {
                Self {
                    bitset: self.bitset | other.bitset
                }
            }

            fn intersection(&self, other: &Self) -> Self {
                Self {
                    bitset: self.bitset & other.bitset
                }
            }

            fn difference(&self, other: &Self) -> Self {
                Self {
                    bitset: self.bitset & !other.bitset
                }
            }

            fn invert(&self) -> Self {
                Self {
                    bitset: !self.bitset
                }
            }

            fn is_superset(&self, other: &Self) -> bool {
                (self.bitset & other.bitset) == other.bitset
            }

            fn enumerate(maxn: usize) -> $iter_enum {
                let maxval = if maxn == $size {
                    <$type>::MAX
                } else {
                    1 << maxn
                };

                $iter_enum {
                    maxval,
                    curr: 0,
                    finished: false,
                    last: None,
                }
            }

            fn shuffle(&mut self, permutation: &Self::Perm) {
                let old = *self;
                self.bitset = 0;
                
                for (i, j) in permutation.iter() {
                    if old.get(i) {
                        self.set(j);
                    }
                }
            }

            fn iter(&self) -> Self::Iter {
                self.into_iter()
            }
        }

        impl IntoIterator for $name {
            type Item = usize;
            type IntoIter = $iter;

            fn into_iter(self) -> $iter {
                $iter {
                    bitset: self.bitset,
                }
            }
        }

        impl<'a> IntoIterator for &'a $name {
            type Item = usize;
            type IntoIter = $iter;

            fn into_iter(self) -> $iter {
                $iter {
                    bitset: self.bitset,
                }
            }
        }

        #[derive(Copy, Clone)]
        pub struct $iter {
            bitset: $type,
        }

        impl Iterator for $iter {
            type Item = usize;

            fn next(&mut self) -> Option<usize> {
                if self.bitset != 0 {
                    let i = self.bitset.trailing_zeros() as usize;
                    self.bitset = self.bitset ^ (1 << i);
                    Some(i)
                } else {
                    None 
                }
            }
        }

        impl std::iter::DoubleEndedIterator for $iter {
            fn next_back(&mut self) -> Option<usize> {
                if self.bitset != 0 {
                    let i = $size - 1 - self.bitset.leading_zeros() as usize;
                    self.bitset = self.bitset ^ (1 << i);
                    Some(i)
                } else {
                    None 
                }
            }
        }

        impl std::fmt::Debug for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_set().entries(self.into_iter()).finish()
            }
        }

        pub struct $iter_enum {
            maxval: $type,
            curr: $type,
            finished: bool,
            last: Option<$name>,
        }

        impl Iterator for $iter_enum {
            type Item = $name;

            fn next(&mut self) -> Option<$name> {
                if self.finished {
                    return self.last.take()
                }
                let res = $name::$from_ty(self.curr);
                self.curr += 1;

                if self.curr >= self.maxval {
                    self.finished = true;
                    self.last = Some($name::$from_ty(self.curr));
                }

                Some(res)
            }
        }
    )
}

bit_set!(Bitset16, 16, u16, from_u16, to_u16, Iter16, IterEnumerate16);
bit_set!(Bitset32, 32, u32, from_u32, to_u32, Iter32, IterEnumerate32);
bit_set!(Bitset64, 64, u64, from_u64, to_u64, Iter64, IterEnumerate64);
bit_set!(Bitset128, 128, u128, from_u128, to_u128, Iter128, IterEnumerate128);

pub struct DynIntSet {
    inner: HashSet<usize>,
}

impl Intset for DynIntSet {
    fn new() -> Self {
        Self {
            inner: HashSet::new(),
        }
    }

    fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    fn get(&self, i: usize) -> bool {
        self.inner.contains(&i)
    }

    fn set_val(&mut self, i: usize, v: bool) {
        if v {
            self.inner.insert(i);
        } else {
            self.inner.remove(&i);
        }
    }

    fn smallest(&self) -> Option<usize> {
        // FIXME: this is not correct!!!
        self.inner.iter().next().copied()
    }

    fn count(&self) -> usize {
        self.inner.len()
    }
}

impl<'a> IntoIterator for &'a DynIntSet {
    type Item = usize;
    type IntoIter = std::iter::Copied<std::collections::hash_set::Iter<'a, usize>>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.iter().copied()
    }
}
