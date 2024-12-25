use super::weightedunionfind::Group;
use crate::util::Commutativity;

/// アーベル群のトレイト
pub trait AbelianGroup: Group + Commutativity {}
impl<T: Group + Commutativity> AbelianGroup for T {}

/// BinaeyIndexedTree. FenwickTreeとも
///
/// SegmentTreeがモノイドがアーベル群のときに限定して高速化されている
#[derive(Clone)]
pub struct BinaryIndexedTree<G: AbelianGroup>(Box<[G::T]>, G);

impl<G: AbelianGroup> BinaryIndexedTree<G> {
    /// 新しい長さ`n`のBinaryIndexedTreeを構築する.
    ///
    /// # Time complexity
    ///
    /// - *O*(*n*)
    #[must_use]
    pub fn new(group: G, n: usize) -> Self {
        Self((0..n).map(|_| group.e()).collect(), group)
    }

    /// BinaryIndexedTreeの長さを返す
    ///
    /// # Time complexity
    ///
    /// - *O*(1)
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// BinaryIndexedTreeが空か判定する
    ///
    /// # Time complexity
    ///
    /// - *O*(1)
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// `index`番目の要素に`val`を掛ける
    ///
    /// # Constraints
    ///
    /// - `index < self.len()`
    ///
    /// # Time complexity
    ///
    /// - *O*(log *n*)
    pub fn add(&mut self, mut index: usize, val: &G::T) {
        debug_assert!(index < self.len());
        while index < self.len() {
            self.0[index] = self.1.op(&self.0[index], val);
            index |= (index + 1) & !index;
        }
    }

    /// `range`の範囲の要素の漱石を計算する
    ///
    /// # Constraints
    ///
    /// - `range`は`0..self.len()`に含まれる区間である
    ///
    /// # Time complexity
    ///
    /// - *O*(log *n*)
    pub fn sum(&self, range: impl std::ops::RangeBounds<usize>) -> G::T {
        let mut left = match range.start_bound() {
            std::ops::Bound::Included(&i) => i,
            std::ops::Bound::Excluded(i) => i + 1,
            std::ops::Bound::Unbounded => 0,
        };
        let mut right = match range.end_bound() {
            std::ops::Bound::Included(i) => i + 1,
            std::ops::Bound::Excluded(&i) => i,
            std::ops::Bound::Unbounded => self.len(),
        };
        debug_assert!(left <= right && right <= self.len());

        let mut s = self.1.e();
        while left != right {
            if left < right {
                s = self.1.op(&s, &self.0[right - 1]);
                right &= !(right & !(right - 1));
            } else {
                s = self.1.opinv(&s, &self.0[left - 1]);
                left &= !(left & !(left - 1));
            }
        }
        s
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn add() {
        use super::super::util::{Associativity, Commutativity, Identity, Inverse, Magma};

        struct G;
        impl Magma for G {
            type T = i32;
            fn op(&self, a: &i32, b: &i32) -> i32 {
                a + b
            }
        }
        impl Inverse for G {
            fn inv(&self, a: &i32) -> i32 {
                -a
            }
        }
        impl Identity for G {
            fn e(&self) -> i32 {
                0
            }
        }
        impl Associativity for G {}
        impl Commutativity for G {}

        let mut v = BinaryIndexedTree::new(G, 10);
        assert_eq!(v.sum(1..5), 0);
        assert_eq!(v.sum(3..9), 0);
        assert_eq!(v.sum(..), 0);

        v.add(2, &12);
        assert_eq!(v.sum(1..5), 12);
        assert_eq!(v.sum(3..9), 0);
        assert_eq!(v.sum(..), 12);

        v.add(4, &3);
        assert_eq!(v.sum(1..5), 15);
        assert_eq!(v.sum(3..9), 3);
        assert_eq!(v.sum(..), 15);
    }
}
