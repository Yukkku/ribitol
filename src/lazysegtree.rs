use super::segmenttree::Monoid;

pub trait LazyMonoid: Monoid {
    type F;

    fn id(&self) -> Self::F;
    fn comp(&self, g: &Self::F, f: &Self::F) -> Self::F;
    fn map(&self, f: &Self::F, x: &Self::T) -> Self::T;
}

/// 遅延伝搬セグメントツリー
///
/// 特定の条件を満たすクエリの区間更新・区間取得が可能である
pub struct LazySegmentTree<M: LazyMonoid>(Box<[M::T]>, Box<[M::F]>, M);

impl<M: LazyMonoid> LazySegmentTree<M> {
    #[must_use]
    pub fn new(monoid: M, n: usize) -> Self {
        Self(
            (0..n * 2 - n.count_ones() as usize)
                .map(|_| monoid.e())
                .collect(),
            (0..n - n.count_ones() as usize)
                .map(|_| monoid.id())
                .collect(),
            monoid,
        )
    }

    /// 配列`vec`からLazySegmentTreeを構築する.
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
        Self(
            vec.into_boxed_slice(),
            (0..n - n.count_ones() as usize)
                .map(|_| monoid.id())
                .collect(),
            monoid,
        )
    }

    /// 列の長さを返す.
    ///
    /// # Time complexity
    ///
    /// - *O*(1)
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len() - self.1.len()
    }

    /// 列が空かどうか判定する
    ///
    /// # Time complexity
    ///
    /// - *O*(1)
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn map(&mut self, index: usize) {
        let mut offset = self.0.len() - 1;
        let len = self.len();
        for i in (0..usize::BITS - 1 - len.leading_zeros()).rev() {
            let next_offset = offset - (len >> i);
            if len >> (i + 1) == index >> (i + 1) {
                offset = next_offset;
                continue;
            }
            let j = offset - len + (index >> (i + 1));
            let k = next_offset + ((index >> i) & !1);
            self.0[k] = self.2.map(&self.1[j], &self.0[k]);
            self.0[k + 1] = self.2.map(&self.1[j], &self.0[k + 1]);
            if i > 0 {
                self.1[k - len] = self.2.comp(&self.1[j], &self.1[k - len]);
                self.1[k + 1 - len] = self.2.comp(&self.1[j], &self.1[k + 1 - len]);
            }
            self.1[j] = self.2.id();
            offset = next_offset;
        }
    }

    fn map_range(&mut self, l: usize, r: usize) {
        let r = r - 1;
        let len = self.len();
        let mut offset = self.1.len();
        for i in (1..usize::BITS - len.leading_zeros()).rev() {
            offset -= len >> i;
            if len >> i == l >> i {
                continue;
            }
            let n_offset = offset + len - (len >> (i - 1));
            {
                let j = offset + (l >> i);
                let k = n_offset + ((l >> (i - 1)) & !1);
                self.0[k] = self.2.map(&self.1[j], &self.0[k]);
                self.0[k + 1] = self.2.map(&self.1[j], &self.0[k + 1]);
                if i > 1 {
                    self.1[k - len] = self.2.comp(&self.1[j], &self.1[k - len]);
                    self.1[k + 1 - len] = self.2.comp(&self.1[j], &self.1[k + 1 - len]);
                }
                self.1[j] = self.2.id();
            }
            if len >> i == r >> i || r >> i == l >> i {
                continue;
            }
            {
                let j = offset + (r >> i);
                let k = n_offset + ((r >> (i - 1)) & !1);
                self.0[k] = self.2.map(&self.1[j], &self.0[k]);
                self.0[k + 1] = self.2.map(&self.1[j], &self.0[k + 1]);
                if i > 1 {
                    self.1[k - len] = self.2.comp(&self.1[j], &self.1[k - len]);
                    self.1[k + 1 - len] = self.2.comp(&self.1[j], &self.1[k + 1 - len]);
                }
                self.1[j] = self.2.id();
            }
        }
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

    fn update_range(&mut self, l: usize, r: usize) {
        let mut len = self.len();
        let mut slice = self.0.as_mut();
        for i in 0.. {
            if l >> (i + 1) >= len >> 1 {
                break;
            }
            if l >> (i + 1) != ((l - 1) >> (i + 1)) + 1 {
                slice[len + (l >> (i + 1))] =
                    self.2.op(&slice[(l >> i) & !1], &slice[(l >> i) | 1]);
            }
            if r >> (i + 1) < len >> 1
                && l >> (i + 1) != r >> (i + 1)
                && r >> (i + 1) != ((r - 1) >> (r + 1)) + 1
            {
                slice[len + ((r - 1) >> (i + 1)) + 1] = self.2.op(
                    &slice[(((r - 1) >> i) + 2) & !1],
                    &slice[(((r - 1) >> i) + 2) | 1],
                );
            }
            slice = &mut slice[len..];
            len >>= 1;
        }
    }

    /// 指定した位置の値を変更する
    ///
    /// # Constraints
    ///
    /// - `index < self.len()`
    ///
    /// # Time complexity
    ///
    /// - *O*(log *n*)
    pub fn set(&mut self, index: usize, item: M::T) {
        debug_assert!(index < self.len());

        self.map(index);
        self.0[index] = item;
        self.update(index);
    }

    /// 指定した位置の要素の可変参照のラッパーを返す
    ///
    /// # Constraints
    ///
    /// - `index < self.len()`
    ///
    /// # Time complexity
    ///
    /// - *O*(log *n*)
    /// - *O*(log *n*) (デストラクタ)
    #[must_use]
    pub fn setter(&mut self, index: usize) -> impl std::ops::DerefMut<Target = M::T> + use<'_, M> {
        debug_assert!(index < self.len());

        struct Wrapper<'a, M: LazyMonoid>(&'a mut LazySegmentTree<M>, usize);
        impl<M: LazyMonoid> std::ops::Deref for Wrapper<'_, M> {
            type Target = M::T;
            fn deref(&self) -> &M::T {
                &self.0 .0[self.1]
            }
        }
        impl<M: LazyMonoid> std::ops::DerefMut for Wrapper<'_, M> {
            fn deref_mut(&mut self) -> &mut M::T {
                &mut self.0 .0[self.1]
            }
        }
        impl<M: LazyMonoid> Drop for Wrapper<'_, M> {
            fn drop(&mut self) {
                self.0.update(self.1);
            }
        }

        self.map(index);
        Wrapper(self, index)
    }

    /// 指定した位置の値を取得する
    ///
    /// # Constraints
    ///
    /// - `index < self.len()`
    ///
    /// # Time complexity
    ///
    /// - *O*(log *n*)
    #[must_use]
    pub fn get(&mut self, index: usize) -> &M::T {
        debug_assert!(index < self.len());
        self.map(index);
        &self.0[index]
    }

    /// 指定した位置の値を取得する
    ///
    /// # Constraints
    ///
    /// - `index < self.len()`
    ///
    /// # Time complexity
    ///
    /// - *O*(log *n*)
    #[must_use]
    pub fn get_imu(&self, index: usize) -> M::T {
        debug_assert!(index < self.len());
        let len = self.len();
        let mut f = self.2.id();
        let mut offset = self.1.len();
        for i in (1..usize::BITS - len.leading_zeros()).rev() {
            offset -= len >> i;
            if len >> i == index >> i {
                continue;
            }
            f = self.2.comp(&f, &self.1[offset + (index >> i)]);
        }
        self.2.map(&f, &self.0[index])
    }

    /// 指定した区間の値の総積を計算する
    ///
    /// # Constraints
    ///
    /// - `range`は`0..self.len()`に含まれる区間である.
    ///
    /// # Time complexity
    ///
    /// - *O*(log *n*)
    pub fn prod(&mut self, range: impl std::ops::RangeBounds<usize>) -> M::T {
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
        if left_index == right_index {
            return self.2.e();
        }
        self.map_range(left_index, right_index);
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

    /// 指定した区間に作用素`f`を適用する
    ///
    /// # Constraints
    ///
    /// - `range`は`0..self.len()`に含まれる区間である.
    ///
    /// # Time complexity
    ///
    /// - *O*(log *n*)
    pub fn apply(&mut self, range: impl std::ops::RangeBounds<usize>, f: &M::F) {
        let mut len = self.len();
        let left = match range.start_bound() {
            std::ops::Bound::Included(&i) => i,
            std::ops::Bound::Excluded(&i) => i + 1,
            std::ops::Bound::Unbounded => 0,
        };
        let right = match range.end_bound() {
            std::ops::Bound::Included(&i) => i + 1,
            std::ops::Bound::Excluded(&i) => i,
            std::ops::Bound::Unbounded => len,
        };
        debug_assert!(left <= right && right <= len);
        if left == right {
            return;
        }
        self.map_range(left, right);
        {
            let mut left = left;
            let mut right = right;
            if left & 1 == 1 {
                self.0[left] = self.2.map(f, &self.0[left]);
                left += 1;
            }
            if right & 1 == 1 {
                right -= 1;
                self.0[right] = self.2.map(f, &self.0[right]);
            }
            let mut slice = self.1.as_mut();
            let mut slice_v = &mut self.0[len..];
            len >>= 1;
            left >>= 1;
            right >>= 1;
            while left != right {
                if left & 1 == 1 {
                    slice[left] = self.2.comp(f, &slice[left]);
                    slice_v[left] = self.2.map(f, &slice_v[left]);
                    left += 1;
                }
                if right & 1 == 1 {
                    right -= 1;
                    slice[right] = self.2.comp(f, &slice[right]);
                    slice_v[right] = self.2.map(f, &slice_v[right]);
                }
                slice = &mut slice[len..];
                slice_v = &mut slice_v[len..];
                len >>= 1;
                left >>= 1;
                right >>= 1;
            }
        }
        self.update_range(left, right);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn minmax_add() {
        struct TestMonoid;
        impl super::super::util::Magma for TestMonoid {
            type T = (i32, i32);
            fn op(&self, &a: &(i32, i32), &b: &(i32, i32)) -> (i32, i32) {
                (a.0.min(b.0), a.1.max(b.1))
            }
        }
        impl super::super::util::Identity for TestMonoid {
            fn e(&self) -> (i32, i32) {
                (i32::MAX, i32::MIN)
            }
        }
        impl super::super::util::Associativity for TestMonoid {}
        impl LazyMonoid for TestMonoid {
            type F = i32;
            fn id(&self) -> i32 {
                0
            }
            fn comp(&self, &g: &i32, &f: &i32) -> i32 {
                g + f
            }
            fn map(&self, &f: &i32, &x: &(i32, i32)) -> (i32, i32) {
                (
                    if x.0 == i32::MAX { i32::MAX } else { x.0 + f },
                    if x.1 == i32::MIN { i32::MIN } else { x.1 + f },
                )
            }
        }

        let mut seg = LazySegmentTree::from_vec(
            TestMonoid,
            vec![
                (0, 0),
                (1, 1),
                (2, 2),
                (3, 3),
                (4, 4),
                (5, 5),
                (6, 6),
                (7, 7),
                (8, 8),
                (9, 9),
            ],
        );
        assert_eq!(seg.prod(2..7), (2, 6));
        assert_eq!(seg.prod(5..), (5, 9));
        seg.apply(1..4, &12);
        assert_eq!(seg.prod(2..7), (4, 15));
        assert_eq!(seg.prod(5..), (5, 9));
        seg.apply(3..8, &-20);
        assert_eq!(seg.prod(2..7), (-16, 14));
        assert_eq!(seg.prod(5..), (-15, 9));
    }
}
