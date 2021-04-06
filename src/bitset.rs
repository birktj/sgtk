use crate::seq::*;

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
