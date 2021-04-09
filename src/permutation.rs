pub trait Permutation {
    fn len(&self) -> usize;
    fn get(&self, i: usize) -> usize;
    fn iter<'a>(&'a self) -> PermIter<'a, Self> {
        PermIter {
            i: 0,
            perm: &self
        }
    }
}

pub struct SmallPerm<const N: usize> {
    perm: [u8; N],
}

impl<const N: usize> Permutation for SmallPerm<N> {
    fn len(&self) -> usize {
        N
    }

    fn get(&self, i: usize) -> usize {
        usize::from(self.perm[i])
    }
}

pub struct PermIter<'a, P: ?Sized> {
    i: usize,
    perm: &'a P,
}

impl<'a, P: Permutation> Iterator for PermIter<'a, P> {
    type Item = (usize, usize);

    fn next(&mut self) -> Option<(usize, usize)> {
        if self.i >= self.perm.len() {
            None
        } else {
            let i = self.i;
            self.i += 1;
            Some((i, self.perm.get(i)))
        }
    }
}
