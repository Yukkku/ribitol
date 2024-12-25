use std::num::NonZero;

const MOD: u64 = 0xffffffffffffffc5;
const R: NonZero<u64> = NonZero::new(!MOD + 1).unwrap();

const BASE: u64 = {
    const DBASE: u64 = 0xf464373a98197da9;
    let mut r = 1;
    let mut h = 0;
    let bytes = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/", file!()));
    let mut i = 0;
    while i < bytes.len() {
        h = add_mod(h, mul_mr(r, bytes[i] as u64));
        r = mul_mr(r, DBASE);
        i += 1;
    }
    if h <= 1 {
        DBASE
    } else {
        h
    }
};

/// モンゴメリリダクション
const fn mr(a: u128) -> u64 {
    const M: u64 = {
        let mut m = 0;
        let mut k = 0u64;
        {
            let mut i = 0;
            while i < 64 {
                if (k >> i) & 1 == 0 {
                    k = k.wrapping_add(MOD << i);
                    m |= 1 << i;
                }
                i += 1;
            }
        }
        m
    };
    let (b, f) = a.overflowing_add(M.wrapping_mul(a as u64) as u128 * MOD as u128);
    let b = (b >> 64) as u64;
    if f || b >= MOD {
        b.wrapping_sub(MOD)
    } else {
        b
    }
}

/// 2つの数を掛けた後, モンゴメリリダクションを行う
const fn mul_mr(a: u64, b: u64) -> u64 {
    mr(a as u128 * b as u128)
}

const fn add_mod(a: u64, b: u64) -> u64 {
    let (r, f) = a.overflowing_add(b);
    if f || r >= MOD {
        r.wrapping_sub(MOD)
    } else {
        r
    }
}

const fn sub_mod(a: u64, b: u64) -> u64 {
    let (r, f) = a.overflowing_sub(b);
    if f {
        r.wrapping_add(MOD)
    } else {
        r
    }
}

/// ローリングハッシュの型
///
/// 文字列の比較を確率的に定数時間で行うことが出来る
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct RollingHash(u64, NonZero<u64>);

impl RollingHash {
    /// 空の列のRollingHashを得る
    ///
    /// # Time complexity
    ///
    /// - *O*(1)
    #[must_use]
    pub fn new() -> Self {
        Self(0, R)
    }

    /// バイト列のRollingHashを得る
    ///
    /// # Time complexity
    ///
    /// - *O*(`bytes.len()`)
    #[must_use]
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let mut r = R.get();
        let mut h = 0;
        for &byte in bytes {
            h = add_mod(h, mul_mr(r, byte as u64));
            r = mul_mr(r, BASE);
        }
        Self(h, NonZero::new(r).unwrap())
    }
}

impl std::ops::Add for RollingHash {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self(
            add_mod(self.0, mul_mr(self.1.get(), rhs.0)),
            NonZero::new(mul_mr(self.1.get(), rhs.1.get())).unwrap(),
        )
    }
}

impl std::ops::Sub for RollingHash {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        const { assert!(MOD == 0xffffffffffffffc5) };
        /// v^2^bを計算する
        fn pow_2_pow(mut v: u64, b: u8) -> u64 {
            for _ in 0..b {
                v = mul_mr(v, v);
            }
            v
        }

        // 0xffffffffffffffc3 乗する
        // 0b111...1111000011
        let finv = {
            let r = rhs.1.get();
            // r^3
            let r3 = mul_mr(r, mul_mr(r, r));
            // r^(2^3-1)
            let rt = mul_mr(r, mul_mr(r3, r3));
            // r^(2^6-1)
            let rt = mul_mr(rt, pow_2_pow(rt, 3));
            // r^(2^7-1)
            let rt = mul_mr(r, mul_mr(rt, rt));
            // r^(2^14-1)
            let rt = mul_mr(rt, pow_2_pow(rt, 7));
            // r^(2^28-1)
            let rt = mul_mr(rt, pow_2_pow(rt, 14));
            // r^(2^29-1)
            let rt = mul_mr(r, mul_mr(rt, rt));
            // r^(2^58-1)
            let rt = mul_mr(rt, pow_2_pow(rt, 29));
            // r^(2^64-61)
            mul_mr(r3, pow_2_pow(rt, 6))
        };
        let nb = mul_mr(self.1.get(), finv);
        Self(
            sub_mod(self.0, mul_mr(nb, rhs.0)),
            NonZero::new(nb).unwrap(),
        )
    }
}

impl std::fmt::Debug for RollingHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // write!(f, "{:0>16x}", self.0):
        write!(f, "{:?} {:?}", self.0, self.1)
    }
}

impl Default for RollingHash {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add() {
        let a = RollingHash::from_bytes(b"Hello, World!");
        let b = RollingHash::from_bytes(b"Hello, ");
        let c = RollingHash::from_bytes(b"World!");
        assert_eq!(a, b + c);
        assert_ne!(a, c + b);
    }

    #[test]
    fn sub() {
        let a = RollingHash::from_bytes(b"Hello, World!");
        let b = RollingHash::from_bytes(b"Hello, ");
        let c = RollingHash::from_bytes(b"World!");
        assert_eq!(a - c, b);
        assert_ne!(a - b, c);
    }
}
