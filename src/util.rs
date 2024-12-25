/// 「0」に相当する値が存在することを表すトレイト
pub trait HasZero {
    /// 0の値を返す
    fn zero() -> Self;
}
/// 「1」に相当する値が存在することを表すトレイト
pub trait HasOne {
    /// 1の値を返す
    fn one() -> Self;
}
/// その型に最小値が存在することを表すトレイト
pub trait HasMin {
    /// その型が取り得る最も小さい値を返す
    fn min_value() -> Self;
}
/// その型に最大値が存在することを表すトレイト
pub trait HasMax {
    /// その型が取り得る最も大きい値を返す
    fn max_value() -> Self;
}

/// 整数型にHasZero, HasOne, HasMin, HasMaxを実装するマクロ
macro_rules! impl_zero {
    ($($t: ty),*) => {$(
        impl HasZero for $t {
            fn zero() -> $t { 0 }
        }
        impl HasOne for $t {
            fn one() -> $t { 1 }
        }
        impl HasMin for $t {
            fn min_value() -> $t { Self::MIN }
        }
        impl HasMax for $t {
            fn max_value() -> $t { Self::MAX }
        }
    )*};
}

impl_zero! { u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize }

/// マグマ. 二項演算で閉じている代数構造
pub trait Magma {
    /// マグマの元の型
    type T: Eq;
    /// 二項演算
    fn op(&self, lhs: &Self::T, rhs: &Self::T) -> Self::T;
}

/// マグマに単位元があることを表すトレイト
pub trait Identity: Magma<T: Clone> {
    /// 単位元を構築して返す
    fn e(&self) -> Self::T;
}

/// マグマの全ての元が逆元を持つことを表すトレイト
pub trait Inverse: Magma {
    /// 逆元を返す
    fn inv(&self, v: &Self::T) -> Self::T;

    /// self.op(lhs, &self.inv(rhs)) と同じ
    fn opinv(&self, lhs: &Self::T, rhs: &Self::T) -> Self::T {
        self.op(lhs, &self.inv(rhs))
    }
    /// self.op(&self.inv(lhs), rhs) と同じ
    fn invop(&self, lhs: &Self::T, rhs: &Self::T) -> Self::T {
        self.op(&self.inv(lhs), rhs)
    }
}

/// マグマが結合律が成り立つことを表すトレイト
pub trait Associativity: Magma {}
/// マグマが交換則が成り立つことを表すトレイト
pub trait Commutativity: Magma {}
/// マグマが冪等則が成り立つことを表すトレイト
pub trait Idempotence: Magma {}
