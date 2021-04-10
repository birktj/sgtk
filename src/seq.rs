pub type Seq16 = SmallSeq<16>;

pub trait Seq: Clone {
    type IterPerm: Iterator<Item = Self>;

    fn new() -> Self;
    fn len(&self) -> usize;
    fn get(&self, i: usize) -> usize;
    fn push(&mut self, v: usize);
    fn pop(&mut self) -> Option<usize>;
    fn reverse(&mut self) where Self: Sized {
        let mut old = std::mem::replace(self, Self::new());
        while let Some(i) = old.pop() {
            self.push(i);
        }
    }
    fn iter(&self) -> SeqIter<Self> {
        SeqIter {
            i: 0,
            len: self.len(),
            seq: &self,
        }
    }
    fn permutations(self) -> Self::IterPerm;
}

#[derive(Clone)]
pub struct SeqIter<'a, S: ?Sized> {
    i: usize,
    len: usize,
    seq: &'a S,
}

impl<'a, S: Seq> Iterator for SeqIter<'a, S> {
    type Item = usize;

    fn next(&mut self) -> Option<usize> {
        if self.i < self.len {
            let i = self.i;
            self.i += 1;
            Some(self.seq.get(i))
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len - self.i, Some(self.len - self.i))
    }
}

impl<'a, S: Seq> DoubleEndedIterator for SeqIter<'a, S> {
    fn next_back(&mut self) -> Option<usize> {
        if self.i < self.len {
            self.len -= 1;
            Some(self.seq.get(self.len))
        } else {
            None
        }
    }
}

impl<'a, S: Seq> ExactSizeIterator for SeqIter<'a, S> {}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct SmallSeq<const N: usize> {
    len: usize,
    values: [u8; N]
}

impl<const N: usize> Seq for SmallSeq<N> {
    type IterPerm = SeqPermutations<N>;

    fn new() -> Self {
        Self {
            len: 0,
            values: [0; N]
        }
    }

    fn len(&self) -> usize {
        self.len
    }

    fn get(&self, i: usize) -> usize {
        assert!(i < self.len);
        usize::from(self.values[i])
    }

    fn push(&mut self, val: usize) {
        self.values[self.len] = val as u8;
        self.len += 1;
    }

    fn pop(&mut self) -> Option<usize> {
        if self.len == 0 {
            None
        } else {
            self.len -= 1;
            Some(self.values[self.len] as usize)
        }
    }

    fn permutations(mut self) -> SeqPermutations<N> {
        (&mut self.values[..self.len]).sort();

        SeqPermutations {
            seq: Some(self),
        }
    }
}

impl<const N: usize> SmallSeq<N> {
    pub const fn from_slice(slice: &[u8]) -> Self {
        let mut values = [0; N];
        // We use a while loop to keep the function const
        let mut i = 0;
        while i < slice.len() {
            values[i] = slice[i];
            i += 1;
        }
        Self {
            len: slice.len(),
            values
        }
    }

    pub fn slice(&self) -> &[u8] {
        &self.values[0..self.len]
    }

    pub fn slice_mut(&mut self) -> &mut [u8] {
        &mut self.values[0..self.len]
    }

    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn first(&self) -> Option<usize> {
        if self.len == 0 {
            None
        } else {
            Some(usize::from(self.values[0]))
        }
    }

    pub fn last(&self) -> Option<usize> {
        if self.len == 0 {
            None
        } else {
            Some(usize::from(self.values[self.len - 1]))
        }
    }

    pub fn insert(&mut self, i: usize, v: usize) {
        self.values.copy_within(i..self.len, i+1);
        self.values[i] = v as u8;
        self.len += 1;
    }

    pub fn extend(&mut self, slice: &[u8]) {
        for x in slice {
            self.push(usize::from(*x));
        }
    }

    pub fn swap(&mut self, i: usize, j: usize) {
        self.values.swap(i, j);
    }

    pub fn reverse(&mut self) {
        (&mut self.values[0..self.len]).reverse()
    }

    pub fn contains(&self, val: usize) -> bool {
        self.values.contains(&(val as u8))
    }

}

#[derive(Copy, Clone)]
pub struct SeqPermutations<const N: usize> {
    seq: Option<SmallSeq<N>>,
}

impl<const N: usize> SeqPermutations<N> {
    pub fn empty() -> Self {
        Self {
            seq: None
        }
    }
}

impl<const N: usize> Iterator for SeqPermutations<N> {
    type Item = SmallSeq<N>;

    fn next(&mut self) -> Option<SmallSeq<N>> {
        let res = self.seq;

        if let Some(mut seq) = self.seq {
            let l = seq.iter().rev()
                .scan(0, |st, x| {
                    let res = x >= *st;
                    *st = x;
                    Some(res)
                })
                .take_while(|x| *x)
                .count();

            let i = seq.len - l;

            if i > 0 {
                let x = seq.get(i-1);
                let j = seq.iter().enumerate()
                    .rev()
                    .skip_while(|(_, y)| *y < x)
                    .next().unwrap().0;

                seq.values.swap(i-1, j);
                (&mut seq.values[i..seq.len]).reverse();
                self.seq = Some(seq);
            } else {
                self.seq = None;
            }
        }

        res
    }
}

impl<const N: usize> std::ops::Index<usize> for SmallSeq<N> {
    type Output = u8;

    fn index(&self, i: usize) -> &u8 {
        debug_assert!(i < self.len);
        &self.values[i]
    }
}

impl<const N: usize> std::ops::IndexMut<usize>for SmallSeq<N> {
    fn index_mut(&mut self, i: usize) -> &mut u8 {
        debug_assert!(i < self.len);
        &mut self.values[i]
    }
}

impl<const N: usize> std::fmt::Debug for SmallSeq<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}
