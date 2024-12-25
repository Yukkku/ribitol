use std::mem::ManuallyDrop;

union RawBinaryTree<T> {
    trunc: ManuallyDrop<Box<(usize, Self, Self)>>,
    leaf: ManuallyDrop<Box<T>>,
    none: (),
}

impl<T> Default for RawBinaryTree<T> {
    fn default() -> Self {
        Self { none: () }
    }
}

impl<T> RawBinaryTree<T> {
    unsafe fn index(&self, mut len: usize, mut index: usize) -> &T {
        let mut node = self;
        loop {
            if len == 1 {
                return &node.leaf;
            }
            let trunc = node.trunc.as_ref();
            if index < trunc.0 {
                len = trunc.0;
                node = &trunc.1;
            } else {
                index -= trunc.0;
                len -= trunc.0;
                node = &trunc.2;
            }
        }
    }
    unsafe fn index_mut(&mut self, mut len: usize, mut index: usize) -> &mut T {
        let mut node = self;
        loop {
            if len == 1 {
                return &mut node.leaf;
            }
            let trunc = node.trunc.as_mut();
            if index < trunc.0 {
                len = trunc.0;
                node = &mut trunc.1;
            } else {
                index -= trunc.0;
                len -= trunc.0;
                node = &mut trunc.2;
            }
        }
    }

    unsafe fn insert(&mut self, len: usize, index: usize, item: Box<T>) {
        if len == 1 {
            let new = Self {
                leaf: ManuallyDrop::new(item),
            };
            let old = std::mem::take(self);
            *self = Self {
                trunc: ManuallyDrop::new(Box::new(if index == 0 {
                    (1, new, old)
                } else {
                    (1, old, new)
                })),
            };
            return;
        }
        let trunc = self.trunc.as_mut();
        let lsize = trunc.0;
        if index <= lsize {
            trunc.1.insert(lsize, index, item);
            trunc.0 += 1;
        } else {
            trunc.2.insert(len - lsize, index - lsize, item);
        }
        self.rebalance(len + 1);
    }

    unsafe fn remove(&mut self, len: usize, index: usize) -> Box<T> {
        let lsize = self.trunc.0;
        let r;
        if index < lsize {
            if lsize == 1 {
                let (_, left, right) = *ManuallyDrop::into_inner(std::mem::take(self).trunc);
                *self = right;
                return ManuallyDrop::into_inner(left.leaf);
            }
            r = self.trunc.1.remove(lsize, index);
            self.trunc.0 -= 1;
        } else {
            if lsize == len - 1 {
                let (_, left, right) = *ManuallyDrop::into_inner(std::mem::take(self).trunc);
                *self = left;
                return ManuallyDrop::into_inner(right.leaf);
            }
            r = self.trunc.2.remove(len - lsize, index - lsize);
        }
        self.rebalance(len - 1);
        r
    }

    unsafe fn drop(&mut self, len: usize) {
        if len == 1 {
            ManuallyDrop::drop(&mut self.leaf)
        } else {
            let trunc = self.trunc.as_mut();
            let l_len = trunc.0;
            trunc.1.drop(l_len);
            trunc.2.drop(len - l_len);
            ManuallyDrop::drop(&mut self.trunc)
        }
    }

    unsafe fn rot_l(&mut self, _len: usize) {
        let root = (*self.trunc).as_mut();
        let lsize = root.0;
        std::mem::swap(&mut root.1, &mut root.2);
        let sub = (*root.1.trunc).as_mut();
        let new_lsize = lsize + sub.0;
        sub.0 = lsize;
        root.0 = new_lsize;
        std::mem::swap(&mut sub.1, &mut sub.2);
        std::mem::swap(&mut sub.1, &mut root.2);
    }
    unsafe fn rot_r(&mut self, _len: usize) {
        let root = (*self.trunc).as_mut();
        let lsize = root.0;
        std::mem::swap(&mut root.1, &mut root.2);
        let sub = (*root.2.trunc).as_mut();
        let new_lsize = sub.0;
        sub.0 = lsize - sub.0;
        root.0 = new_lsize;
        std::mem::swap(&mut sub.1, &mut sub.2);
        std::mem::swap(&mut sub.2, &mut root.1);
    }

    unsafe fn rebalance(&mut self, len: usize) {
        let l = self.trunc.0;
        let r = len - l;
        if l >= 3 * r {
            let ltree = &mut self.trunc.1;
            let ll = ltree.trunc.0;
            if ll <= r {
                ltree.rot_l(l);
            }
            self.rot_r(len);
        } else if r >= 3 * l {
            let rtree = &mut self.trunc.2;
            let rr = r - rtree.trunc.0;
            if rr <= l {
                rtree.rot_r(r);
            }
            self.rot_l(len);
        }
    }

    pub unsafe fn debug(&self, len: usize, f: &mut std::fmt::DebugList<'_, '_>)
    where
        T: std::fmt::Debug,
    {
        if len == 1 {
            f.entry(self.leaf.as_ref());
        } else {
            let trunc = self.trunc.as_ref();
            let l_len = trunc.0;
            trunc.1.debug(l_len, f);
            trunc.2.debug(len - l_len, f);
        }
    }
}

/// 挿入/削除が高速な配列としての平衡二分木
pub struct BinaryTree<T> {
    len: usize,
    root: RawBinaryTree<T>,
}

impl<T> BinaryTree<T> {
    /// 空の平衡二分木を構築する
    ///
    /// # Time complexity
    ///
    /// - *O*(1)
    #[must_use]
    pub fn new() -> Self {
        Self {
            len: 0,
            root: RawBinaryTree { none: () },
        }
    }

    /// 平衡二分木の要素の数を返す
    ///
    /// # Time complexity
    ///
    /// - *O*(1)
    #[must_use]
    pub fn len(&self) -> usize {
        self.len
    }

    /// 平衡二分木が空かどうか判定する
    ///
    /// # Time complexity
    ///
    /// - *O*(1)
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// 値を指定の場所に挿入する
    ///
    /// # Constraints
    ///
    /// - `index <= self.len()`
    ///
    /// # Time complexity
    ///
    /// - *O*(log *n*)
    pub fn insert(&mut self, index: usize, item: T) {
        assert!(index <= self.len());
        if self.len == 0 {
            self.root = RawBinaryTree {
                leaf: ManuallyDrop::new(Box::new(item)),
            }
        } else {
            unsafe {
                self.root.insert(self.len, index, Box::new(item));
            }
        }
        self.len += 1;
    }

    /// 指定した位置の値を削除して, その値を返す
    ///
    /// # Constraints
    ///
    /// - `index < self.len()`
    ///
    /// # Time complexity
    ///
    /// - *O*(log *n*)
    pub fn remove(&mut self, index: usize) -> T {
        assert!(index < self.len());
        *unsafe {
            if self.len == 1 {
                self.len = 0;
                ManuallyDrop::into_inner(
                    std::mem::replace(&mut self.root, RawBinaryTree { none: () }).leaf,
                )
            } else {
                self.len -= 1;
                self.root.remove(self.len + 1, index)
            }
        }
    }
}

impl<T> std::ops::Index<usize> for BinaryTree<T> {
    type Output = T;

    fn index(&self, index: usize) -> &T {
        assert!(index < self.len());
        unsafe { self.root.index(self.len, index) }
    }
}
impl<T> std::ops::IndexMut<usize> for BinaryTree<T> {
    fn index_mut(&mut self, index: usize) -> &mut T {
        assert!(index < self.len());
        unsafe { self.root.index_mut(self.len, index) }
    }
}

impl<T> Drop for BinaryTree<T> {
    fn drop(&mut self) {
        if self.len >= 1 {
            unsafe {
                self.root.drop(self.len);
            }
        }
    }
}

impl<T> Default for BinaryTree<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for BinaryTree<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut list = f.debug_list();
        if !self.is_empty() {
            unsafe {
                self.root.debug(self.len, &mut list);
            }
        }
        list.finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let mut bs = BinaryTree::<i32>::new();
        bs.insert(0, 1);
        bs.insert(0, 2);
        bs.insert(2, 3);
        bs.insert(1, 4);
        bs.insert(3, 5);
        // [2, 4, 1, 5, 3]
        assert_eq!(bs.len(), 5);
        assert_eq!(bs.remove(2), 1);
        assert_eq!(bs.remove(3), 3);
        assert_eq!(bs.remove(0), 2);
        assert_eq!(bs.remove(1), 5);
        assert_eq!(bs.remove(0), 4);
        assert_eq!(bs.len(), 0);
    }
}
