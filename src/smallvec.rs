use std::mem::MaybeUninit;

pub struct Smallvec<T, const N: usize> {
    len: usize,
    values: [MaybeUninit<T>; N],
}

impl<T, const N: usize> Smallvec<T, N> {
    pub fn new() -> Self {
        let values = unsafe { MaybeUninit::uninit().assume_init() };
        Self {
            len: 0,
            values,
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            None
        } else {
            self.len -= 1;
            let res = std::mem::replace(&mut self.values[self.len], MaybeUninit::uninit());
            Some(unsafe { res.assume_init() })
        }
    }

    pub fn push(&mut self, val: T) {
        self.values[self.len] = MaybeUninit::new(val);
        self.len += 1;
    }

    pub const fn len(&self) -> usize {
        self.len
    }

    pub fn slice(&self) -> &[T] {
        unsafe { &*(&self.values[0..self.len] as *const [MaybeUninit<T>] as *const [T]) }
    }

    pub fn slice_mut(&mut self) -> &mut [T] {
        unsafe { &mut *(&mut self.values[0..self.len] as *mut [MaybeUninit<T>] as *mut [T]) }
    }

    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn iter(&self) -> std::slice::Iter<T> {
        self.slice().iter()
    }
}

impl<T, const N: usize> Drop for Smallvec<T, N> {
    fn drop(&mut self) {
        while let Some(_) = self.pop() {}
    }
}

impl<T, const N: usize> std::ops::Index<usize> for Smallvec<T, N> {
    type Output = T;

    fn index(&self, i: usize) -> &T {
        assert!(i < self.len);
        unsafe { &*self.values[i].as_ptr() }
    }
}

impl<T, const N: usize> std::ops::IndexMut<usize> for Smallvec<T, N> {
    fn index_mut(&mut self, i: usize) -> &mut T {
        assert!(i < self.len);
        unsafe { &mut *self.values[i].as_mut_ptr() }
    }
}

impl<T: std::fmt::Debug, const N: usize> std::fmt::Debug for Smallvec<T, N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}
