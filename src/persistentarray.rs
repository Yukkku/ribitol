use std::{num::NonZero, rc::Rc};

#[derive(Clone)]
enum RawPersitentArray<T> {
    Value(T),
    Relay(NonZero<u8>, (Rc<Self>, Rc<Self>)),
}
use RawPersitentArray::*;

impl<T> RawPersitentArray<T> {
    fn level(&self) -> u8 {
        match self {
            Value(_) => 0,
            Relay(n, _) => n.get(),
        }
    }

    fn get_relay_mut(this: &mut Rc<Self>) -> (&mut NonZero<u8>, &mut (Rc<Self>, Rc<Self>)) {
        if Rc::weak_count(this) == 0 && Rc::strong_count(this) == 1 {
            if let Some(Relay(level, children)) = Rc::get_mut(this) {
                return (level, children);
            }
            unreachable!();
        } else {
            let (level, children) = {
                let Relay(level, children) = this.as_ref() else {
                    unreachable!();
                };
                (*level, children.clone())
            };
            *this = Rc::new(Relay(level, children));
            let Some(Relay(level, children)) = Rc::get_mut(this) else {
                unreachable!();
            };
            (level, children)
        }
    }
}

/// 永続配列
#[derive(Default)]
pub struct PersistentArray<T>(Option<Rc<RawPersitentArray<T>>>);

impl<T> PersistentArray<T> {
    /// 空の永続配列を作る
    ///
    /// # Time complexity
    ///
    /// - *O*(1)
    #[must_use]
    pub fn new() -> Self {
        Self(None)
    }

    /// 永続配列が空か判定する
    ///
    /// # Time complexity
    ///
    /// - *O*(1)
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_none()
    }

    /// 永続配列の長さを返す
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
                Relay(level, (_, child)) => {
                    len += 1 << (level.get() - 1);
                    node = child;
                }
            }
        }
    }

    fn push_with_len(&mut self, mut len: usize, item: T) {
        let item = Rc::new(Value(item));
        let Some(node) = &mut self.0 else {
            self.0 = Some(item);
            return;
        };
        let mut node = node;
        while !len.is_power_of_two() {
            let (level, (_, child)) = RawPersitentArray::get_relay_mut(node);
            len -= 1 << (level.get() - 1);
            node = child;
        }
        *node = Rc::new(Relay(
            NonZero::new(node.level() + 1).unwrap(),
            (node.clone(), item),
        ));
    }

    /// 末尾に要素を追加する
    ///
    /// # Time complexity
    ///
    /// - *O*(log *N*)
    pub fn push(&mut self, item: T) {
        self.push_with_len(self.len(), item);
    }

    /// 末尾の要素を削除し, その要素を返す
    /// 空だった場合は`None`を返す
    ///
    /// # Time complexity
    ///
    /// - *O*(log *N*)
    pub fn pop(&mut self) -> Option<T>
    where
        T: Clone,
    {
        let node = self.0.as_mut()?;
        if matches!(node.as_ref(), Value(_)) {
            let Some(Some(Value(v))) = std::mem::take(&mut self.0).map(Rc::into_inner) else {
                unreachable!()
            };
            return Some(v);
        }
        let mut node = node;
        loop {
            let Relay(_, (l, r)) = node.as_ref() else {
                unreachable!();
            };
            if let Value(v) = r.as_ref() {
                let ret = Some(v.clone());
                *node = l.clone();
                return ret;
            } else {
                node = &mut RawPersitentArray::get_relay_mut(node).1 .1;
            }
        }
    }
}

impl<T> Clone for PersistentArray<T> {
    fn clone(&self) -> Self {
        match &self.0 {
            Some(rc) => Self(Some(rc.clone())),
            None => Self(None),
        }
    }
}

impl<T> std::ops::Index<usize> for PersistentArray<T> {
    type Output = T;

    fn index(&self, mut index: usize) -> &T {
        let Some(node) = &self.0 else {
            panic!("PersistentArray is empty.")
        };
        let mut node = node.as_ref();
        loop {
            match node {
                Value(v) => {
                    debug_assert!(index == 0);
                    return v;
                }
                Relay(level, children) => {
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
impl<T: Clone> std::ops::IndexMut<usize> for PersistentArray<T> {
    fn index_mut(&mut self, mut index: usize) -> &mut T {
        let Some(node) = &mut self.0 else {
            panic!("PersistentArray is empty.")
        };
        let mut node = Rc::make_mut(node);
        loop {
            match node {
                Value(v) => {
                    debug_assert_eq!(index, 0);
                    return v;
                }
                Relay(level, children) => {
                    let m = 1 << (level.get() - 1);
                    if index < m {
                        node = Rc::make_mut(&mut children.0);
                    } else {
                        index -= m;
                        node = Rc::make_mut(&mut children.1);
                    }
                }
            }
        }
    }
}

impl<T> FromIterator<T> for PersistentArray<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut r = Self(None);
        for (len, v) in iter.into_iter().enumerate() {
            r.push_with_len(len, v);
        }
        r
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for PersistentArray<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn internal<T: std::fmt::Debug>(
            v: &RawPersitentArray<T>,
            list: &mut std::fmt::DebugList<'_, '_>,
        ) {
            match v {
                Value(v) => {
                    list.entry(v);
                }
                Relay(_, (l, r)) => {
                    internal(l, list);
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

impl<T> Extend<T> for PersistentArray<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        let mut len = self.len();
        for v in iter {
            self.push_with_len(len, v);
            len += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let a = (0..10).collect::<PersistentArray<_>>();
        assert_eq!(a.len(), 10);
        assert_eq!(a[3], 3);
        let mut b = a.clone();
        assert_eq!(b.len(), 10);
        assert_eq!(b[3], 3);
        b[3] = 7;
        assert_eq!(b[3], 7);
        assert_eq!(a[3], 3);
    }
}
