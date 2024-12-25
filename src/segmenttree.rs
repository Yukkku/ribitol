use super::util::{Associativity, Identity};

pub trait Monoid: Associativity + Identity {}
impl<T: Associativity + Identity> Monoid for T {}

#[derive(Clone)]
pub struct SegmentTree<M: Monoid>(Box<[M::T]>, usize, M);

impl<M: Monoid> SegmentTree<M> {
    /// 全ての要素が`monoid.e()`で初期化された長さ`n`のSegmentTreeを構築する.
    ///
    /// # Time complexity
    ///
    /// - *O*(*n*)
    #[must_use]
    pub fn new(monoid: M, n: usize) -> Self {
        Self(
            (0..n * 2 - n.count_ones() as usize)
                .map(|_| monoid.e())
                .collect(),
            n,
            monoid,
        )
    }

    /// 配列`vec`からSegmentTreeを構築する.
    ///
    /// # Time complexity
    ///
    /// - *O*(*n*)
    #[must_use]
    pub fn from_vec(monoid: M, mut vec: Vec<M::T>) -> Self {
        let n = vec.len();
        vec.reserve_exact(n - n.count_ones() as usize);
        {
            let mut len = n;
            let mut offset = 0;
            while len > 1 {
                for i in (1..len).step_by(2) {
                    vec.push(monoid.op(&vec[i + offset], &vec[i + offset - 1]));
                }
                offset += len;
                len >>= 1;
            }
        }
        Self(vec.into_boxed_slice(), n, monoid)
    }

    /// SegmentTreeの長さを返す.
    ///
    /// # Time complexity
    ///
    /// - *O*(*1*)
    #[must_use]
    pub fn len(&self) -> usize {
        self.1
    }

    /// SegmentTreeが空かどうか調べる
    ///
    /// # Time complexity
    ///
    /// - *O*(*1*)
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.1 == 0
    }

    /// SegmentTreeの`index`番目の値を`value`に設定する.
    ///
    /// # Constraints
    ///
    /// - `index < self.len()`
    ///
    /// # Time complexity
    ///
    /// - *O*(log *n*)
    pub fn set(&mut self, index: usize, value: M::T) {
        debug_assert!(index < self.len());
        self.0[index] = value;
        self.update(index);
    }

    /// SegmentTreeの`index`番目の値を取得する.
    ///
    /// # Constraints
    ///
    /// - `index < self.len()`
    ///
    /// # Time complexity
    ///
    /// - *O*(1)
    #[must_use]
    pub fn get(&self, index: usize) -> &M::T {
        debug_assert!(index < self.len());
        &self.0[index]
    }

    /// SegmentTreeの`index`番目の値の可変参照(のラッパー)を取得する.
    ///
    /// # Constraints
    ///
    /// - `index < self.len()`
    ///
    /// # Time complexity
    ///
    /// - *O*(1)
    /// - *O*(log *n*)  (ラッパーのデストラクタ)
    #[must_use]
    pub fn setter(&mut self, index: usize) -> impl std::ops::DerefMut<Target = M::T> + use<'_, M> {
        debug_assert!(index < self.len());

        struct Wrapper<'a, M: Monoid>(&'a mut SegmentTree<M>, usize);
        impl<M: Monoid> std::ops::Deref for Wrapper<'_, M> {
            type Target = M::T;
            fn deref(&self) -> &M::T {
                &self.0 .0[self.1]
            }
        }
        impl<M: Monoid> std::ops::DerefMut for Wrapper<'_, M> {
            fn deref_mut(&mut self) -> &mut M::T {
                &mut self.0 .0[self.1]
            }
        }
        impl<M: Monoid> Drop for Wrapper<'_, M> {
            fn drop(&mut self) {
                self.0.update(self.1);
            }
        }

        Wrapper(self, index)
    }

    fn update(&mut self, mut index: usize) {
        let mut len = self.len();
        let mut slice = self.0.as_mut();
        while index | 1 < len {
            let val = self.2.op(&slice[index & !1], &slice[index | 1]);
            slice = &mut slice[len..];
            len >>= 1;
            index >>= 1;
            slice[index] = val;
        }
    }

    /// SegmentTreeの`range`の範囲の要素の総積を計算する.
    ///
    /// # Constraints
    ///
    /// - `range`は`0..self.len()`に含まれる区間である.
    ///
    /// # Time complexity
    ///
    /// - *O*(log *n*)
    #[must_use]
    pub fn prod(&self, range: impl std::ops::RangeBounds<usize>) -> M::T {
        let mut len = self.len();
        let mut left_index = match range.start_bound() {
            std::ops::Bound::Included(&i) => i,
            std::ops::Bound::Excluded(&i) => i + 1,
            std::ops::Bound::Unbounded => 0,
        };
        let mut right_index = match range.end_bound() {
            std::ops::Bound::Included(&i) => i + 1,
            std::ops::Bound::Excluded(&i) => i,
            std::ops::Bound::Unbounded => len,
        };
        debug_assert!(left_index <= right_index && right_index <= len);
        let mut left_val = self.2.e();
        let mut right_val = self.2.e();
        let mut slice = self.0.as_ref();
        while left_index != right_index {
            if left_index & 1 == 1 {
                left_val = self.2.op(&left_val, &slice[left_index]);
                left_index += 1;
            }
            if right_index & 1 == 1 {
                right_index -= 1;
                right_val = self.2.op(&slice[right_index], &right_val);
            }
            slice = &slice[len..];
            len >>= 1;
            left_index >>= 1;
            right_index >>= 1;
        }
        self.2.op(&left_val, &right_val)
    }

    /// `f(self.prod(index..x))`だが`!f(self.prod(index..=x))`な最小の`x`を見つけるような二分探索を行う.
    ///
    /// 見つからなかった場合は`self.len()`を返す.
    ///
    /// # Constraints
    ///
    /// - `f(monoid.e())`は`true`である.
    ///
    /// # Time complexity
    ///
    /// - *O*(log *n*)
    #[must_use]
    pub fn binary_search(&self, mut index: usize, f: impl Fn(&M::T) -> bool) -> usize {
        let mut sh = 0;
        let mut offset = 0;
        let mut r = self.2.e();
        debug_assert!(f(&r));
        while index < self.1 >> sh {
            while index & 1 == 0 && self.1 >> sh > 1 {
                index >>= 1;
                offset += self.1 >> sh;
                sh += 1;
            }
            if index == self.1 >> sh {
                break;
            }
            let temp = self.2.op(&r, &self.0[index + offset]);
            if !f(&temp) {
                break;
            }
            r = temp;
            index += 1;
        }
        while sh > 0 {
            sh -= 1;
            offset -= self.1 >> sh;
            index <<= 1;
            if index == self.1 >> sh {
                continue;
            }
            let temp = self.2.op(&r, &self.0[index + offset]);
            if f(&temp) {
                r = temp;
                index += 1;
            }
        }
        index
    }
}

impl<M: Monoid + Default> From<Vec<M::T>> for SegmentTree<M> {
    fn from(value: Vec<M::T>) -> Self {
        Self::from_vec(M::default(), value)
    }
}

impl<M: Monoid> AsRef<[M::T]> for SegmentTree<M> {
    fn as_ref(&self) -> &[M::T] {
        &self.0[..self.len()]
    }
}

impl<M: Monoid> std::ops::Index<usize> for SegmentTree<M> {
    type Output = M::T;

    fn index(&self, index: usize) -> &M::T {
        self.get(index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sum() {
        struct SumMonoid;
        impl super::super::util::Magma for SumMonoid {
            type T = i32;
            fn op(&self, a: &i32, b: &i32) -> i32 {
                a + b
            }
        }
        impl Associativity for SumMonoid {}
        impl Identity for SumMonoid {
            fn e(&self) -> i32 {
                0
            }
        }

        let mut seg = SegmentTree::from_vec(SumMonoid, vec![3, 1, 4, 1, 5, 9, 2, 6, 5, 3]);

        assert_eq!(seg.prod(0..3), 8);
        assert_eq!(seg.prod(1..8), 28);
        assert_eq!(seg.prod(..), 39);
        assert_eq!(seg.prod(4..4), 0);
        assert_eq!(seg.binary_search(0, |&v| v < 22), 5);
        assert_eq!(seg.binary_search(1, |&v| v < 22), 6);

        seg.set(4, -100);

        assert_eq!(seg.prod(0..3), 8);
        assert_eq!(seg.prod(1..8), -77);
        assert_eq!(seg.prod(..), -66);
        assert_eq!(seg.prod(4..4), 0);
        assert_eq!(seg.binary_search(0, |&v| v < 22), 10);
        assert_eq!(seg.binary_search(1, |&v| v < 22), 10);
    }
}
