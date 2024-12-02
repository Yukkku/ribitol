/// 素集合データ構造
///
/// 幾つかのグループのマージとグループの所属判定を高速に行える.
#[derive(Clone)]
pub struct UnionFind(Box<[isize]>, usize);

impl UnionFind {
    /// `n`個の要素があり, それぞれ別のグループに属しているUnionFindを作る.
    ///
    /// # Time complexity
    ///
    /// - *O*(*n*)
    #[must_use]
    pub fn new(n: usize) -> Self {
        Self(vec![-1; n].into_boxed_slice(), n)
    }

    /// 要素の総数を返す.
    ///
    /// # Time complexity
    ///
    /// - *O*(1)
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// 要素の連結成分数を返す.
    ///
    /// # Time complexity
    ///
    /// - *O*(1)
    #[must_use]
    pub fn count(&self) -> usize {
        self.1
    }

    /// 要素`a`が属するグループと要素`b`が属するグループを1つのグループにマージし, 新しいグループの代表を返す.
    /// 最初から同じグループに属していた場合は, 何もせずにそのグループの代表を返す.
    ///
    /// # Time complexity
    ///
    /// - *O*(α(*n*))
    pub fn union(&mut self, a: usize, b: usize) -> usize {
        debug_assert!(a < self.len());
        debug_assert!(b < self.len());
        let a = self.find(a);
        let b = self.find(b);
        if a == b {
            return a;
        }
        self.1 -= 1;
        if a < b {
            self.0[a] += self.0[b];
            self.0[b] = a as isize;
            a
        } else {
            self.0[b] += self.0[a];
            self.0[a] = b as isize;
            b
        }
    }

    /// 要素`a`が属するグループの代表を返す.
    ///
    /// # Time complexity
    ///
    /// - *O*(α(*n*))
    #[must_use]
    pub fn find(&mut self, mut a: usize) -> usize {
        debug_assert!(a < self.len());
        let mut b = a;
        while self.0[b] >= 0 {
            b = self.0[b] as usize;
        }
        while a != b {
            let tmp = self.0[a];
            self.0[a] = b as isize;
            a = tmp as usize;
        }
        a
    }

    /// 要素`a`が属するグループの代表を返す.
    ///
    /// # Time complexity
    ///
    /// - *O*(log(*n*))
    #[must_use]
    pub fn find_imu(&self, mut a: usize) -> usize {
        debug_assert!(a < self.len());
        while self.0[a] >= 0 {
            a = self.0[a] as usize;
        }
        a
    }

    /// 要素`a`が属するグループの要素数を返す.
    ///
    /// # Time complexity
    ///
    /// - *O*(α(*n*))
    #[must_use]
    pub fn size(&mut self, a: usize) -> usize {
        debug_assert!(a < self.len());
        -self.0[self.find(a)] as usize
    }

    /// 要素`a`が属するグループの要素数を返す.
    ///
    /// # Time complexity
    ///
    /// - *O*(log(*n*))
    #[must_use]
    pub fn size_imu(&self, a: usize) -> usize {
        debug_assert!(a < self.len());
        -self.0[self.find_imu(a)] as usize
    }

    /// 要素`a`, `b`が同じグループに属するか判定する.
    ///
    /// # Time complexity
    ///
    /// - *O*(α(*n*))
    pub fn same(&mut self, a: usize, b: usize) -> bool {
        debug_assert!(a < self.len());
        debug_assert!(b < self.len());
        self.find(a) == self.find(b)
    }

    /// 要素`a`, `b`が同じグループに属するか判定する.
    ///
    /// # Time complexity
    ///
    /// - *O*(log(*n*))
    pub fn same_imu(&self, a: usize, b: usize) -> bool {
        debug_assert!(a < self.len());
        debug_assert!(b < self.len());
        self.find_imu(a) == self.find_imu(b)
    }
}

impl std::fmt::Debug for UnionFind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        struct VecSet(Vec<usize>);
        impl std::fmt::Debug for VecSet {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_set().entries(&self.0).finish()
            }
        }

        let mut q = Vec::with_capacity(self.len());
        for &v in self.0.iter() {
            if v < 0 {
                let mut w = Vec::with_capacity(-v as usize);
                w.push(q.len());
                q.push(w);
            } else {
                q.push(vec![]);
            }
        }
        for i in 0..self.len() {
            let g = self.find_imu(i);
            if i != g {
                q[g].push(i);
            }
        }
        f.debug_set()
            .entries(
                q.into_iter()
                    .filter_map(|v| if v.is_empty() { None } else { Some(VecSet(v)) }),
            )
            .finish()
    }
}

impl Default for UnionFind {
    fn default() -> Self {
        Self(vec![].into(), 0)
    }
}
