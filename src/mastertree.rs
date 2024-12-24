use std::{cell::UnsafeCell, rc::Rc};

/// 疑似乱数生成器 (xorshift64)
#[derive(Clone, Copy)]
struct Rng(u64);
impl Rng {
    fn new() -> Self {
        // 疑似乱数のデフォルトシード
        const SEED: u64 = {
            const BASE: u64 = 0xf285692d6bf31f57;
            let mut r: u64 = 1;
            let mut h: u64 = 0;
            let bytes = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/", file!()));
            let mut i = 0;
            while i < bytes.len() {
                h = h.wrapping_add(r.wrapping_mul(bytes[i] as u64));
                r = r.wrapping_mul(BASE);
                i += 1;
            }
            if h == 0 {
                BASE
            } else {
                h
            }
        };
        Self(SEED)
    }
    fn gen(&mut self) -> u64 {
        self.0 ^= self.0 << 7;
        self.0 ^= self.0 >> 9;
        self.0
    }
    fn choose(&mut self, a: usize, b: usize) -> bool {
        ((self.gen() as u128 * (a + b) as u128 >> 64) as usize) < a
    }

    fn make(&mut self) -> Self {
        self.gen();
        let mut r = *self;
        r.0 ^= r.0 >> 7;
        r.0 ^= r.0 << 9;
        r
    }
}

/// MasterTreeを制御するためのトレイト
pub trait MasterManager {
    /// 列の値の型
    type T: Clone;
    /// 区間の情報 (区間和・遅延伝搬の情報) を持つ型
    type Info: Clone;
    /// モノイドの型
    type Prod: Clone;
    /// 作用素の型
    type Lazy: Clone;

    /// left, mid, rightをこの順で繋げた列の区間のInfoを生成する
    ///
    /// left, rightには(Some(区間の情報), 区間の長さ)というタプルが与えられる
    /// (ただし, その区間が空であった場合は(None, 0)が与えられる)
    #[must_use]
    fn make_info(
        left: (Option<&Self::Info>, usize),
        mid: &Self::T,
        right: (Option<&Self::Info>, usize),
    ) -> Self::Info;

    /// 列が反転されたときの区間の情報を得る
    fn rev(info: &mut Self::Info, len: usize);
    /// Infoに作用素を適用する
    fn apply_info(info: &mut Self::Info, len: usize, lazy: &Self::Lazy);
    /// valに作用素を適用する
    fn apply_val(val: &mut Self::T, lazy: &Self::Lazy);
    /// 遅延伝搬する
    fn propagate(
        info: &mut Self::Info,
        left: (Option<&mut Self::Info>, usize),
        val: &mut Self::T,
        right: (Option<&mut Self::Info>, usize),
    );

    /// Info型からProd型に変換する
    #[must_use]
    fn info2prod(info: &Self::Info) -> Self::Prod;
    /// T型からProd型に変換する
    #[must_use]
    fn val2prod(val: &Self::T) -> Self::Prod;

    /// モノイドの単位元を得る
    #[must_use]
    fn e() -> Self::Prod;
    /// モノイドの演算
    #[must_use]
    fn op(left: Self::Prod, right: Self::Prod) -> Self::Prod;
}

struct Node<M: MasterManager> {
    val: M::T,
    info: M::Info,
    idx: usize,
    rev: bool,
    left: Option<NodeWrapper<M>>,
    right: Option<NodeWrapper<M>>,
}
impl<M: MasterManager> Clone for Node<M> {
    fn clone(&self) -> Self {
        Self {
            val: self.val.clone(),
            info: self.info.clone(),
            idx: self.idx,
            rev: self.rev,
            left: self.left.clone(),
            right: self.right.clone(),
        }
    }
}
impl<M: MasterManager> Node<M> {
    fn setup(&mut self, len: usize) {
        M::propagate(
            &mut self.info,
            (self.left.as_mut().map(|v| &mut v.as_mut().info), self.idx),
            &mut self.val,
            (
                self.right.as_mut().map(|v| &mut v.as_mut().info),
                len - 1 - self.idx,
            ),
        );
        if self.rev {
            M::rev(&mut self.info, len);
            std::mem::swap(&mut self.left, &mut self.right);
            if let Some(v) = &mut self.left {
                v.as_mut().rev ^= true;
            }
            if let Some(v) = &mut self.right {
                v.as_mut().rev ^= true;
            }
            self.idx = len - self.idx - 1;
            self.rev = false;
        }
    }

    fn update(&mut self, len: usize) {
        self.info = M::make_info(
            (self.left.as_ref().map(|v| &v.as_ref().info), self.idx),
            &self.val,
            (
                self.right.as_ref().map(|v| &v.as_ref().info),
                len - 1 - self.idx,
            ),
        );
    }
}

struct NodeWrapper<M: MasterManager>(Rc<UnsafeCell<Node<M>>>);
impl<M: MasterManager> Clone for NodeWrapper<M> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
impl<M: MasterManager> AsRef<Node<M>> for NodeWrapper<M> {
    fn as_ref(&self) -> &Node<M> {
        unsafe { &*self.0.get() }
    }
}
impl<M: MasterManager> AsMut<Node<M>> for NodeWrapper<M> {
    fn as_mut(&mut self) -> &mut Node<M> {
        unsafe {
            if Rc::weak_count(&self.0) == 0 && Rc::strong_count(&self.0) == 1 {
                Rc::get_mut(&mut self.0).unwrap_unchecked().get_mut()
            } else {
                let v = &*self.0.get();
                self.0 = Rc::new(UnsafeCell::new(v.clone()));
                Rc::get_mut(&mut self.0).unwrap_unchecked().get_mut()
            }
        }
    }
}

impl<M: MasterManager> NodeWrapper<M> {
    fn new(item: M::T) -> Self {
        let info = M::make_info((None, 0), &item, (None, 0));
        Self(Rc::new(UnsafeCell::new(Node {
            val: item,
            info,
            idx: 0,
            rev: false,
            left: None,
            right: None,
        })))
    }

    fn setup(&self, len: usize) -> &Node<M> {
        let r = unsafe { &mut *self.0.get() };
        r.setup(len);
        r
    }

    fn index(&self, mut len: usize, mut index: usize) -> &M::T {
        use std::cmp::Ordering::*;
        let mut node = self.setup(len);
        loop {
            let idx = node.idx;
            match index.cmp(&idx) {
                Less => {
                    len = idx;
                    node = node.left.as_ref().unwrap().setup(len);
                }
                Equal => return &node.val,
                Greater => {
                    index -= idx + 1;
                    len -= idx + 1;
                    node = node.right.as_ref().unwrap().setup(len);
                }
            }
        }
    }

    fn merge(lhs: (Self, usize), rhs: (Self, usize), rng: &mut Rng) -> Self {
        let len = lhs.1 + rhs.1;
        let mut ret;
        if rng.choose(lhs.1, rhs.1) {
            ret = lhs.0;
            ret.setup(lhs.1);
            let ri = ret.as_mut();
            if let Some(k) = std::mem::take(&mut ri.right) {
                ri.right = Some(Self::merge((k, lhs.1 - 1 - ri.idx), rhs, rng));
            } else {
                ri.right = Some(rhs.0);
            }
            ri.update(len);
        } else {
            ret = rhs.0;
            ret.setup(rhs.1);
            let ri = ret.as_mut();
            let nidx = ri.idx + lhs.1;
            if let Some(k) = std::mem::take(&mut ri.left) {
                ri.left = Some(Self::merge(lhs, (k, ri.idx), rng));
            } else {
                ri.left = Some(lhs.0);
            }
            ri.idx = nidx;
            ri.update(len);
        };
        ret
    }

    fn split(this: Option<Self>, len: usize, index: usize) -> (Option<Self>, Option<Self>) {
        let Some(mut this) = this else {
            return (None, None);
        };
        if index == 0 {
            return (None, Some(this));
        }
        if index == len {
            return (Some(this), None);
        }
        this.setup(len);
        let node = this.as_mut();
        let idx = node.idx;
        if index > idx {
            let (l, r) = Self::split(
                std::mem::take(&mut node.right),
                len - 1 - idx,
                index - 1 - idx,
            );
            node.right = l;
            node.update(index);
            (Some(this), r)
        } else {
            let (l, r) = Self::split(std::mem::take(&mut node.left), idx, index);
            node.left = r;
            node.idx = idx - index;
            node.update(len - index);
            (l, Some(this))
        }
    }

    fn insert(this: &mut Option<Self>, len: usize, index: usize, item: M::T, rng: &mut Rng) {
        let Some(v) = this else {
            *this = Some(Self::new(item));
            return;
        };
        if rng.choose(len, 1) {
            v.setup(len);
            let node = v.as_mut();
            let idx = node.idx;
            if index > node.idx {
                Self::insert(&mut node.right, len - 1 - idx, index - 1 - idx, item, rng);
            } else {
                Self::insert(&mut node.left, idx, index, item, rng);
                node.idx += 1;
            }
            node.update(len + 1);
        } else {
            let (l, r) = Self::split(this.take(), len, index);
            let info = M::make_info(
                (l.as_ref().map(|v| &v.as_ref().info), index),
                &item,
                (r.as_ref().map(|v| &v.as_ref().info), len - index),
            );
            *this = Some(Self(Rc::new(UnsafeCell::new(Node {
                val: item,
                info,
                idx: index,
                rev: false,
                left: l,
                right: r,
            }))));
        }
    }

    fn remove(mut self, len: usize, index: usize, rng: &mut Rng) -> (M::T, Option<Self>) {
        use std::cmp::Ordering::*;
        self.setup(len);
        let node = self.as_mut();
        let idx = node.idx;
        match index.cmp(&idx) {
            Less => {
                let (v, o) = node.left.take().unwrap().remove(idx, index, rng);
                node.left = o;
                node.update(len - 1);
                (v, Some(self))
            }
            Equal => match (node.left.take(), node.right.take()) {
                (None, v) | (v, None) => (
                    Rc::try_unwrap(self.0)
                        .map_or_else(|v| unsafe { &*v.get() }.val.clone(), |v| v.into_inner().val),
                    v,
                ),
                (Some(l), Some(r)) => (
                    Rc::try_unwrap(self.0)
                        .map_or_else(|v| unsafe { &*v.get() }.val.clone(), |v| v.into_inner().val),
                    Some(Self::merge((l, idx), (r, len - 1 - idx), rng)),
                ),
            },
            Greater => {
                let (v, o) = node
                    .right
                    .take()
                    .unwrap()
                    .remove(len - 1 - idx, index - 1 - idx, rng);
                node.left = o;
                node.update(len - 1);
                (v, Some(self))
            }
        }
    }

    fn prod_left(&self, mut len: usize, mut index: usize) -> M::Prod {
        if index == 0 {
            return M::e();
        }
        let mut node = self.setup(len);
        if index == len {
            return M::info2prod(&node.info);
        }
        let mut ret = M::e();
        loop {
            if index < node.idx {
                len = node.idx;
                node = node.left.as_ref().unwrap().setup(len);
            } else if index == node.idx {
                return M::op(
                    ret,
                    M::info2prod(&node.left.as_ref().unwrap().setup(node.idx).info),
                );
            } else {
                if let Some(left) = node.left.as_ref() {
                    ret = M::op(ret, M::info2prod(&left.setup(node.idx).info));
                }
                ret = M::op(ret, M::val2prod(&node.val));
                if index == node.idx + 1 {
                    return ret;
                }
                index -= node.idx + 1;
                len -= node.idx + 1;
                node = node.right.as_ref().unwrap().setup(len);
            }
        }
    }

    fn prod_right(&self, mut len: usize, mut index: usize) -> M::Prod {
        if index == len {
            return M::e();
        }
        let mut node = self.setup(len);
        if index == 0 {
            return M::info2prod(&node.info);
        }
        let mut ret = M::e();
        loop {
            if index <= node.idx {
                if let Some(right) = node.right.as_ref() {
                    ret = M::op(M::info2prod(&right.setup(len - 1 - node.idx).info), ret);
                }
                ret = M::op(M::val2prod(&node.val), ret);
                if index == node.idx {
                    return ret;
                }
                len = node.idx;
                node = node.left.as_ref().unwrap().setup(len);
            } else if index == node.idx + 1 {
                return M::op(
                    M::info2prod(&node.right.as_ref().unwrap().setup(len - 1 - node.idx).info),
                    ret,
                );
            } else {
                index -= node.idx + 1;
                len -= node.idx + 1;
                node = node.right.as_ref().unwrap().setup(len);
            }
        }
    }

    fn apply_left(&mut self, len: usize, index: usize, lazy: &M::Lazy) {
        if index == 0 {
            return;
        }
        if index == len {
            M::apply_info(&mut self.as_mut().info, len, lazy);
            return;
        }
        self.setup(len);
        let node = self.as_mut();
        let idx = node.idx;
        if index <= idx {
            node.left.as_mut().unwrap().apply_left(idx, index, lazy);
        } else {
            if let Some(left) = node.left.as_mut() {
                M::apply_info(&mut left.as_mut().info, idx, lazy);
            }
            if index > idx {
                M::apply_val(&mut node.val, lazy);
                if index + 1 > idx {
                    node.right
                        .as_mut()
                        .unwrap()
                        .apply_left(len - 1 - idx, index - 1 - idx, lazy);
                }
            }
        }
        node.update(len);
    }

    fn apply_right(&mut self, len: usize, index: usize, lazy: &M::Lazy) {
        if index == 0 {
            M::apply_info(&mut self.as_mut().info, len, lazy);
            return;
        }
        if index == len {
            return;
        }
        self.setup(len);
        let node = self.as_mut();
        let idx = node.idx;
        if index > idx + 1 {
            node.right
                .as_mut()
                .unwrap()
                .apply_right(len - 1 - idx, index - 1 - idx, lazy);
        } else {
            if let Some(right) = node.right.as_mut() {
                M::apply_info(&mut right.as_mut().info, len - 1 - idx, lazy);
            }
            if index <= idx {
                M::apply_val(&mut node.val, lazy);
                if index < idx {
                    node.left.as_mut().unwrap().apply_right(idx, index, lazy);
                }
            }
        }
        node.update(len);
    }

    fn apply(&mut self, len: usize, left: usize, right: usize, lazy: &M::Lazy) {
        if left == 0 {
            self.apply_left(len, right, lazy);
            return;
        }
        if right == len {
            self.apply_right(len, left, lazy);
            return;
        }
        self.setup(len);
        let node = self.as_mut();
        let idx = node.idx;
        if left <= idx && right <= idx {
            node.left.as_mut().unwrap().apply(idx, left, right, lazy);
        } else if left > idx && right > idx {
            node.right.as_mut().unwrap().apply(
                len - 1 - idx,
                left - 1 - idx,
                right - 1 - idx,
                lazy,
            );
        } else {
            if let Some(l) = node.left.as_mut() {
                l.apply_right(idx, left, lazy);
            }
            if let Some(r) = node.right.as_mut() {
                r.apply_left(len - 1 - idx, right - 1 - idx, lazy);
            }
            M::apply_val(&mut node.val, lazy);
        }
        node.update(len);
    }
}

/// MasterTree. 大量の機能が詰め込まれた平衡二分木
pub struct MasterTree<M: MasterManager>(Option<NodeWrapper<M>>, usize, Rng);
impl<M: MasterManager> MasterTree<M> {
    /// 空の列のMasterTreeを作る
    ///
    /// # Time complexity
    ///
    /// - *O*(1)
    #[must_use]
    pub fn new() -> Self {
        Self(None, 0, Rng::new())
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

    /// 2つの列を混ぜた新しいMasterTreeを作る
    ///
    /// # Constraints
    ///
    /// - 元の列の長さの和が`usize::MAX`以下
    ///
    /// # Time complexity
    ///
    /// - *O*(log *N*)
    #[must_use]
    pub fn merge(self, rhs: Self) -> Self {
        let mut rng = self.2;
        let new_len = self
            .1
            .checked_add(rhs.1)
            .expect("MasterTree length overflow");
        Self(
            match (self.0, rhs.0) {
                (None, v) => v,
                (v, None) => v,
                (Some(l), Some(r)) => Some(NodeWrapper::merge((l, self.1), (r, rhs.1), &mut rng)),
            },
            new_len,
            rng,
        )
    }

    /// 1つの列を指定した場所で2つの列に分ける
    ///
    /// # Constraints
    ///
    /// - `index <= self.len()`
    ///
    /// # Time complexity
    ///
    /// - *O*(log *N*)
    #[must_use]
    pub fn split(self, index: usize) -> (Self, Self) {
        debug_assert!(index <= self.len());
        let mut rng = self.2;
        let (l, r) = NodeWrapper::split(self.0, self.1, index);
        (Self(l, index, rng), Self(r, self.1 - index, rng.make()))
    }

    /// 指定した場所に値を挿入する
    ///
    /// # Constraints
    ///
    /// - `index <= self.len()`
    /// - `self.len() < usize::MAX`
    ///
    /// # Time complexity
    ///
    /// - *O*(log *N*)
    pub fn insert(&mut self, index: usize, item: M::T) {
        debug_assert!(index <= self.len());
        let new_len = self.1.checked_add(1).expect("MasterTree length overflow");
        NodeWrapper::insert(&mut self.0, self.1, index, item, &mut self.2);
        self.1 = new_len;
    }

    /// 指定した場所の値を削除し, その値を返す
    ///
    /// # Constraints
    ///
    /// - `index < self.len()`
    ///
    /// # Time complexity
    ///
    /// - *O*(log *N*)
    pub fn remove(&mut self, index: usize) -> M::T {
        debug_assert!(index < self.len());
        let (r, o) = self.0.take().unwrap().remove(self.1, index, &mut self.2);
        self.1 -= 1;
        self.0 = o;
        r
    }

    /// 列を前後反転する
    ///
    /// # Time complexity
    ///
    /// - *O*(1)
    pub fn reverse(&mut self) {
        if let Some(node) = &mut self.0 {
            node.as_mut().rev ^= true;
        }
    }

    /// イテレータを返す
    pub fn iter(&self) -> impl ExactSizeIterator<Item = &M::T> + use<'_, M> {
        struct Iter<'a, M: MasterManager>(Vec<(&'a UnsafeCell<Node<M>>, usize)>, usize);
        impl<'a, M: MasterManager> Iterator for Iter<'a, M> {
            type Item = &'a M::T;
            fn next(&mut self) -> Option<&'a M::T> {
                let (node, mut len) = self.0.pop()?;
                let f = unsafe { &*node.get() };
                let ret = &f.val;
                let mut node = &f.right;
                len -= f.idx + 1;
                while let Some(ni) = node {
                    let f = ni.0.as_ref();
                    unsafe { &mut *f.get() }.setup(self.len());
                    self.0.push((f, len));
                    let f = unsafe { &*f.get() };
                    node = &f.left;
                    len = f.idx;
                }
                Some(ret)
            }

            fn size_hint(&self) -> (usize, Option<usize>) {
                (self.1, Some(self.1))
            }
        }
        impl<'a, M: MasterManager> ExactSizeIterator for Iter<'a, M> {
            fn len(&self) -> usize {
                self.1
            }
        }

        let mut node = &self.0;
        let mut len = self.len();
        let mut vec = vec![];
        while let Some(ni) = node {
            let f = ni.0.as_ref();
            unsafe { &mut *f.get() }.setup(self.len());
            vec.push((f, len));
            let f = unsafe { &*f.get() };
            node = &f.left;
            len = f.idx;
        }

        Iter(vec, self.len())
    }

    /// 指定した区間のモノイド総積を求める
    ///
    /// # Constraints
    ///
    /// - `range`は`0..self.len()`に含まれる区間である.
    ///
    /// # Time complexity
    ///
    /// - *O*(log *N*)
    pub fn prod(&self, range: impl std::ops::RangeBounds<usize>) -> M::Prod
    where
        M::Prod: std::fmt::Debug,
    {
        use std::ops::Bound::*;
        if cfg!(debug_assertions) {
            match range.start_bound() {
                Included(&left) => assert!(left <= self.len()),
                Excluded(&left) => assert!(left < self.len()),
                Unbounded => (),
            }
            match range.end_bound() {
                Included(&right) => assert!(right < self.len()),
                Excluded(&right) => assert!(right <= self.len()),
                Unbounded => (),
            }
        }
        let mut len = self.1;
        let mut left = match range.start_bound() {
            Included(&left) => left,
            Excluded(&left) => left + 1,
            Unbounded => 0,
        };
        let mut right = match range.end_bound() {
            Included(&right) => right + 1,
            Excluded(&right) => right,
            Unbounded => len,
        };
        debug_assert!(left <= right);
        if left == right {
            return M::e();
        }
        let mut node = self.0.as_ref().unwrap();
        loop {
            if left == 0 && right == len {
                return M::info2prod(&node.setup(len).info);
            }
            if left == 0 {
                return node.prod_left(len, right);
            }
            if right == len {
                return node.prod_right(len, left);
            }
            let f = node.setup(len);
            let idx = f.idx;
            if left <= idx && right <= idx {
                node = f.left.as_ref().unwrap();
                len = idx;
            } else if left > idx && right > idx {
                node = f.right.as_ref().unwrap();
                left -= idx + 1;
                right -= idx + 1;
                len -= idx + 1;
            } else {
                return M::op(
                    M::op(
                        f.left.as_ref().unwrap().prod_right(idx, left),
                        M::val2prod(&f.val),
                    ),
                    f.right
                        .as_ref()
                        .unwrap()
                        .prod_left(len - 1 - idx, right - 1 - idx),
                );
            }
        }
    }

    /// 指定した区間に作用素を適用する
    ///
    /// # Constraints
    ///
    /// - `range`は`0..self.len()`に含まれる区間である.
    ///
    /// # Time complexity
    ///
    /// - *O*(log *N*)
    pub fn apply(&mut self, range: impl std::ops::RangeBounds<usize>, lazy: &M::Lazy) {
        use std::ops::Bound::*;
        if cfg!(debug_assertions) {
            match range.start_bound() {
                Included(&left) => assert!(left <= self.len()),
                Excluded(&left) => assert!(left < self.len()),
                Unbounded => (),
            }
            match range.end_bound() {
                Included(&right) => assert!(right < self.len()),
                Excluded(&right) => assert!(right <= self.len()),
                Unbounded => (),
            }
        }
        let len = self.1;
        let left = match range.start_bound() {
            Included(&left) => left,
            Excluded(&left) => left + 1,
            Unbounded => 0,
        };
        let right = match range.end_bound() {
            Included(&right) => right + 1,
            Excluded(&right) => right,
            Unbounded => len,
        };
        debug_assert!(left <= right);
        if left == right {
            return;
        }
        self.0.as_mut().unwrap().apply(len, left, right, lazy);
    }
}

impl<M: MasterManager> Clone for MasterTree<M> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), self.1, self.2)
    }
}

impl<M: MasterManager> std::ops::Index<usize> for MasterTree<M> {
    type Output = M::T;
    fn index(&self, index: usize) -> &M::T {
        debug_assert!(index < self.len());
        let Some(v) = &self.0 else {
            unreachable!();
        };
        v.index(self.1, index)
    }
}

impl<M: MasterManager<T: std::fmt::Debug>> std::fmt::Debug for MasterTree<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn internal<M: MasterManager<T: std::fmt::Debug>>(
            v: &Option<NodeWrapper<M>>,
            len: usize,
            list: &mut std::fmt::DebugList<'_, '_>,
        ) {
            let Some(v) = v else {
                return;
            };
            v.setup(len);
            let v = v.as_ref();
            list.entry(&"(");
            internal(&v.left, v.idx, list);
            list.entry(&v.val);
            internal(&v.right, len - 1 - v.idx, list);
            list.entry(&")");
        }
        let mut list: std::fmt::DebugList<'_, '_> = f.debug_list();
        internal(&self.0, self.1, &mut list);
        list.finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_sum() {
        struct M;
        impl MasterManager for M {
            type T = i32;
            type Info = (i32, i32);
            type Prod = i32;
            type Lazy = i32;
            fn make_info(
                left: (Option<&(i32, i32)>, usize),
                mid: &i32,
                right: (Option<&(i32, i32)>, usize),
            ) -> (i32, i32) {
                (
                    left.0.map_or(0, |&(v, _)| v) + mid + right.0.map_or(0, |&(v, _)| v),
                    0,
                )
            }
            fn rev(_: &mut (i32, i32), _: usize) {}
            fn apply_info(info: &mut (i32, i32), len: usize, lazy: &i32) {
                info.0 += lazy * len as i32;
                info.1 += lazy;
            }
            fn apply_val(val: &mut i32, lazy: &i32) {
                *val += lazy;
            }
            fn propagate(
                info: &mut (i32, i32),
                left: (Option<&mut (i32, i32)>, usize),
                val: &mut Self::T,
                right: (Option<&mut (i32, i32)>, usize),
            ) {
                *val += info.1;
                if let Some((v, l)) = left.0 {
                    *v += info.1 * left.1 as i32;
                    *l += info.1;
                }
                if let Some((v, l)) = right.0 {
                    *v += info.1 * right.1 as i32;
                    *l += info.1;
                }
                info.1 = 0;
            }
            fn info2prod(info: &(i32, i32)) -> i32 {
                info.0
            }
            fn val2prod(val: &i32) -> i32 {
                *val
            }
            fn e() -> i32 {
                0
            }
            fn op(left: i32, right: i32) -> i32 {
                left + right
            }
        }

        let mut rng = Rng::new();
        // 乱数のシードを変えて1000回試す
        for _ in 0..1000 {
            let mut mt = MasterTree::<M>::new();
            mt.2 = rng.make();

            mt.insert(0, 3);
            mt.insert(0, 1);
            mt.insert(0, 4);
            mt.insert(0, 1);
            mt.insert(0, 5);
            mt.insert(0, 9);
            mt.insert(0, 2);
            mt.insert(0, 6);
            mt.insert(0, 5);
            mt.insert(0, 3);
            mt.insert(0, 5);
            mt.insert(0, 8);
            mt.insert(0, 9);
            mt.insert(0, 7);
            mt.insert(0, 9);
            mt.insert(0, 3);
            // [3, 9, 7, 9, 8, 5, 3, 5, 6, 2, 9, 5, 1, 4, 1, 3]
            mt.reverse();
            // [3, 1, 4, 1, 5 ,9, 2, 6, 5, 3, 5, 8, 9, 7, 9, 3]
            assert_eq!(mt.prod(4..12), 43);
            assert_eq!(mt.prod(5..), 66);
            mt.apply(7..12, &20);
            // [3, 1, 4, 1, 5, 9, 2, 26, 25, 23, 25, 28, 9, 7, 9, 3]
            mt.apply(.., &40);
            // [43, 41, 44, 41, 45, 49, 42, 66, 65, 63, 65, 68, 49, 47, 49, 43]
            // [43, 41, 44, 41, 45, 49, 42, 46, 45, 43, 45, 48, 49, 47, 49, 43]
            assert_eq!(mt.prod(4..12), 463);
            assert_eq!(mt.prod(5..), 606);

            // mt2 = [43, 41, 44, 41, 45, 49, 42, 66, 65]
            // mt3 = [63, 65, 68, 49, 47, 49, 43]
            let (mt2, mt3) = mt.split(9);
            assert_eq!(mt2.prod(3..6), 135);
            assert_eq!(mt3.prod(..5), 292);

            // mt = [63, 65, 68, 49, 47, 49, 43, 43, 41, 44, 41, 45, 49, 42, 66, 65]
            mt = mt3.merge(mt2);
            let hand_calculation = [
                63, 65, 68, 49, 47, 49, 43, 43, 41, 44, 41, 45, 49, 42, 66, 65,
            ];
            for (i, &v) in hand_calculation.iter().enumerate() {
                assert_eq!(mt[i], v);
            }
        }
    }
}
