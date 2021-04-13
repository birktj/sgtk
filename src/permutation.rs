pub type Perm16 = SmallPerm<16>;
pub type Perm32 = SmallPerm<32>;
pub type Perm64 = SmallPerm<64>;

pub trait Permutation: Clone + Eq + std::hash::Hash {
    fn new() -> Self;

    fn len(&self) -> usize;

    fn get(&self, i: usize) -> usize;

    fn swap(&mut self, i: usize, j: usize);

    fn from_iter<I: Iterator<Item = (usize, usize)>>(iter: I) -> Option<Self> where Self: Sized;

    fn iter<'a>(&'a self) -> PermIter<'a, Self> {
        PermIter {
            i: 0,
            perm: &self
        }
    }

    fn invert(&self) -> Self where Self: Sized {
        Self::from_iter(self.iter().map(|(i, j)| (j, i))).unwrap()
    }

    fn chain(&self, other: &Self) -> Self where Self: Sized {
        Self::from_iter(self.iter().map(|(i, j)| (i, other.get(j)))).unwrap()
    }

    fn generating_set(n: usize) -> Vec<Self> {
        let mut res = Vec::new();

        for i in 0..n-1 {
            let mut perm = Self::new();
            perm.swap(i, i+1);
            res.push(perm);
        }
        res
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct SmallPerm<const N: usize> {
    perm: [u8; N],
}

impl<const N: usize> Permutation for SmallPerm<N> {
    fn new() -> Self {
        let mut perm = [0; N];
        for i in 0..N {
            perm[i] = i as u8;
        }
        Self {
            perm
        }
    }

    fn len(&self) -> usize {
        N
    }

    fn get(&self, i: usize) -> usize {
        usize::from(self.perm[i])
    }

    fn swap(&mut self, i: usize, j: usize) {
        self.perm.swap(i, j);
    }

    fn from_iter<I: Iterator<Item = (usize, usize)>>(iter: I) -> Option<Self> {
        let mut perm = [0; N];
        let mut used = [false; N];
        let mut n = 0;
        for (i, j) in iter {
            perm[i] = j as u8;
            used[j] = true;
            n += 1;
        }

        for i in n..N {
            perm[i] = i as u8;
            used[i] = true;
        }
        for i in 0..N {
            if !used[i] {
                return None
            }
        }

        Some(Self {
            perm
        })
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
