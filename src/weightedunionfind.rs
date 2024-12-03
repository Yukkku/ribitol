use std::num::NonZero;

/// 群を表すトレイト
pub trait Group {
    /// 群の元を表現する型
    type T: Eq;

    /// 群の単位元を返す
    #[must_use]
    fn e(&self) -> Self::T;

    /// 逆元を求める
    #[must_use]
    fn inv(&self, a: &Self::T) -> Self::T;

    /// 群の演算
    ///
    /// 以下の条件を満たす.
    /// - 任意の `x`, `y`, `z` について `self.op(&self.op(&x, &y), &z) == self.op(&x, &self.op(&y, &z))`
    /// - 任意の `x` について `self.op(&self.e(), &x) == x && self.op(&x, &self.e()) == x`
    #[must_use]
    fn op(&self, a: &Self::T, b: &Self::T) -> Self::T;

    /// 逆元との積を求める
    ///
    /// `self.op(a, &self.inv(b))` と同じ
    #[must_use]
    fn opinv(&self, a: &Self::T, b: &Self::T) -> Self::T {
        self.op(a, &self.inv(b))
    }

    /// 逆元の積を求める
    ///
    /// `self.op(&self.inv(a), b)` と同じ
    #[must_use]
    fn invop(&self, a: &Self::T, b: &Self::T) -> Self::T {
        self.op(&self.inv(a), b)
    }
}

#[derive(Copy, Clone)]
enum Node<G: Group> {
    Root(NonZero<usize>, bool),
    NotRoot(usize, G::T),
}

/// `WeightedUnionFind`での値の差の評価
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Diff<T> {
    /// 情報が矛盾していることを表す.
    Invaild,
    /// 情報が不足していて答えが分からないことを表す.
    Unconnected,
    /// 答えが一意に定まることを表す.
    Connected(T),
}

pub struct WeightedUnionFind<G: Group>(Box<[Node<G>]>, G);

impl<G: Group> WeightedUnionFind<G> {
    /// `n`個の要素からなる群`group`による`WeightedUnionFind`を作成する.
    ///
    /// 仮想的には長さ`n`の群`group`の要素からなる配列`x`が作られる.
    #[must_use]
    pub fn new(group: G, n: usize) -> Self {
        const ONE: NonZero<usize> = unsafe { NonZero::new_unchecked(1) };

        Self(
            std::iter::repeat_with(|| Node::Root(ONE, true))
                .take(n)
                .collect(),
            group,
        )
    }

    /// 要素数を返す.
    ///
    /// # Time complexity
    ///
    /// - *O*(1)
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// `group.opinv(x[a], x[b])` が一意に定まるか判定し, 一意なら計算する.
    ///
    /// # Time complexity
    ///
    /// - *O*(α(*n*))
    #[must_use]
    pub fn diff(&mut self, a: usize, b: usize) -> Diff<G::T> {
        debug_assert!(a < self.len());
        debug_assert!(b < self.len());
        let (a, _, ah) = self.diff_internal(a);
        let (b, _, bh) = self.diff_internal(b);
        if a != b {
            Diff::Unconnected
        } else if let (Some(ah), Some(bh)) = (ah, bh) {
            Diff::Connected(self.1.opinv(&ah, &bh))
        } else {
            Diff::Invaild
        }
    }

    /// `group.opinv(x[a], x[b])` が一意に定まるか判定し, 一意なら計算する.
    ///
    /// # Time complexity
    ///
    /// - *O*(log *n*)
    #[must_use]
    pub fn diff_imu(&self, a: usize, b: usize) -> Diff<G::T> {
        debug_assert!(a < self.len());
        debug_assert!(b < self.len());
        let (a, _, ah) = self.diff_internal_imu(a);
        let (b, _, bh) = self.diff_internal_imu(b);
        if a != b {
            Diff::Unconnected
        } else if let (Some(ah), Some(bh)) = (ah, bh) {
            Diff::Connected(self.1.opinv(&ah, &bh))
        } else {
            Diff::Invaild
        }
    }

    /// UnionFindとしてどの連結成分に属しているか判別する.
    ///
    /// # Time complexity
    ///
    /// - *O*(α(*n*))
    #[must_use]
    pub fn find(&mut self, a: usize) -> usize {
        debug_assert!(a < self.len());
        self.find_internal(a).0
    }

    /// UnionFindとしてどの連結成分に属しているか判別する.
    ///
    /// # Time complexity
    ///
    /// - *O*(log *n*)
    #[must_use]
    pub fn find_imu(&self, a: usize) -> usize {
        debug_assert!(a < self.len());
        self.find_internal_imu(a).0
    }

    /// UnionFindとして, 連結成分の要素数を数える.
    ///
    /// # Time complexity
    ///
    /// - *O*(α(*n*))
    #[must_use]
    pub fn size(&mut self, a: usize) -> NonZero<usize> {
        debug_assert!(a < self.len());
        self.find_internal(a).1
    }

    /// UnionFindとして, 連結成分の要素数を数える.
    ///
    /// # Time complexity
    ///
    /// - *O*(log *n*)
    #[must_use]
    pub fn size_imu(&self, a: usize) -> NonZero<usize> {
        debug_assert!(a < self.len());
        self.find_internal_imu(a).1
    }

    fn find_internal(&mut self, mut a: usize) -> (usize, NonZero<usize>, bool) {
        loop {
            match &self.0[a] {
                Node::Root(s, v) => return (a, *s, *v),
                Node::NotRoot(b, f) => match &self.0[*b] {
                    Node::Root(s, v) => return (*b, *s, *v),
                    Node::NotRoot(c, g) => {
                        let c = *c;
                        let nf = self.1.op(f, g);
                        self.0[a] = Node::NotRoot(c, nf);
                        a = c;
                    }
                },
            }
        }
    }
    fn find_internal_imu(&self, mut a: usize) -> (usize, NonZero<usize>, bool) {
        loop {
            match &self.0[a] {
                Node::Root(s, v) => return (a, *s, *v),
                Node::NotRoot(b, _) => a = *b,
            }
        }
    }

    fn diff_internal(&mut self, mut a: usize) -> (usize, NonZero<usize>, Option<G::T>) {
        use Node::*;
        let mut h = self.1.e();
        loop {
            match &self.0[a] {
                Root(s, v) => return (a, *s, if *v { Some(h) } else { None }),
                NotRoot(b, f) => match &self.0[*b] {
                    Root(s, v) => return (*b, *s, if *v { Some(self.1.op(&h, f)) } else { None }),
                    NotRoot(c, g) => {
                        let c = *c;
                        let nf = self.1.op(f, g);
                        h = self.1.op(&h, &nf);
                        self.0[a] = NotRoot(c, nf);
                        a = c;
                    }
                },
            }
        }
    }
    fn diff_internal_imu(&self, mut a: usize) -> (usize, NonZero<usize>, Option<G::T>) {
        use Node::*;
        let mut h = self.1.e();
        loop {
            match &self.0[a] {
                Root(s, v) => return (a, *s, if *v { Some(h) } else { None }),
                NotRoot(x, y) => {
                    h = self.1.op(&h, y);
                    a = *x;
                }
            }
        }
    }

    /// `group.opinv(x[a], x[b]) ==  diff` だという情報を追加する.
    ///
    /// UnionFindとしては, 要素`a`が属するグループと要素`b`が属するグループを1つのグループにマージする.
    ///
    /// # Time complexity
    ///
    /// - *O*(α(*n*))
    pub fn union(&mut self, a: usize, b: usize, diff: &G::T) {
        debug_assert!(a < self.len());
        debug_assert!(b < self.len());
        let (a, a_size, ah) = self.diff_internal(a);
        let (b, b_size, bh) = self.diff_internal(b);
        if a == b {
            if let (Some(ah), Some(bh)) = (ah, bh) {
                if &self.1.opinv(&ah, &bh) != diff {
                    self.0[a] = Node::Root(a_size, false);
                }
            }
            return;
        }
        let size = a_size.checked_add(b_size.get()).unwrap();
        if let (Some(ah), Some(bh)) = (ah, bh) {
            if a_size > b_size {
                self.0[a] = Node::Root(size, true);
                self.0[b] = Node::NotRoot(a, self.1.invop(&self.1.op(diff, &bh), &ah));
            } else {
                self.0[b] = Node::Root(size, true);
                self.0[a] = Node::NotRoot(b, self.1.op(&self.1.invop(&ah, diff), &bh));
            }
        } else {
            if a_size > b_size {
                self.0[a] = Node::Root(size, false);
                self.0[b] = Node::NotRoot(a, self.1.e());
            } else {
                self.0[b] = Node::Root(size, false);
                self.0[a] = Node::NotRoot(b, self.1.e());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diff() {
        /// ℤの加法群
        struct IntGroup;
        impl Group for IntGroup {
            type T = i32;
            fn e(&self) -> i32 {
                0
            }
            fn inv(&self, a: &i32) -> i32 {
                -a
            }
            fn op(&self, a: &i32, b: &i32) -> i32 {
                a + b
            }
        }

        let mut wuf = WeightedUnionFind::new(IntGroup, 4);

        wuf.union(1, 0, &9);
        assert_eq!(wuf.diff(0, 2), Diff::Unconnected);
        wuf.union(1, 2, &15);
        assert_eq!(wuf.diff(2, 0), Diff::Connected(-6));
        wuf.union(0, 2, &6);
        assert_eq!(wuf.diff(2, 0), Diff::Connected(-6));
        wuf.union(1, 3, &5);
        wuf.union(2, 3, &8);
        assert_eq!(wuf.diff(2, 0), Diff::Invaild);
    }
}
