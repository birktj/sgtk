pub struct Smallvec<T, const N: usize> {
    len: usize,
    values: [T; N],
}

impl<T: Default + Copy, const N: usize> Smallvec<T, N> {
    pub fn new() -> Self {
        Self {
            len: 0,
            values: [T::default(); N],
        }
    }
}

impl<T: Copy, const N: usize> Smallvec<T, N> {
    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            None
        } else {
            self.len -= 1;
            Some(self.values[self.len])
        }
    }
}

impl<T, const N: usize> Smallvec<T, N> {
    pub fn push(&mut self, val: T) {
        self.values[self.len] = val;
        self.len += 1;
    }


    pub const fn len(&self) -> usize {
        self.len
    }

    pub fn slice(&self) -> &[T] {
        &self.values[0..self.len]
    }

    pub fn slice_mut(&mut self) -> &mut [T] {
        &mut self.values[0..self.len]
    }

    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn iter(&self) -> std::slice::Iter<T> {
        (&self.values[..self.len]).iter()
    }
}

impl<T, const N: usize> std::ops::Index<usize> for Smallvec<T, N> {
    type Output = T;

    fn index(&self, i: usize) -> &T {
        debug_assert!(i < self.len);
        &self.values[i]
    }
}

impl<T, const N: usize> std::ops::IndexMut<usize> for Smallvec<T, N> {
    fn index_mut(&mut self, i: usize) -> &mut T {
        debug_assert!(i < self.len);
        &mut self.values[i]
    }
}

impl<T: std::fmt::Debug, const N: usize> std::fmt::Debug for Smallvec<T, N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}
