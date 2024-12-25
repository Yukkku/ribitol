use super::util::{Associativity, Idempotence, Identity};

/// 冪等性を表すトレイト
///
/// 任意の`x`について`self.op(&x, &x) == x`を満たす必要がある
pub trait IdempotentMonoid: Associativity + Idempotence + Identity {}
impl<T: Associativity + Idempotence + Identity> IdempotentMonoid for T {}

/// 区間最小値クエリなどを定数時間で処理できるデータ構造
#[derive(Clone)]
pub struct SparseTable<M: IdempotentMonoid>(Box<[M::T]>, usize, M);

impl<M: IdempotentMonoid> SparseTable<M>
where
    M::T: std::fmt::Debug,
{
    /// 列からSparseTableを構築する
    ///
    /// # Time complexity
    ///
    /// - *O*(*N* log *N*)
    #[must_use]
    pub fn new(monoid: M, items: impl Into<Vec<M::T>>) -> Self {
        let mut items: Vec<M::T> = items.into();
        let len = items.len();
        if len == 0 {
            return Self(items.into(), len, monoid);
        }
        let log = (usize::BITS - len.leading_zeros()) as usize;
        items.reserve_exact((len + 1) * log - ((1 << log) - 1) - len);
        for i in 0..log - 1 {
            let offset = (len + 1) * i - ((1 << i) - 1);
            let span = 1 << i;
            for j in span..=len - span {
                items.push(monoid.op(&items[j - span + offset], &items[j + offset]))
            }
        }
        Self(items.into(), len, monoid)
    }

    /// 列の長さを返す
    ///
    /// # Time complexity
    ///
    /// - *O*(1)
    #[must_use]
    pub fn len(&self) -> usize {
        self.1
    }

    /// 列が空かどうか調べる
    ///
    /// # Time complexity
    ///
    /// - *O*(1)
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.1 == 0
    }

    /// `range`の範囲の総積を計算する
    ///
    /// # Constraints
    ///
    /// - `range`は`0..self.len()`に含まれる区間である.
    ///
    /// # Time complexity
    ///
    /// - *O*(1)
    #[must_use]
    pub fn query(&self, range: impl std::ops::RangeBounds<usize>) -> M::T {
        let len = self.1;
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
            return self.2.e();
        }

        let diff = right - left;
        if diff.is_power_of_two() {
            let log = diff.trailing_zeros() as usize;
            return self.0[(len + 1) * log - ((1 << log) - 1) + left].clone();
        }
        let log = (usize::BITS - diff.leading_zeros() - 1) as usize;
        let offset = (len + 1) * log - ((1 << log) - 1);
        self.2
            .op(&self.0[offset + left], &self.0[offset + right - (1 << log)])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn min_max() {
        struct MinMaxMonoid;
        impl super::super::util::Magma for MinMaxMonoid {
            type T = (i32, i32);
            fn op(&self, a: &(i32, i32), b: &(i32, i32)) -> (i32, i32) {
                (a.0.min(b.0), a.1.max(b.1))
            }
        }
        impl Identity for MinMaxMonoid {
            fn e(&self) -> (i32, i32) {
                (i32::MAX, i32::MIN)
            }
        }
        impl Associativity for MinMaxMonoid {}
        impl Idempotence for MinMaxMonoid {}

        let table = SparseTable::new(
            MinMaxMonoid,
            [
                (0, 1),
                (1, 2),
                (2, 3),
                (3, 4),
                (4, 5),
                (5, 6),
                (6, 7),
                (7, 8),
                (8, 9),
            ],
        );
        assert_eq!(table.query(0..4), (0, 4));
        assert_eq!(table.query(2..6), (2, 6));
        assert_eq!(table.query(3..), (3, 9));
        assert_eq!(table.query(..), (0, 9));
    }
}
