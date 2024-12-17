use super::segmenttree::Monoid;
use std::{num::NonZero, rc::Rc};

enum RawPersistentSegmentTree<M: Monoid> {
    Value(M::T),
    Relay(NonZero<u8>, M::T, (Rc<Self>, Rc<Self>)),
}
use RawPersistentSegmentTree::*;

impl<M: Monoid> RawPersistentSegmentTree<M> {
    fn val(&self) -> &M::T {
        match self {
            Value(v) => v,
            Relay(_, v, _) => v,
        }
    }

    fn set(&mut self, index: usize, item: M::T, monoid: &M) {
        match self {
            Value(v) => *v = item,
            Relay(s, x, (l, r)) => {
                let g = 1 << (s.get() - 1);
                if index < g {
                    Rc::make_mut(l).set(index, item, monoid);
                } else {
                    Rc::make_mut(r).set(index - g, item, monoid);
                }
                *x = monoid.op(l.val(), r.val());
            }
        }
    }

    fn prod_left(&self, mut index: usize, monoid: &M) -> M::T {
        let mut node = self;
        let mut val = monoid.e();
        while index > 0 {
            match node {
                Value(v) => return monoid.op(&val, v),
                Relay(s, _, children) => {
                    let g = 1 << (s.get() - 1);
                    if index < g {
                        node = children.0.as_ref();
                    } else {
                        val = monoid.op(&val, children.0.val());
                        node = children.1.as_ref();
                        index -= g;
                    }
                }
            }
        }

        return val;
    }

    fn prod_right(&self, mut index: usize, monoid: &M) -> M::T {
        let mut node = self;
        let mut val = monoid.e();
        while index > 0 {
            match node {
                Value(_) => return val,
                Relay(s, _, children) => {
                    let g = 1 << (s.get() - 1);
                    if index < g {
                        node = children.0.as_ref();
                        val = monoid.op(children.1.val(), &val);
                    } else {
                        node = children.1.as_ref();
                        index -= g;
                    }
                }
            }
        }
        return monoid.op(node.val(), &val);
    }
}

impl<M: Monoid> Clone for RawPersistentSegmentTree<M> {
    fn clone(&self) -> Self {
        match self {
            Self::Value(arg0) => Self::Value(arg0.clone()),
            Self::Relay(arg0, arg1, arg2) => Self::Relay(arg0.clone(), arg1.clone(), arg2.clone()),
        }
    }
}

/// 永続配列
#[derive(Clone, Default)]
pub struct PersistentSegmentTree<M: Monoid>(Option<Rc<RawPersistentSegmentTree<M>>>, M);

impl<M: Monoid> PersistentSegmentTree<M> {
    /// 空の永続SegmentTreeを作る
    ///
    /// # Time complexity
    ///
    /// - *O*(1)
    #[must_use]
    pub fn new(monoid: M, intoiter: impl IntoIterator<Item = M::T>) -> Self {
        let mut r: Vec<Option<RawPersistentSegmentTree<M>>> = vec![];
        'a: for v in intoiter {
            let mut v = Value(v);
            for (i, r) in r.iter_mut().enumerate() {
                if let Some(r) = std::mem::take(r) {
                    v = Relay(
                        NonZero::new(i as u8 + 1).unwrap(),
                        monoid.op(r.val(), v.val()),
                        (Rc::new(r), Rc::new(v)),
                    );
                } else {
                    *r = Some(v);
                    continue 'a;
                }
            }
            r.push(Some(v));
        }
        let mut t: Option<Rc<RawPersistentSegmentTree<M>>> = None;
        for (i, r) in r.into_iter().enumerate() {
            let Some(r) = r else {
                continue;
            };
            if let Some(s) = t {
                t = Some(Rc::new(Relay(
                    NonZero::new(i as u8 + 1).unwrap(),
                    monoid.op(r.val(), s.val()),
                    (Rc::new(r), s),
                )));
            } else {
                t = Some(Rc::new(r));
            }
        }
        Self(t, monoid)
    }

    /// 永続SegmentTreeの長さを返す
    ///
    /// # Time complexity
    ///
    /// - *O*(log *N*)
    #[must_use]
    pub fn len(&self) -> usize {
        let Some(node) = &self.0 else {
            return 0;
        };
        let mut node = node.as_ref();
        let mut len = 0;
        loop {
            match node {
                Value(_) => {
                    return len + 1;
                }
                Relay(level, _, (_, child)) => {
                    len += 1 << (level.get() - 1);
                    node = child;
                }
            }
        }
    }

    /// 値を設定する
    ///
    /// # Constraints
    ///
    /// - `index < self.len()`
    ///
    /// # Time complexity
    ///
    /// - *O*(log *N*)
    pub fn set(&mut self, index: usize, item: M::T) {
        debug_assert!(index < self.len());
        Rc::make_mut(self.0.as_mut().unwrap()).set(index, item, &self.1);
    }

    /// 区間の総積を計算する
    ///
    /// # Constraints
    ///
    /// - `range`は`0..self.len()`に含まれる区間である.
    ///
    /// # Time complexity
    ///
    /// - *O*(log *N*)
    pub fn prod(&self, range: impl std::ops::RangeBounds<usize>) -> M::T {
        let left = match range.start_bound() {
            std::ops::Bound::Included(&i) => Some(i),
            std::ops::Bound::Excluded(&i) => Some(i + 1),
            std::ops::Bound::Unbounded => None,
        };
        let right = match range.end_bound() {
            std::ops::Bound::Included(&i) => Some(i + 1),
            std::ops::Bound::Excluded(&i) => Some(i),
            std::ops::Bound::Unbounded => None,
        };
        let (mut left, mut right) = match (left, right) {
            (None | Some(0), None) => {
                return self
                    .0
                    .as_ref()
                    .map(|v| v.val().clone())
                    .unwrap_or_else(|| self.1.e())
            }
            (None | Some(0), Some(r)) => {
                return self
                    .0
                    .as_ref()
                    .map(|v| v.prod_left(r, &self.1))
                    .unwrap_or_else(|| {
                        debug_assert_eq!(r, 0);
                        self.1.e()
                    })
            }
            (Some(l), None) => {
                return self
                    .0
                    .as_ref()
                    .map(|v| v.prod_right(l, &self.1))
                    .unwrap_or_else(|| {
                        debug_assert_eq!(l, 0);
                        self.1.e()
                    })
            }
            (Some(l), Some(r)) => (l, r),
        };
        if left == right {
            return self.1.e();
        }
        let Some(mut node) = self.0.as_ref() else {
            panic!()
        };
        loop {
            match node.as_ref() {
                Value(v) => return v.clone(),
                Relay(s, _, children) => {
                    let g = 1 << (s.get() - 1);
                    match (left < g, right < g) {
                        (true, true) => node = &children.0,
                        (false, false) => {
                            node = &children.1;
                            left -= g;
                            right -= g;
                        }
                        (true, false) => {
                            return self.1.op(
                                &children.0.prod_right(left, &self.1),
                                &children.1.prod_left(right - g, &self.1),
                            );
                        }
                        (false, true) => unreachable!(),
                    }
                }
            }
        }
    }
}

impl<M: Monoid> std::ops::Index<usize> for PersistentSegmentTree<M> {
    type Output = M::T;

    fn index(&self, mut index: usize) -> &M::T {
        let Some(node) = &self.0 else {
            panic!("PersistentSegmentTree is empty.")
        };
        let mut node = node.as_ref();
        loop {
            match node {
                Value(v) => {
                    debug_assert!(index == 0);
                    return v;
                }
                Relay(level, _, children) => {
                    let m = 1 << (level.get() - 1);
                    if index < m {
                        node = &children.0;
                    } else {
                        index -= m;
                        node = &children.1;
                    }
                }
            }
        }
    }
}

impl<M: Monoid> std::fmt::Debug for PersistentSegmentTree<M>
where
    M::T: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn internal<M: Monoid>(
            v: &RawPersistentSegmentTree<M>,
            list: &mut std::fmt::DebugList<'_, '_>,
        ) where
            M::T: std::fmt::Debug,
        {
            match v {
                Value(v) => {
                    list.entry(v);
                }
                Relay(_, v, (l, r)) => {
                    internal(l, list);
                    list.entry(&(v,));
                    internal(r, list);
                }
            }
        }

        let mut list = f.debug_list();
        if let Some(v) = self.0.as_ref() {
            internal(v, &mut list);
        }
        list.finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sum() {
        #[derive(Clone)]
        struct SumMonoid;
        impl Monoid for SumMonoid {
            type T = i32;
            fn e(&self) -> i32 {
                0
            }
            fn op(&self, a: &i32, b: &i32) -> i32 {
                a + b
            }
        }

        let mut seg = PersistentSegmentTree::new(SumMonoid, 0..13);
        assert_eq!(seg.prod(1..5), 10);
        assert_eq!(seg.prod(4..10), 39);
        assert_eq!(seg.prod(..), 78);

        let mut seg2 = seg.clone();
        seg2.set(3, 8);
        assert_eq!(seg.prod(1..5), 10);
        assert_eq!(seg.prod(4..10), 39);
        assert_eq!(seg2.prod(1..5), 15);
        assert_eq!(seg2.prod(4..10), 39);

        seg.set(7, -3);
        assert_eq!(seg.prod(1..5), 10);
        assert_eq!(seg.prod(4..10), 29);
        assert_eq!(seg2.prod(1..5), 15);
        assert_eq!(seg2.prod(4..10), 39);
    }
}
