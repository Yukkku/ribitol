use std::{cell::Cell, num::NonZero, rc::Rc};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Node {
    Root(NonZero<usize>),
    Sub(NonZero<usize>, usize),
}

/// PersistentUnionFindの参照.
/// union, sizeはできないが, findは行うことが出来る
#[derive(Clone)]
pub struct PersistentUFRef(Rc<[Cell<Node>]>, NonZero<usize>);
impl PersistentUFRef {
    /// 要素数を返す
    ///
    /// # Time complexity
    ///
    /// - *O*(1)
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// `index`が属するグループの代表を返す
    ///
    /// # Constraints
    ///
    /// - `index < self.len()`
    ///
    /// # Time complexity
    ///
    /// - *O*(log *N*)
    #[must_use]
    pub fn find(&self, mut index: usize) -> usize {
        let sl = self.0.as_ref();
        loop {
            match sl[index].get() {
                Node::Root(_) => return index,
                Node::Sub(t, j) => {
                    if t >= self.1 {
                        return index;
                    } else {
                        index = j;
                    }
                }
            }
        }
    }
}

/// 部分永続UnionFind
pub struct PersistentUnionFind(Rc<[Cell<Node>]>, Cell<NonZero<usize>>);

impl PersistentUnionFind {
    /// 要素数`n`の部分永続UnionFindを作成する
    ///
    /// # Time complexity
    ///
    /// - *O*(*n*)
    #[must_use]
    pub fn new(n: usize) -> Self {
        Self(
            std::iter::repeat_n(const { Cell::new(Node::Root(NonZero::new(1).unwrap())) }, n)
                .collect(),
            const { Cell::new(NonZero::new(1).unwrap()) },
        )
    }

    /// 要素数を返す
    ///
    /// # Time complexity
    ///
    /// - *O*(1)
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[must_use]
    fn find_size(&self, mut index: usize) -> (usize, NonZero<usize>) {
        let sl = self.0.as_ref();
        loop {
            match sl[index].get() {
                Node::Root(s) => return (index, s),
                Node::Sub(_, j) => index = j,
            }
        }
    }

    /// `index`が属するグループの代表を返す
    ///
    /// # Constraints
    ///
    /// - `index < self.len()`
    ///
    /// # Time complexity
    ///
    /// - *O*(log *N*)
    #[must_use]
    pub fn find(&self, index: usize) -> usize {
        self.find_size(index).0
    }

    /// `index`が属するグループの要素数を返す
    ///
    /// # Constraints
    ///
    /// - `index < self.len()`
    ///
    /// # Time complexity
    ///
    /// - *O*(log *N*)
    #[must_use]
    pub fn size(&self, index: usize) -> NonZero<usize> {
        self.find_size(index).1
    }

    /// `a`が属するグループと`b`が属するグループをマージして, 1つの大きなグループにする
    ///
    /// 元から同じグループに属していた場合は, 何もしない
    ///
    /// # Constraints
    ///
    /// - `a < self.len()`
    /// - `b < self.len()`
    ///
    /// # Time complexity
    ///
    /// - *O*(log *N*)
    pub fn union(&mut self, a: usize, b: usize) {
        let (a, sa) = self.find_size(a);
        let (b, sb) = self.find_size(b);
        if a == b {
            return;
        }
        if sa > sb {
            self.0[a].set(Node::Root(sa.checked_add(sb.get()).unwrap()));
            self.0[b].set(Node::Sub(self.1.get(), a));
        } else {
            self.0[b].set(Node::Root(sa.checked_add(sb.get()).unwrap()));
            self.0[a].set(Node::Sub(self.1.get(), b));
        }
    }

    /// `PersistentUFRef`を作成する
    ///
    /// # Time complexity
    ///
    /// - *O*(1)
    pub fn get_ref(&self) -> PersistentUFRef {
        let t = self.1.get().checked_add(1).unwrap();
        self.1.set(t);
        PersistentUFRef(self.0.clone(), t)
    }
}

impl Clone for PersistentUnionFind {
    fn clone(&self) -> Self {
        Self(
            self.0
                .iter()
                .map(|v| {
                    Cell::new(match v.get() {
                        Node::Root(s) => Node::Root(s),
                        Node::Sub(_, i) => Node::Sub(const { NonZero::new(1).unwrap() }, i),
                    })
                })
                .collect(),
            const { Cell::new(NonZero::new(1).unwrap()) },
        )
    }
}
