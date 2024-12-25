use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

const fn phi_const(mut n: u32) -> u32 {
    let mut r = 1;
    if n & 1 == 0 {
        let k = n.trailing_zeros();
        r <<= k - 1;
        n >>= k;
    }
    let mut i = 3;
    while i * i <= n {
        if n % i == 0 {
            r *= i - 1;
            n /= i;
            while n % i == 0 {
                n /= i;
                r *= i;
            }
        }
        i += 2;
    }
    if n != 1 {
        r *= n - 1;
    }
    r
}

/// 計算すると自動で mod `N` での値をとる数値型
#[derive(Clone, Copy, PartialEq, Eq, Default, Hash)]
pub struct ModInt<const N: u32 = 998244353>(u32);

impl<const N: u32> ModInt<N> {
    pub(super) const PHI: u32 = phi_const(N);

    /// `val`を`N`で割って`ModInt<N>`を作る
    ///
    /// # Time complexity
    ///
    /// - *O*(1)
    #[must_use]
    pub fn new(val: u32) -> Self {
        Self(val % N)
    }

    /// `N`で割った値をu32で取り出す.
    ///
    /// # Time complexity
    ///
    /// - *O*(1)
    #[must_use]
    pub fn val(self) -> u32 {
        self.0
    }

    fn mul_pow(self, mut b: Self, mut s: u32) -> Self {
        let mut r = if s & 1 == 1 { self * b } else { self };
        loop {
            s >>= 1;
            if s == 0 {
                return r;
            }
            b *= b;
            if s & 1 == 1 {
                r *= b;
            }
        }
    }

    /// 値を`s`乗する
    ///
    /// # Time complexity
    ///
    /// - *O*(1)
    ///
    /// # Time complexity
    ///
    /// - *O*(log *s*)
    #[must_use]
    pub fn pow(&self, s: u32) -> Self {
        Self(1).mul_pow(*self, s)
    }

    /// 値の逆数を求める
    ///
    /// # Constraints
    ///
    /// - 値は`N`と互いに素である
    ///
    /// # Time complexity
    ///
    /// - *O*(log *N*)
    #[must_use]
    pub fn inv(&self) -> Self {
        Self(1).mul_pow(*self, Self::PHI - 1)
    }
}

impl<const N: u32> Add for ModInt<N> {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        let (r, f) = self.0.overflowing_add(rhs.0);
        Self(if f || r >= N { r.wrapping_sub(N) } else { r })
    }
}
impl<const N: u32> Sub for ModInt<N> {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        let (r, f) = self.0.overflowing_sub(rhs.0);
        Self(if f { r.wrapping_add(N) } else { r })
    }
}
impl<const N: u32> Mul for ModInt<N> {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        Self((self.0 as u64 * rhs.0 as u64 % N as u64) as u32)
    }
}
impl<const N: u32> Div for ModInt<N> {
    type Output = Self;
    fn div(self, rhs: Self) -> Self {
        #[allow(clippy::suspicious_arithmetic_impl)]
        self.mul_pow(rhs, Self::PHI - 1)
    }
}

impl<const N: u32> Neg for ModInt<N> {
    type Output = Self;
    fn neg(self) -> Self {
        Self(if self.0 == 0 { 0 } else { N - self.0 })
    }
}
impl<const N: u32> Neg for &ModInt<N> {
    type Output = ModInt<N>;
    fn neg(self) -> ModInt<N> {
        ModInt(if self.0 == 0 { 0 } else { N - self.0 })
    }
}

macro_rules! impl_ops {
    ($({$tr: ident, $mt: ident, $tr2: ident, $mt2: ident}),*$(,)?) => {$(
        impl<const N: u32> $tr for &ModInt<N> {
            type Output = ModInt<N>;
            fn $mt(self, rhs: Self) -> ModInt<N> {
                (*self).$mt(*rhs)
            }
        }
        impl<const N: u32> $tr<&Self> for ModInt<N> {
            type Output = Self;
            fn $mt(self, rhs: &Self) -> Self {
                self.$mt(*rhs)
            }
        }
        impl<const N: u32> $tr<ModInt<N>> for &ModInt<N> {
            type Output = ModInt<N>;
            fn $mt(self, rhs: ModInt<N>) -> ModInt<N> {
                (*self).$mt(rhs)
            }
        }
        impl<const N: u32> $tr2 for ModInt<N> {
            fn $mt2(&mut self, rhs: Self) {
                *self = self.$mt(rhs);
            }
        }
        impl<const N: u32> $tr2<&Self> for ModInt<N> {
            fn $mt2(&mut self, rhs: &Self) {
                *self = self.$mt(rhs);
            }
        }
    )*};
}

impl_ops! {
    { Add, add, AddAssign, add_assign },
    { Sub, sub, SubAssign, sub_assign },
    { Mul, mul, MulAssign, mul_assign },
    { Div, div, DivAssign, div_assign },
}

macro_rules! impl_cast_uint {
    ($($t: ty),*$(,)?) => {$(
        impl<const N: u32> From<$t> for ModInt<N> {
            fn from(value: $t) -> Self {
                if const { <$t>::BITS > u32::BITS } {
                    Self((value % N as $t) as u32)
                } else {
                    Self(value as u32 % N)
                }
            }
        }
    )*};
}
impl_cast_uint! { u8, u16, u32, u64, u128, usize }

macro_rules! impl_cast_int {
    ($($t: ty),*$(,)?) => {$(
        impl<const N: u32> From<$t> for ModInt<N> {
            fn from(value: $t) -> Self {
                if const { <$t>::BITS > u32::BITS } {
                    Self((value.rem_euclid(N as $t)) as u32)
                } else {
                    if const { N <= i32::MAX as u32 } {
                        Self((value as i32).rem_euclid(N as i32) as u32)
                    } else {
                        if value < 0 {
                            Self(N - value.abs_diff(0) as u32)
                        } else {
                            Self(value as u32)
                        }
                    }
                }
            }
        }

    )*};
}
impl_cast_int! { i8, i16, i32, i64, i128, isize }

impl<const N: u32> std::fmt::Debug for ModInt<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
impl<const N: u32> std::fmt::Display for ModInt<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prime() {
        type Mint = ModInt<998244353>;
        let a = Mint::new(3141592653);
        let b = Mint::new(2718281828);
        assert_eq!((a + b).val(), 868652716);
        assert_eq!((a - b).val(), 423310825);
        assert_eq!((a * b).val(), 675854546);
        assert_eq!((a / b).val(), 877474378);
    }

    #[test]
    fn non_prime() {
        type Mint = ModInt<123456789>;
        let a = Mint::new(577215664);
        let b = Mint::new(2718281828);
        assert_eq!((a + b).val(), 85620978);
        assert_eq!((a - b).val(), 81156038);
        assert_eq!((a * b).val(), 121926614);
        assert_eq!((a / b).val(), 44131967);
    }
}
