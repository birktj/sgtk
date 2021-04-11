use crate::seq::*;
use crate::permutation::{Permutation, SmallPerm};

pub trait Intset {
    type Iter: Iterator<Item = usize> + DoubleEndedIterator + Clone;

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

    fn iter(&self) -> Self::Iter;
}

pub trait Bitset: Intset + Eq + Clone {
    const SIZE: usize;

    type Enumerate: Iterator<Item = Self>;
    type Perm: Permutation;

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
            type Iter = $iter;

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

            fn iter(&self) -> Self::Iter {
                self.into_iter()
            }
        }

        impl Bitset for $name {
            const SIZE: usize = $size;

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

/*
impl Bitset16 {
    pub fn shuffle(&mut self, permutation: &Seq16) {
        let old = *self;
        self.bitset = 0;
        
        for (i, j) in permutation.iter().enumerate() {
            if old.get(i) {
                self.set(j);
            }
        }
    }
}
*/

/*
#[derive(Copy, Clone, Default, Eq, PartialEq, Hash)]
pub struct Bitset16 {
    bitset: u16
}

impl Bitset16 {
    pub const fn new() -> Bitset16 {
        Bitset16 {
            bitset: 0
        }
    }

    pub const fn from_u16(bitset: u16) -> Bitset16 {
        Bitset16 {
            bitset
        }
    }

    pub const fn to_u16(self) -> u16 {
        self.bitset
    }

    pub fn mask_le(n: usize) -> Bitset16 {
        Bitset16 {
            bitset: (1u16 << n).wrapping_sub(1),
        }
    }

    pub fn mask_ge(n: usize) -> Bitset16 {
        Bitset16::mask_le(n).invert()
    }

    pub fn is_empty(&self) -> bool {
        self.bitset == 0
    }

    pub fn get(&self, i: usize) -> bool {
        debug_assert!(i < 16);
        (self.bitset & (1 << i)) > 0
    }

    pub fn set(&mut self, i: usize) {
        debug_assert!(i < 16);
        self.bitset |= 1 << i;
    }

    pub fn clear(&mut self, i: usize) {
        debug_assert!(i < 16);
        self.bitset &= !(1 << i);
    }

    pub fn swap(&mut self, i: usize, j: usize) {
        let vi = (self.bitset >> i) & 1;
        let vj = (self.bitset >> j) & 1;
        self.bitset = (self.bitset & !((1 << i) | (1 << j))) | (vi << i) | (vj << j);
    }

    pub fn smallest(&self) -> Option<usize> {
        // TODO benchmark diff
        /*
        if self.bitset != 0 {
            Some(self.bitset.trailing_zeros() as usize)
        } else {
            None
        }
        */
        (*self).into_iter().next()
    }

    pub fn count(&self) -> usize {
        self.bitset.count_ones() as usize
    }

    pub fn union(&self, other: &Bitset16) -> Bitset16 {
        Bitset16 {
            bitset: self.bitset | other.bitset
        }
    }

    pub fn intersection(&self, other: &Bitset16) -> Bitset16 {
        Bitset16 {
            bitset: self.bitset & other.bitset
        }
    }

    pub fn difference(&self, other: &Bitset16) -> Bitset16 {
        Bitset16 {
            bitset: self.bitset & !other.bitset
        }
    }

    pub fn invert(&self) -> Bitset16 {
        Bitset16 {
            bitset: !self.bitset
        }
    }

    pub fn is_superset(&self, other: &Bitset16) -> bool {
        (self.bitset & other.bitset) == other.bitset
    }

    pub fn enumerate(maxn: usize) -> IterEnumerate16 {
        IterEnumerate16 {
            maxval: ((1u32 << maxn) - 1) as u16,
            curr: 0,
            finished: false,
            last: None,
        }
    }

    pub fn to_seq(&self) -> Seq16 {
        let mut seq = Seq16::new();

        for x in self.into_iter() {
            seq.push(x);
        }

        seq
    }

    pub fn shuffle(&mut self, permutation: &Seq16) {
        let old = *self;
        self.bitset = 0;
        
        for (i, j) in permutation.iter().enumerate() {
            if old.get(i) {
                self.set(*j as usize);
            }
        }
    }
}

impl IntoIterator for Bitset16 {
    type Item = usize;
    type IntoIter = IterBitset16;

    fn into_iter(self) -> IterBitset16 {
        IterBitset16 {
            bitset: self.bitset,
        }
    }
}

#[derive(Copy, Clone)]
pub struct IterBitset16 {
    bitset: u16,
}

impl Iterator for IterBitset16 {
    type Item = usize;

    fn next(&mut self) -> Option<usize> {
        if self.bitset != 0 {
            let i = self.bitset.trailing_zeros() as usize;
            self.bitset = self.bitset ^ (1 << i);
            Some(i)
        } else {
            None 
        }
        /*
        while self.i < 16 {
            if self.bitset & (1 << self.i) > 0 {
                self.i += 1;
                return Some(self.i - 1)
            }
            self.i += 1
        }
        None
        */
    }
}

impl std::iter::DoubleEndedIterator for IterBitset16 {
    fn next_back(&mut self) -> Option<usize> {
        if self.bitset != 0 {
            let i = 15 - self.bitset.leading_zeros() as usize;
            self.bitset = self.bitset ^ (1 << i);
            Some(i)
        } else {
            None 
        }
    }
}

impl std::fmt::Debug for Bitset16 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_set().entries(self.into_iter()).finish()
    }
}

pub struct IterEnumerate16 {
    maxval: u16,
    curr: u16,
    finished: bool,
    last: Option<Bitset16>,
}

impl Iterator for IterEnumerate16 {
    type Item = Bitset16;

    fn next(&mut self) -> Option<Bitset16> {
        if self.finished {
            return self.last.take()
        }
        let res = Bitset16::from_u16(self.curr);
        self.curr += 1;

        if self.curr >= self.maxval {
            self.finished = true;
            self.last = Some(Bitset16::from_u16(self.curr));
        }

        Some(res)
    }
}
*/
