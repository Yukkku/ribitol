use super::util::HasMin;

/// `RadixHeap` に値を乗せるためのトレイト
pub trait Radix: Copy + Ord + HasMin {
    /// `radix_dist` が返す可能性のある最大の値.
    const MAX_DIST: usize;

    /// `RadixHeap`で用いられる超距離関数
    ///
    /// 以下の条件を満たす.
    /// 以下, `x`, `y`, `z` を `Self` のインスタンスとする.
    /// - 任意の `x` , `y` について `x.radix_dist(y) <= Self::MIN`
    /// - 任意の `x` , `y` について `x.radix_dist(y) == 0` と `x == y` が同値
    /// - 任意の `x` , `y`, `z` について `x < y && y < z` なら `x.radix_dist(y) <= x.radix_dist(z)`
    /// - 任意の `x` , `y`, `z` について `x.radix_dist(y) < x.radix_dist(z)` なら `x.radix_dist(z) == y.radix_dist(z)`
    #[must_use]
    fn radix_dist(&self, rhs: &Self) -> usize;
}

// 整数型にRadixを実装するマクロ
macro_rules! impl_int {
    ($($t: ty),*) => {$(
        impl Radix for $t {
            const MAX_DIST: usize = Self::BITS as usize;

            fn radix_dist(&self, rhs: &Self) -> usize {
                (Self::BITS - (self ^ rhs).leading_zeros()) as usize
            }
        }
    )*};
}

impl_int! { u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize }

/// 基数ヒープ
///
/// 「最後に取り出した値より小さい値を入れることが出来ない」などの制限がある代わりに高速に動作する.
#[derive(Clone)]
pub struct RadixHeap<T: Radix> {
    buckets: Box<[Vec<T>]>,
    // 最後に取り出した値 (最初はその型の最小値が入っている)
    last: T,
    // 要素数
    len: usize,
}

impl<T: Radix> RadixHeap<T> {
    /// 新しい空の `RadixHeap<T>` を作成する.
    ///
    /// # Time complexity
    ///
    /// - *O*(`T::MAX_DIST`)
    #[must_use]
    pub fn new() -> Self {
        Self {
            buckets: std::iter::repeat_with(Vec::new)
                .take(T::MAX_DIST + 1)
                .collect(),
            last: T::min_value(),
            len: 0,
        }
    }

    /// 要素をヒープに追加する.
    /// このとき追加する要素は `self.last()` よりも大きい必要がある.
    ///
    /// # Constraints
    ///
    /// - `item >= self.last()`
    ///
    /// # Time complexity
    ///
    /// - *O*(1)
    pub fn push(&mut self, item: T) {
        debug_assert!(item >= self.last);
        self.len += 1;
        self.buckets[self.last.radix_dist(&item)].push(item);
    }

    /// ヒープから最小の要素を削除し, その要素を返す.
    /// ヒープが空の場合は `None` を返す.
    ///
    /// # Time complexity
    ///
    /// - *O*(`T::MAX_DIST`)
    pub fn pop(&mut self) -> Option<T> {
        if let Some(r) = self.buckets[0].pop() {
            self.len -= 1;
            return Some(r);
        };
        let rak = std::mem::take(self.buckets.iter_mut().find(|v| !v.is_empty())?);

        self.last = *rak.iter().min().unwrap();

        for v in rak {
            self.buckets[self.last.radix_dist(&v)].push(v);
        }

        self.len -= 1;
        self.buckets[0].pop()
    }

    /// ヒープの持つ要素の総数を返す.
    ///
    /// # Time complexity
    ///
    /// - *O*(1)
    #[must_use]
    pub fn len(&self) -> usize {
        self.len
    }

    /// ヒープが空かどうか調べる.
    ///
    /// # Time complexity
    ///
    /// - *O*(1)
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// ヒープから最後に取り出した値を返す.
    /// ヒープからまだ値を取り出したことが無ければ, `T::MIN` を返す.
    ///
    /// # Time complexity
    ///
    /// - *O*(1)
    #[must_use]
    pub fn last(&self) -> &T {
        &self.last
    }
}

impl<T: Radix> Default for RadixHeap<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<A: Radix> FromIterator<A> for RadixHeap<A> {
    fn from_iter<T: IntoIterator<Item = A>>(iter: T) -> Self {
        let mut heap = RadixHeap::new();
        for item in iter {
            heap.push(item);
        }
        heap
    }
}

impl<A: Radix> Extend<A> for RadixHeap<A> {
    fn extend<T: IntoIterator<Item = A>>(&mut self, iter: T) {
        for item in iter {
            self.push(item);
        }
    }
}

/// RadixHeapの値を取り出すためのイテレータ
pub struct RadixIter<T: Radix>(pub RadixHeap<T>);
impl<T: Radix> Iterator for RadixIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        self.0.pop()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len(), Some(self.len()))
    }
}
impl<T: Radix> ExactSizeIterator for RadixIter<T> {
    fn len(&self) -> usize {
        self.0.len()
    }
}
impl<T: Radix> std::iter::FusedIterator for RadixIter<T> {}

impl<T: Radix> IntoIterator for RadixHeap<T> {
    type Item = T;

    type IntoIter = RadixIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        RadixIter(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unsigned_int() {
        let mut heap = RadixHeap::<u32>::new();

        heap.push(314159);
        heap.push(0);
        heap.push(265358);
        heap.push(u32::MAX);

        assert_eq!(heap.len(), 4);
        assert_eq!(heap.pop(), Some(0));
        assert_eq!(heap.pop(), Some(265358));
        assert_eq!(heap.len(), 2);

        heap.push(979323);
        heap.push(846264);

        assert_eq!(heap.len(), 4);
        assert_eq!(heap.pop(), Some(314159));
        assert_eq!(heap.pop(), Some(846264));
        assert_eq!(heap.pop(), Some(979323));
        assert_eq!(heap.pop(), Some(u32::MAX));
        assert_eq!(heap.pop(), None);
        assert_eq!(heap.len(), 0);
    }

    #[test]
    fn signed_int() {
        let mut heap = RadixHeap::<i32>::new();

        heap.push(-314159);
        heap.push(0);
        heap.push(-265358);
        heap.push(i32::MAX);
        heap.push(i32::MIN);

        assert_eq!(heap.len(), 5);
        assert_eq!(heap.pop(), Some(i32::MIN));
        assert_eq!(heap.pop(), Some(-314159));
        assert_eq!(heap.pop(), Some(-265358));
        assert_eq!(heap.len(), 2);

        heap.push(979323);
        heap.push(846264);

        assert_eq!(heap.len(), 4);
        assert_eq!(heap.pop(), Some(0));
        assert_eq!(heap.pop(), Some(846264));
        assert_eq!(heap.pop(), Some(979323));
        assert_eq!(heap.pop(), Some(i32::MAX));
        assert_eq!(heap.pop(), None);
        assert_eq!(heap.len(), 0);
    }
}
