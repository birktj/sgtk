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

    pub fn iter(&self) -> std::slice::Iter<u8> {
        (&self.values[..self.len]).iter()
    }

    pub fn contains(&self, val: usize) -> bool {
        self.values.contains(&(val as u8))
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
