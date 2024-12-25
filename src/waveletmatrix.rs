struct Dict {
    raw: Box<[u64]>,
    big: Box<[u64]>,
    small: Box<[u16]>,
    one_count: usize,
}
impl Dict {
    fn new(raw: Box<[u64]>) -> Self {
        let mut small = Vec::with_capacity(raw.len() + 1);
        let mut big = Vec::with_capacity((raw.len() >> 10) + 1);
        let mut big_count = 0;
        let mut small_count = 0;
        big.push(0);
        small.push(0);
        for (i, &v) in raw.iter().enumerate() {
            let v = v.count_ones() as u64;
            big_count += v;
            small_count += v;
            if i & 1023 == 1023 {
                big.push(big_count);
                small_count = 0;
            }
            small.push(small_count as u16);
        }

        Self {
            raw,
            big: big.into_boxed_slice(),
            small: small.into_boxed_slice(),
            one_count: big_count as usize,
        }
    }

    fn len(&self) -> usize {
        self.raw.len() * 64
    }

    unsafe fn get_unchecked(&self, index: usize) -> bool {
        (self.raw.get_unchecked(index >> 6) >> (index & 63)) & 1 == 1
    }
    fn get(&self, index: usize) -> bool {
        assert!(index < self.len());
        unsafe { self.get_unchecked(index) }
    }

    unsafe fn rank_unchecked(&self, index: usize) -> usize {
        *self.big.get_unchecked(index >> 16) as usize
            + *self.small.get_unchecked(index >> 6) as usize
            + (self.raw.get_unchecked(index >> 6) & !(!0 << (index & 63))).count_ones() as usize
    }

    unsafe fn select_unchecked(&self, count: usize) -> usize {
        let x = match self.big.binary_search(&(count as u64)) {
            Ok(i) => i,
            Err(i) => i - 1,
        };
        let count = (count - *self.big.get_unchecked(x) as usize) as u16;
        let x = x << 10;
        let x = x | match self
            .small
            .get_unchecked(x..self.small.len().min(x + 1024))
            .binary_search(&count)
        {
            Ok(i) => i,
            Err(i) => i - 1,
        };
        let count = (count - *self.small.get_unchecked(x)) as u8;
        let v = *self.raw.get_unchecked(x);
        let mut k = !0u64;
        let mut r = x << 6;
        #[inline(always)]
        fn internal(k: &mut u64, r: &mut usize, s: usize, v: u64, count: u8) {
            let temp = *k << s;
            if (v & !temp).count_ones() as u8 <= count {
                *k = temp;
                *r |= s;
            }
        }
        internal(&mut k, &mut r, 1 << 5, v, count);
        internal(&mut k, &mut r, 1 << 4, v, count);
        internal(&mut k, &mut r, 1 << 3, v, count);
        internal(&mut k, &mut r, 1 << 2, v, count);
        internal(&mut k, &mut r, 1 << 1, v, count);
        internal(&mut k, &mut r, 1 << 0, v, count);
        r
    }
    fn select(&self, count: usize) -> Option<usize> {
        if count >= self.one_count {
            None
        } else {
            Some(unsafe { self.select_unchecked(count) })
        }
    }

    unsafe fn select_zero_unchecked(&self, count: usize) -> usize {
        let x = {
            let mut size = self.big.len();
            let mut x = 0;
            while size > 1 {
                let half = size / 2;
                let mid = x + half;
                let val = unsafe { *self.big.get_unchecked(mid) } as usize;
                if (mid << 16) - val <= count {
                    x = mid;
                    size -= half;
                } else {
                    size = half;
                }
            }
            x
        };
        let count = (count - ((x << 16) - unsafe { *self.big.get_unchecked(x) } as usize)) as u16;
        let x = x << 10;
        let y = {
            let slice = unsafe { self.small.get_unchecked(x..self.small.len().min(x + 1024)) };
            let mut size = slice.len();
            let mut t = 0;
            while size > 1 {
                let half = size / 2;
                let mid = t + half;
                let val = unsafe { *slice.get_unchecked(mid) };
                if (mid << 6) as u16 - val <= count {
                    t = mid;
                    size -= half;
                } else {
                    size = half;
                }
            }
            t
        };
        let x = x | y;

        let count = (count - ((y << 6) as u16 - unsafe { *self.small.get_unchecked(x) })) as u8;

        let v = unsafe { *self.raw.get_unchecked(x) };
        let mut k = !0u64;
        let mut r = x << 6;
        #[inline(always)]
        fn internal(k: &mut u64, r: &mut usize, s: usize, v: u64, count: u8) {
            let temp = *k << s;
            if (v | temp).count_zeros() as u8 <= count {
                *k = temp;
                *r |= s;
            }
        }
        internal(&mut k, &mut r, 1 << 5, v, count);
        internal(&mut k, &mut r, 1 << 4, v, count);
        internal(&mut k, &mut r, 1 << 3, v, count);
        internal(&mut k, &mut r, 1 << 2, v, count);
        internal(&mut k, &mut r, 1 << 1, v, count);
        internal(&mut k, &mut r, 1 << 0, v, count);
        r
    }
    fn select_zero(&self, count: usize) -> Option<usize> {
        if count >= self.len() - self.one_count {
            None
        } else {
            Some(unsafe { self.select_zero_unchecked(count) })
        }
    }
}

/// Wavelet行列. 整数列に関するクエリを処理できる
pub struct WaveletMatrix<const N: usize = 64>([Dict; N], usize);

impl<const N: usize> WaveletMatrix<N> {
    /// u32のスライスからWaveletMatrixを構築する
    #[must_use]
    pub fn from_u32_slice(v: &[u32]) -> Self {
        const { assert!(1 <= N && N <= 32) };
        let len = v.len();
        enum SliceOrVec<'a> {
            Slice(&'a [u32]),
            Vector(Vec<u32>),
        }
        use SliceOrVec::*;
        let mut v = Slice(v);

        Self(
            std::array::from_fn::<_, N, _>(|i| {
                let mut nv = Vec::with_capacity(len);
                let pv = match &v {
                    Slice(v) => *v,
                    Vector(v) => v,
                };
                let mask = 1 << (N - 1 - i);
                let mut ret = std::iter::repeat_n(0, (len + 63) >> 6).collect::<Box<_>>();

                for (i, &v) in pv.iter().enumerate() {
                    if v & mask == 0 {
                        nv.push(v);
                    } else {
                        ret[i >> 6] |= 1 << (i & 63);
                    }
                }
                for &v in pv {
                    if v & mask != 0 {
                        nv.push(v);
                    }
                }
                v = Vector(nv);
                Dict::new(ret)
            }),
            len,
        )
    }

    /// u64のスライスからWaveletMatrixを構築する
    #[must_use]
    pub fn from_u64_slice(v: &[u64]) -> Self {
        const { assert!(1 <= N && N <= 64) };
        let len = v.len();
        enum SliceOrVec<'a> {
            Slice(&'a [u64]),
            Vector(Vec<u64>),
        }
        use SliceOrVec::*;
        let mut v = Slice(v);

        Self(
            std::array::from_fn::<_, N, _>(|i| {
                let mut nv = Vec::with_capacity(len);
                let pv = match &v {
                    Slice(v) => *v,
                    Vector(v) => v.as_ref(),
                };
                let mask = 1 << (N - i - 1);
                let mut ret = std::iter::repeat_n(0, (len + 63) >> 6).collect::<Box<_>>();

                for (i, &v) in pv.iter().enumerate() {
                    if v & mask == 0 {
                        nv.push(v);
                    } else {
                        ret[i >> 6] |= 1 << (i & 63);
                    }
                }
                for &v in pv {
                    if v & mask != 0 {
                        nv.push(v);
                    }
                }
                v = Vector(nv);
                Dict::new(ret)
            }),
            len,
        )
    }

    /// WaveletMatrixの要素数を返す
    ///
    /// # Time complexity
    ///
    /// - *O*(1)
    #[must_use]
    pub fn len(&self) -> usize {
        self.1
    }

    /// WaveletMatrixが空かどうか返す
    ///
    /// # Time complexity
    ///
    /// - *O*(1)
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.1 == 0
    }

    fn range2pair(&self, range: impl std::ops::RangeBounds<usize>) -> (usize, usize) {
        use std::ops::Bound::*;
        let l = match range.start_bound() {
            Included(&i) => i,
            Excluded(&i) => i + 1,
            Unbounded => 0,
        };
        let r = match range.end_bound() {
            Included(&i) => i + 1,
            Excluded(&i) => i,
            Unbounded => self.len(),
        };
        assert!(l <= r && r <= self.len());
        (l, r)
    }

    unsafe fn rank_internal(&self, val: u64, mut r: usize) -> usize {
        let len = self.len();
        for (i, dict) in self.0.iter().enumerate() {
            if (val >> (N - i - 1)) & 1 == 1 {
                r = len - dict.one_count + unsafe { dict.rank_unchecked(r) };
            } else {
                r -= unsafe { dict.rank_unchecked(r) };
            }
        }
        r
    }

    unsafe fn rank_range_internal(&self, max: u64, mut l: usize, mut r: usize) -> usize {
        let len = self.len();
        let mut ret = 0;
        for (i, dict) in self.0.iter().enumerate() {
            if (max >> (N - i - 1)) & 1 == 1 {
                ret += r - l - unsafe { dict.rank_unchecked(r) - dict.rank_unchecked(l) };
                l = len - dict.one_count + unsafe { dict.rank_unchecked(l) };
                r = len - dict.one_count + unsafe { dict.rank_unchecked(r) };
            } else {
                l -= unsafe { dict.rank_unchecked(l) };
                r -= unsafe { dict.rank_unchecked(r) };
            }
        }
        ret
    }

    /// 数列の`index`番目の要素を得る
    ///
    /// # Constraints
    ///
    /// - `index < self.len()`
    pub fn access(&self, mut index: usize) -> u64 {
        assert!(index < self.len());

        let mut ret = 0;
        for (i, dict) in self.0.iter().enumerate() {
            let r = unsafe { dict.rank_unchecked(index) };
            if dict.get(index) {
                ret |= 1 << (N - i - 1);
                index = self.len() - dict.one_count + r;
            } else {
                index -= r;
            }
        }
        ret
    }

    /// `range`の範囲の`val`の個数を数える
    ///
    /// # Constraints
    ///
    /// - `range`は`0..self.len()`に含まれる区間である
    pub fn rank(&self, val: u64, range: impl std::ops::RangeBounds<usize>) -> usize {
        let (mut l, mut r) = self.range2pair(range);
        let len = self.len();
        for (i, dict) in self.0.iter().enumerate() {
            if (val >> (N - i - 1)) & 1 == 1 {
                l = len - dict.one_count + unsafe { dict.rank_unchecked(l) };
                r = len - dict.one_count + unsafe { dict.rank_unchecked(r) };
            } else {
                l -= unsafe { dict.rank_unchecked(l) };
                r -= unsafe { dict.rank_unchecked(r) };
            }
        }
        r - l
    }

    /// `idxrange`の中の`valrange`の範囲にある数の個数を数える
    ///
    /// # Constraints
    ///
    /// - `range`は`0..self.len()`に含まれる区間である
    pub fn rank_range(
        &self,
        valrange: impl std::ops::RangeBounds<u64>,
        idxrange: impl std::ops::RangeBounds<usize>,
    ) -> usize {
        use std::ops::Bound::*;

        let (l, r) = self.range2pair(idxrange);

        let Some(min) = (match valrange.start_bound() {
            Unbounded => Some(0),
            Included(&max) => Some(max),
            Excluded(&max) => max.checked_add(1),
        }) else {
            return 0;
        };
        let max = match valrange.end_bound() {
            Unbounded | Included(&u64::MAX) => {
                return r - l - unsafe { self.rank_range_internal(min, l, r) }
            }
            Included(&max) => max + 1,
            Excluded(&max) => max,
        };
        unsafe { self.rank_range_internal(max, l, r) - self.rank_range_internal(min, l, r) }
    }

    /// `left`以降で`count`番目の`val`の値の位置を返す
    ///
    /// 存在しなかった場合はNoneを返す
    ///
    /// # Constraints
    ///
    /// - `left < self.len()`
    pub fn select(&self, left: usize, val: u64, count: usize) -> Option<usize> {
        debug_assert!(left <= self.len());
        let len = self.len();
        let mut index = unsafe { self.rank_internal(val, left) }
            .checked_add(count)
            .filter(|&v| v < len)?;
        for (i, dict) in self.0.iter().rev().enumerate() {
            let q = (val >> i) & 1 == 1;
            if q {
                index = dict.select(index - (len - dict.one_count))?;
            } else {
                index = dict.select_zero(index).filter(|&v| v < len)?;
            }
        }
        Some(index)
    }

    /// `range`の範囲内の最大値を返す
    ///
    /// # Constraints
    ///
    /// - `range`は`0..self.len()`に含まれる区間である
    pub fn max(&self, range: impl std::ops::RangeBounds<usize>) -> u64 {
        let (mut l, mut r) = self.range2pair(range);
        let len = self.len();
        let mut ret = 0;
        for (i, dict) in self.0.iter().enumerate() {
            let sl = unsafe { dict.rank_unchecked(l) };
            let sr = unsafe { dict.rank_unchecked(r) };
            if sl == sr {
                l -= sl;
                r -= sr;
            } else {
                ret |= 1 << (N - 1 - i);
                l = len - dict.one_count + sl;
                r = len - dict.one_count + sr;
            }
        }
        ret
    }

    /// `range`の範囲内の最小値を返す
    ///
    /// # Constraints
    ///
    /// - `range`は`0..self.len()`に含まれる区間である
    pub fn min(&self, range: impl std::ops::RangeBounds<usize>) -> u64 {
        let (mut l, mut r) = self.range2pair(range);
        let len = self.len();
        let mut ret = 0;
        for (i, dict) in self.0.iter().enumerate() {
            let sl = unsafe { dict.rank_unchecked(l) };
            let sr = unsafe { dict.rank_unchecked(r) };
            if sr - sl == r - l {
                ret |= 1 << (N - 1 - i);
                l = len - dict.one_count + sl;
                r = len - dict.one_count + sr;
            } else {
                l -= sl;
                r -= sr;
            }
        }
        ret
    }

    /// `range`の範囲内で`n`番目に小さい値を返す
    ///
    /// # Constraints
    ///
    /// - `range`は`0..self.len()`に含まれる区間である
    /// - `n`は`range`に含まれる要素数未満である
    pub fn nth_smallest(&self, range: impl std::ops::RangeBounds<usize>, mut n: usize) -> u64 {
        let (mut l, mut r) = self.range2pair(range);
        let len = self.len();
        let mut ret = 0;
        assert!(n < r - l);
        for (i, dict) in self.0.iter().enumerate() {
            let sl = unsafe { dict.rank_unchecked(l) };
            let sr = unsafe { dict.rank_unchecked(r) };
            let count = (r - l) - (sr - sl);
            if count <= n {
                n -= count;
                ret |= 1 << (N - 1 - i);
                l = len - dict.one_count + sl;
                r = len - dict.one_count + sr;
            } else {
                l -= sl;
                r -= sr;
            }
        }
        ret
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let wm = WaveletMatrix::<4>::from_u32_slice(&[3, 1, 4, 1, 5, 9, 2, 6, 5, 3, 5]);
        assert_eq!(wm.access(4), 5);
        assert_eq!(wm.rank(5, 1..6), 1);
        assert_eq!(wm.min(4..7), 2);
        assert_eq!(wm.nth_smallest(3..8, 3), 6);
    }
}
