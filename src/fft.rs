use super::modint::ModInt;

const fn powmod_const(mut a: u32, mut b: u32, n: u32) -> u32 {
    let mut r = 1;
    while b != 0 {
        if b & 1 == 1 {
            r = (r as u64 * a as u64 % n as u64) as u32;
        }
        b >>= 1;
        a = (a as u64 * a as u64 % n as u64) as u32;
    }
    r
}

struct FFTParam<const N: u32>;
impl<const N: u32> FFTParam<N> {
    const D: u32 = ModInt::<N>::PHI.trailing_zeros();
    const R: [u32; 32] = {
        assert!(ModInt::<N>::PHI == N - 1);
        let mut s = 2;
        loop {
            let r = powmod_const(s, ModInt::<N>::PHI >> Self::D, N);
            let mut t = r;
            let mut j = 0;
            while j < Self::D - 1 {
                t = (t as u64 * t as u64 % N as u64) as u32;
                j += 1;
            }
            if t == N - 1 {
                let mut p = [0; 32];
                let mut j = Self::D as usize;
                p[j] = r;
                while j > 0 {
                    p[j - 1] = (p[j] as u64 * p[j] as u64 % N as u64) as u32;
                    j -= 1;
                }
                break p;
            }
            s += 1;
        }
    };
    const R_INV: [u32; 32] = {
        let mut r = [0; 32];
        let mut i = 0;
        while i < 32 {
            r[i] = powmod_const(Self::R[i], ModInt::<N>::PHI - 1, N);
            i += 1;
        }
        r
    };
}

pub fn fft<const N: u32>(a: &mut [ModInt<N>]) {
    debug_assert!(a.len().is_power_of_two());
    debug_assert!(a.len() <= 1 << FFTParam::<N>::D);
    let n = a.len().trailing_zeros();
    for i in 0..a.len() {
        let j = i.reverse_bits() >> (usize::BITS - n);
        if i < j {
            a.swap(i, j);
        }
    }

    for i in 0..n {
        let r = ModInt::<N>::new(FFTParam::<N>::R[i as usize + 1]);
        for j in (0..a.len()).step_by(1 << (i + 1)) {
            let mut s = ModInt::<N>::new(1);
            let k = j + (1 << i);
            for l in 0..1 << i {
                let x = a[l + j];
                let y = a[l + k];
                a[l + j] = x + y * s;
                a[l + k] = x - y * s;
                s *= r;
            }
        }
    }
}

pub fn ifft<const N: u32>(a: &mut [ModInt<N>]) {
    debug_assert!(a.len().is_power_of_two());
    debug_assert!(a.len() <= 1 << FFTParam::<N>::D);
    let n = a.len().trailing_zeros();
    for i in 0..a.len() {
        let j = i.reverse_bits() >> (usize::BITS - n);
        if i < j {
            a.swap(i, j);
        }
    }

    for i in 0..n {
        let r = ModInt::<N>::new(FFTParam::<N>::R_INV[i as usize + 1]);
        for j in (0..a.len()).step_by(1 << (i + 1)) {
            let mut s = ModInt::<N>::new(1);
            let k = j + (1 << i);
            for l in 0..1 << i {
                let x = a[l + j];
                let y = a[l + k];
                a[l + j] = x + y * s;
                a[l + k] = x - y * s;
                s *= r;
            }
        }
    }

    let rev = ModInt::<N>::new(a.len() as u32).inv();
    for a in a.iter_mut() {
        *a *= rev;
    }
}

pub fn convolution<const N: u32>(a: &[ModInt<N>], b: &[ModInt<N>]) -> Vec<ModInt<N>> {
    let len = a.len() + b.len() - 1;
    let len_ceil = len.next_power_of_two();
    let mut a = {
        let mut na = vec![ModInt::<N>::new(0); len_ceil];
        for (i, &a) in a.iter().enumerate() {
            na[i] = a;
        }
        na
    };
    let mut b = {
        let mut nb = vec![ModInt::<N>::new(0); len_ceil];
        for (i, &b) in b.iter().enumerate() {
            nb[i] = b;
        }
        nb
    };
    fft(&mut a);
    fft(&mut b);
    for i in 0..len_ceil {
        a[i] *= b[i];
    }
    ifft(&mut a);
    a.drain(len..);
    a
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fft_ifft() {
        type Mint = ModInt<998244353>;
        let a = [3, 1, 4, 1, 5, 9, 2, 6].map(Mint::new);
        let mut b = a.clone();
        fft(&mut b);
        ifft(&mut b);
        assert_eq!(a, b);
    }

    #[test]
    fn conv() {
        type Mint = ModInt<998244353>;
        let a = [3, 1, 4, 1, 5, 9, 2, 6].map(Mint::new);
        let b = [5, 3, 5, 8, 9, 7, 9, 3].map(Mint::new);
        let mut c = [Mint::new(0); 15];
        for i in 0..8 {
            for j in 0..8 {
                c[i + j] += a[i] * b[j];
            }
        }

        assert_eq!(convolution(&a, &b), c);
    }
}
