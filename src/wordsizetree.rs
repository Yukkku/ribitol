fn mask_split(i: usize) -> (usize, usize) {
    (i >> 6, i & 63)
}

trait MSet {
    unsafe fn add(&mut self, index: usize);
    unsafe fn delete(&mut self, index: usize);
    unsafe fn has(&self, index: usize) -> bool;
    unsafe fn is_empty(&self) -> bool;
    unsafe fn min(&self) -> usize;
}
struct WordSet(u64);
impl MSet for WordSet {
    #[inline(always)]
    unsafe fn add(&mut self, index: usize) {
        std::hint::assert_unchecked(index < 64);
        self.0 |= 1 << index;
    }
    #[inline(always)]
    unsafe fn delete(&mut self, index: usize) {
        std::hint::assert_unchecked(index < 64);
        self.0 &= !(1 << index);
    }
    #[inline(always)]
    unsafe fn has(&self, index: usize) -> bool {
        std::hint::assert_unchecked(index < 64);
        (self.0 >> index) & 1 == 1
    }
    #[inline(always)]
    unsafe fn is_empty(&self) -> bool {
        self.0 == 0
    }
    #[inline(always)]
    unsafe fn min(&self) -> usize {
        std::hint::assert_unchecked(!self.is_empty());
        self.0.trailing_zeros() as usize
    }
}

struct Layer<const N: usize, T: MSet>([WordSet; N], T);
impl<const N: usize, T: MSet> MSet for Layer<N, T> {
    #[inline(always)]
    unsafe fn add(&mut self, index: usize) {
        std::hint::assert_unchecked(index < (N << 6));
        let (i, j) = mask_split(index);
        self.0[i].add(j);
        self.1.add(i);
    }
    #[inline(always)]
    unsafe fn delete(&mut self, index: usize) {
        std::hint::assert_unchecked(index < (N << 6));
        let (i, j) = mask_split(index);
        self.0[i].delete(j);
        if self.0[i].is_empty() {
            self.1.delete(i);
        }
    }
    #[inline(always)]
    unsafe fn has(&self, index: usize) -> bool {
        std::hint::assert_unchecked(index < (N << 6));
        let (i, j) = mask_split(index);
        self.0[i].has(j)
    }
    #[inline(always)]
    unsafe fn is_empty(&self) -> bool {
        self.1.is_empty()
    }
    #[inline(always)]
    unsafe fn min(&self) -> usize {
        std::hint::assert_unchecked(!self.is_empty());
        let v = self.1.min();
        std::hint::assert_unchecked(v < N);
        self.0[v].min() | (v << 6)
    }
}

// 0以上262144未満の整数が入る64分木
pub struct WordSizeTree18(Layer<4096, Layer<64, WordSet>>);
impl WordSizeTree18 {
    pub const LEN: usize = 262144;

    /// 新しい空のWordSizeTree18を作成する.
    #[must_use]
    pub fn new() -> Box<Self> {
        unsafe {
            let mut b = Box::<Self>::new_uninit();
            b.as_mut_ptr().write_bytes(0, 1);
            b.assume_init()
        }
    }

    /// 集合に要素を追加する
    ///
    /// # Constraints
    ///
    /// - `index < 262144`
    pub fn add(&mut self, index: usize) {
        assert!(index < Self::LEN);
        unsafe { self.0.add(index) };
    }

    /// 集合から要素を削除する
    ///
    /// # Constraints
    ///
    /// - `index < 262144`
    pub fn delete(&mut self, index: usize) {
        assert!(index < Self::LEN);
        unsafe { self.0.delete(index) };
    }

    /// 集合が要素を持つか判定する
    ///
    /// # Constraints
    ///
    /// - `index < 262144`
    #[must_use]
    pub fn has(&self, index: usize) -> bool {
        assert!(index < Self::LEN);
        unsafe { self.0.has(index) }
    }

    /// 集合が空かどうか判定する
    ///
    /// # Time complexity
    ///
    /// - *O*(1)
    #[must_use]
    pub fn is_empty(&self) -> bool {
        unsafe { self.0.is_empty() }
    }

    /// 集合の最小値を探す
    /// 集合が空だった場合はNoneを返す
    #[must_use]
    pub fn min(&self) -> Option<usize> {
        (!self.is_empty()).then(|| unsafe { self.0.min() })
    }
}

// 0以上16777216未満の整数が入る64分木
pub struct WordSizeTree24(Layer<262144, Layer<4096, Layer<64, WordSet>>>);
impl WordSizeTree24 {
    pub const LEN: usize = 16777216;

    /// 新しい空のWordSizeTree24を作成する.
    #[must_use]
    pub fn new() -> Box<Self> {
        unsafe {
            let mut b = Box::<Self>::new_uninit();
            b.as_mut_ptr().write_bytes(0, 1);
            b.assume_init()
        }
    }

    /// 集合に要素を追加する
    ///
    /// # Constraints
    ///
    /// - `index < 16777216`
    pub fn add(&mut self, index: usize) {
        assert!(index < Self::LEN);
        unsafe { self.0.add(index) };
    }

    /// 集合から要素を削除する
    ///
    /// # Constraints
    ///
    /// - `index < 16777216`
    pub fn delete(&mut self, index: usize) {
        assert!(index < Self::LEN);
        unsafe { self.0.delete(index) };
    }

    /// 集合が要素を持つか判定する
    ///
    /// # Constraints
    ///
    /// - `index < 16777216`
    #[must_use]
    pub fn has(&self, index: usize) -> bool {
        assert!(index < Self::LEN);
        unsafe { self.0.has(index) }
    }

    /// 集合が空かどうか判定する
    ///
    /// # Time complexity
    ///
    /// - *O*(1)
    #[must_use]
    pub fn is_empty(&self) -> bool {
        unsafe { self.0.is_empty() }
    }

    /// 集合の最小値を探す
    /// 集合が空だった場合はNoneを返す
    #[must_use]
    pub fn min(&self) -> Option<usize> {
        (!self.is_empty()).then(|| unsafe { self.0.min() })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wst18() {
        let mut v = WordSizeTree18::new();
        assert!(v.is_empty());
        assert!(!v.has(31415));
        assert!(!v.has(92653));
        assert!(!v.has(58979));
        v.add(92653);
        assert!(!v.is_empty());
        assert!(!v.has(31415));
        assert!(v.has(92653));
        assert!(!v.has(58979));

        v.add(32384);
        v.add(62643);
        v.add(38327);

        assert_eq!(v.min(), Some(32384));
        v.delete(32384);
        v.delete(116);
        assert_eq!(v.min(), Some(38327));
        v.delete(38327);
        assert_eq!(v.min(), Some(62643));
        v.delete(62643);
        assert_eq!(v.min(), Some(92653));
        v.delete(92653);
        assert_eq!(v.min(), None);
    }

    #[test]
    fn wst24() {
        let mut v = WordSizeTree24::new();
        assert!(v.is_empty());
        assert!(!v.has(3141592));
        assert!(!v.has(6535897));
        assert!(!v.has(9323846));
        v.add(6535897);
        assert!(!v.is_empty());
        assert!(!v.has(3141592));
        assert!(v.has(6535897));
        assert!(!v.has(9323846));

        v.add(2643383);
        v.add(2795028);
        v.add(8419794);

        assert_eq!(v.min(), Some(2643383));
        v.delete(2643383);
        v.delete(2718);
        assert_eq!(v.min(), Some(2795028));
        v.delete(2795028);
        assert_eq!(v.min(), Some(6535897));
        v.delete(6535897);
        assert_eq!(v.min(), Some(8419794));
        v.delete(8419794);
        assert_eq!(v.min(), None);
    }
}
