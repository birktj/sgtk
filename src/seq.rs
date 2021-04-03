#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Seq16 {
    len: usize,
    values: [u8; 16]
}

impl Seq16 {
    pub const fn new() -> Seq16 {
        Seq16 {
            len: 0,
            values: [0; 16]
        }
    }

    pub const fn from_slice(slice: &[u8]) -> Seq16 {
        let mut values = [0; 16];
        // We use a while loop to keep the function const
        let mut i = 0;
        while i < slice.len() {
            values[i] = slice[i];
            i += 1;
        }
        Seq16 {
            len: slice.len(),
            values
        }
    }

    pub const fn len(&self) -> usize {
        self.len
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

    pub fn push(&mut self, val: usize) {
        self.values[self.len] = val as u8;
        self.len += 1;
    }

    pub fn pop(&mut self) -> Option<usize> {
        if self.len == 0 {
            None
        } else {
            self.len -= 1;
            Some(self.values[self.len] as usize)
        }
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

    pub fn iter(&self) -> std::slice::Iter<u8> {
        (&self.values[..self.len]).iter()
    }

    pub fn contains(&self, val: usize) -> bool {
        self.values.contains(&(val as u8))
    }

    pub fn permutations(&self) -> SeqPermutations16 {
        let mut seq = *self;

        (&mut seq.values[..seq.len]).sort();

        SeqPermutations16 {
            seq: Some(seq),
        }
    }
}

#[derive(Copy, Clone)]
pub struct SeqPermutations16 {
    seq: Option<Seq16>,
}

impl SeqPermutations16 {
    pub fn empty() -> Self {
        Self {
            seq: None
        }
    }
}

impl Iterator for SeqPermutations16 {
    type Item = Seq16;

    fn next(&mut self) -> Option<Seq16> {
        let res = self.seq;

        if let Some(mut seq) = self.seq {
            let l = seq.iter().rev()
                .scan(0, |st, x| {
                    let res = x >= st;
                    *st = *x;
                    Some(res)
                })
                .take_while(|x| *x)
                .count();

            let i = seq.len - l;

            if i > 0 {
                let x = seq[i-1];
                let j = seq.iter().enumerate()
                    .rev()
                    .skip_while(|(_, y)| **y < x)
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

impl std::ops::Index<usize> for Seq16 {
    type Output = u8;

    fn index(&self, i: usize) -> &u8 {
        debug_assert!(i < self.len);
        &self.values[i]
    }
}

impl std::ops::IndexMut<usize>for Seq16 {
    fn index_mut(&mut self, i: usize) -> &mut u8 {
        debug_assert!(i < self.len);
        &mut self.values[i]
    }
}

impl std::fmt::Debug for Seq16 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}
