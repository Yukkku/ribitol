use std::{
    fmt::Debug,
    ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not},
};

/// 「2つの配列から小さい順に値を取り出す」を行うモジュール
mod merge {
    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
    pub enum MergeContent<T> {
        Left(T),
        Right(T),
        Both(T, T),
    }

    pub fn merge_iter<T: Ord>(
        lhs: impl IntoIterator<Item = T>,
        rhs: impl IntoIterator<Item = T>,
        mut f: impl FnMut(MergeContent<T>),
    ) {
        use std::cmp::Ordering::*;
        use MergeContent::*;
        let mut lhs = lhs.into_iter();
        let mut rhs = rhs.into_iter();

        let Some(mut litem) = lhs.next() else {
            for item in rhs {
                f(Right(item));
            }
            return;
        };
        let Some(mut ritem) = rhs.next() else {
            f(Left(litem));
            for item in lhs {
                f(Left(item));
            }
            return;
        };
        loop {
            match litem.cmp(&ritem) {
                Less => {
                    f(Left(litem));
                    litem = {
                        let Some(next_l) = lhs.next() else {
                            f(Right(ritem));
                            for item in rhs {
                                f(Right(item));
                            }
                            return;
                        };
                        next_l
                    };
                }
                Equal => {
                    f(Both(litem, ritem));
                    litem = {
                        let Some(next_l) = lhs.next() else {
                            for item in rhs {
                                f(Right(item));
                            }
                            return;
                        };
                        next_l
                    };
                    ritem = {
                        let Some(next_r) = rhs.next() else {
                            f(Left(litem));
                            for item in lhs {
                                f(Left(item));
                            }
                            return;
                        };
                        next_r
                    };
                }
                Greater => {
                    f(Right(ritem));
                    ritem = {
                        let Some(next_r) = rhs.next() else {
                            f(Left(litem));
                            for item in lhs {
                                f(Left(item));
                            }
                            return;
                        };
                        next_r
                    };
                }
            }
        }
    }
}

/// 集合を区間の組み合わせで表現するデータ構造
///
/// この型のドキュメントの計算量の表記では表現に用いている区間の個数を*N*と書くことにする.
#[derive(Clone)]
pub struct RangeSet<T: Ord>(bool, Vec<T>);

impl<T: Ord> RangeSet<T> {
    /// 空の集合を構築する
    ///
    /// # Time complexity
    ///
    /// - *O*(1)
    pub fn new() -> Self {
        Self(false, vec![])
    }

    /// 空の集合を構築する
    ///
    /// # Time complexity
    ///
    /// - *O*(log*N*)
    pub fn has(&self, item: &T) -> bool {
        (match self.1.binary_search(item) {
            Ok(i) => i & 1 == 0,
            Err(i) => i & 1 == 1,
        }) ^ self.0
    }

    fn build<F: Fn(bool, bool) -> bool>(&mut self, rhs: Self, f: F) {
        use merge::MergeContent::*;

        let mut flag = (self.0, rhs.0);
        let mut status = f(flag.0, flag.1);
        let lhs = std::mem::replace(self, Self(status, vec![]));
        merge::merge_iter(lhs.1, rhs.1, |content| {
            let item = match content {
                Left(item) => {
                    flag.0 ^= true;
                    item
                }
                Right(item) => {
                    flag.1 ^= true;
                    item
                }
                Both(item, _) => {
                    flag.0 ^= true;
                    flag.1 ^= true;
                    item
                }
            };
            let next = f(flag.0, flag.1);
            if status == next {
                return;
            }
            status = next;
            self.1.push(item);
        });
    }

    fn copy_build<F: Fn(bool, bool) -> bool>(&self, rhs: &Self, f: F) -> Self
    where
        T: Copy,
    {
        use merge::MergeContent::*;
        let lhs = self;

        let mut flag = (self.0, rhs.0);
        let mut status = f(flag.0, flag.1);
        let mut ret = Self(status, vec![]);
        merge::merge_iter(&lhs.1, &rhs.1, |content| {
            let item = match content {
                Left(item) => {
                    flag.0 ^= true;
                    item
                }
                Right(item) => {
                    flag.1 ^= true;
                    item
                }
                Both(item, _) => {
                    flag.0 ^= true;
                    flag.1 ^= true;
                    item
                }
            };
            let next = f(flag.0, flag.1);
            if status != next {
                return;
            }
            status = next;
            ret.1.push(*item);
        });
        ret
    }
}

impl<T: Ord> Not for RangeSet<T> {
    type Output = Self;
    fn not(self) -> Self {
        Self(!self.0, self.1)
    }
}

impl<T: Ord> BitAndAssign for RangeSet<T> {
    fn bitand_assign(&mut self, rhs: Self) {
        self.build(rhs, |x, y| x && y);
    }
}
impl<T: Ord> BitOrAssign for RangeSet<T> {
    fn bitor_assign(&mut self, rhs: Self) {
        self.build(rhs, |x, y| x || y);
    }
}
impl<T: Ord> BitXorAssign for RangeSet<T> {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.build(rhs, |x, y| x ^ y);
    }
}

impl<T: Ord + Copy> BitAndAssign<&Self> for RangeSet<T> {
    fn bitand_assign(&mut self, rhs: &Self) {
        *self = self.copy_build(rhs, |x, y| x && y);
    }
}
impl<T: Ord + Copy> BitOrAssign<&Self> for RangeSet<T> {
    fn bitor_assign(&mut self, rhs: &Self) {
        *self = self.copy_build(rhs, |x, y| x || y);
    }
}
impl<T: Ord + Copy> BitXorAssign<&Self> for RangeSet<T> {
    fn bitxor_assign(&mut self, rhs: &Self) {
        *self = self.copy_build(rhs, |x, y| x ^ y);
    }
}

impl<T: Ord> BitAnd for RangeSet<T> {
    type Output = Self;
    fn bitand(mut self, rhs: Self) -> Self {
        self &= rhs;
        self
    }
}
impl<T: Ord> BitOr for RangeSet<T> {
    type Output = Self;
    fn bitor(mut self, rhs: Self) -> Self {
        self |= rhs;
        self
    }
}
impl<T: Ord> BitXor for RangeSet<T> {
    type Output = Self;
    fn bitxor(mut self, rhs: Self) -> Self {
        self ^= rhs;
        self
    }
}

impl<T: Ord + Debug> Debug for RangeSet<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.1.len() == 0 {
            if self.0 {
                return f.write_str("RangeSet{(-inf, inf)}");
            } else {
                return f.write_str("RangeSet{}");
            }
        }
        f.write_str("RangeSet{")?;
        let mut comma = false;
        if self.0 {
            write!(f, "(-inf, {:?})", self.1[0])?;
            comma = true;
        }
        for slice in self.1[if self.0 { 1 } else { 0 }..].chunks(2) {
            if comma {
                f.write_str(", ")?;
            }
            if slice.len() == 1 {
                write!(f, "[{:?}, inf)", slice[0])?;
            } else {
                write!(f, "[{:?}, {:?})", slice[0], slice[1])?;
            }
            comma = true;
        }
        f.write_str("}")
    }
}

impl<T: Ord> From<std::ops::Range<T>> for RangeSet<T> {
    fn from(value: std::ops::Range<T>) -> Self {
        Self(false, vec![value.start, value.end])
    }
}
impl<T: Ord> From<std::ops::RangeFrom<T>> for RangeSet<T> {
    fn from(value: std::ops::RangeFrom<T>) -> Self {
        Self(false, vec![value.start])
    }
}
impl<T: Ord> From<std::ops::RangeTo<T>> for RangeSet<T> {
    fn from(value: std::ops::RangeTo<T>) -> Self {
        Self(true, vec![value.end])
    }
}
impl<T: Ord> From<std::ops::RangeFull> for RangeSet<T> {
    fn from(_: std::ops::RangeFull) -> Self {
        Self(true, vec![])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let a = RangeSet::from(1..13);
        let b = RangeSet::from(5..20);
        let c = RangeSet::from(10..);

        assert!(!a.has(&0));
        assert!(a.has(&1));
        assert!(a.has(&4));
        assert!(a.has(&10));
        assert!(a.has(&12));
        assert!(!a.has(&13));

        assert!(!b.has(&1));
        assert!(b.has(&5));
        assert!(b.has(&16));
        assert!(!b.has(&30));

        assert!(!c.has(&8));
        assert!(c.has(&998244353));

        let d = a & b;
        assert!(!d.has(&1));
        assert!(d.has(&6));
        assert!(d.has(&12));
        assert!(!d.has(&13));

        let e = d ^ c;
        assert!(!e.has(&3));
        assert!(e.has(&8));
        assert!(!e.has(&11));
        assert!(e.has(&15));
    }
}
