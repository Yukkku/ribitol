use std::mem::MaybeUninit;

/// 平面上の点をHilbert曲線に対応させる
///
/// # Time complexity
///
/// - *O*(1)
pub fn hilbert_order((mut x, mut y): (u32, u32)) -> u64 {
    let mut r = 0;
    for i in (0..31).rev() {
        let g = 1 << i;
        if x & g > 0 {
            r |= if y & g > 0 { 2 } else { 1 } << (i * 2);
        } else {
            if y & g > 0 {
                r |= 3 << (i * 2);
                x = !x;
                y = !y;
            }
            std::mem::swap(&mut x, &mut y);
        }
    }
    r
}

/// 実行中にpanicを起こしたとき, 値を正しくdropさせるための構造体
struct Manager<'a, T> {
    output: &'a mut [MaybeUninit<T>],
    progress: usize,
    order: Box<[(u64, usize)]>,
}

impl<T> Manager<'_, T> {
    fn len(&self) -> usize {
        self.output.len()
    }

    fn calc(mut self, mut f: impl FnMut(usize) -> T) {
        while self.progress < self.len() {
            let index = self.order[self.progress].1;
            self.output[index].write(f(index));
            self.progress += 1;
        }
    }
}

impl<T> Drop for Manager<'_, T> {
    fn drop(&mut self) {
        if self.progress == self.len() || !std::mem::needs_drop::<T>() {
            return;
        }
        for i in 0..self.progress {
            unsafe {
                self.output[self.order[i].1].assume_init_drop();
            }
        }
    }
}

/// Mo's Algorithmの計算を行うトレイト
pub trait MoStatus {
    /// Mo's Algorithmの各クエリの回答の型
    type Output;

    /// *f*(0, 0) を生成する
    #[must_use]
    fn make_origin() -> Self;

    /// *f*(*x*, *y*) の値を *f*(*x* + 1, *y*) に書き換える.
    ///
    /// 引数`position`には (*x*, *y*) が与えられる.
    fn first_inc(&mut self, position: (u32, u32));

    /// *f*(*x*, *y*) の値を *f*(*x* - 1, *y*) に書き換える.
    ///
    /// 引数`position`には (*x*, *y*) が与えられる.
    fn first_dec(&mut self, position: (u32, u32));

    /// *f*(*x*, *y*) の値を *f*(*x*, *y + 1*) に書き換える.
    ///
    /// 引数`position`には (*x*, *y*) が与えられる.
    fn second_inc(&mut self, position: (u32, u32));

    /// *f*(*x*, *y*) の値を *f*(*x*, *y - 1*) に書き換える.
    ///
    /// 引数`position`には (*x*, *y*) が与えられる.
    fn second_dec(&mut self, position: (u32, u32));

    /// *f*(*x*, *y*) の値から必要なデータを抜き出す.
    ///
    /// 引数`position`には (*x*, *y*) が与えられる.
    #[must_use]
    fn make_output(&self, position: (u32, u32)) -> Self::Output;
}

/// Mo's Algorithmを用いて計算を行う.
///
/// 全てのクエリが `query.0 <= query.1` を満たすとき, 途中の状態として `point.0 > point.1` にならないことが保証される.
///
/// # Time complexity
///
/// - *O*(*N*√*Q*)
///   ただし, *N*はクエリとして与えられる数値の最大値
#[must_use]
pub fn mo<T: MoStatus>(querys: &[(u32, u32)]) -> Box<[T::Output]> {
    let n = querys.len();
    let mut output = Box::<[T::Output]>::new_uninit_slice(n);
    let order = {
        let mut temp = (0..n)
            .map(|i| (hilbert_order(querys[i]), i))
            .collect::<Box<_>>();
        temp.sort_unstable();
        temp
    };

    let manager = Manager {
        output: &mut output,
        progress: 0,
        order,
    };

    let mut status = T::make_origin();
    let mut position = (0, 0);
    manager.calc(|i| {
        let next = querys[i];
        while next.0 < position.0 {
            status.first_dec(position);
            position.0 -= 1;
        }
        while position.1 < next.1 {
            status.second_inc(position);
            position.1 += 1;
        }
        while position.0 < next.0 {
            status.first_inc(position);
            position.0 += 1;
        }
        while next.1 < position.1 {
            status.second_dec(position);
            position.1 -= 1;
        }
        status.make_output(position)
    });
    unsafe { output.assume_init() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subtract() {
        /// 関数 *f*(*x*, *y*) = *y* - *x* でのMo
        struct Subtract(u32);
        impl MoStatus for Subtract {
            type Output = u32;
            fn make_origin() -> Self {
                Self(0)
            }
            fn first_inc(&mut self, position: (u32, u32)) {
                assert_eq!(self.0, position.1 - position.0);
                assert_ne!(position.0, position.1);
                self.0 -= 1;
            }
            fn first_dec(&mut self, position: (u32, u32)) {
                assert_eq!(self.0, position.1 - position.0);
                self.0 += 1;
            }
            fn second_inc(&mut self, position: (u32, u32)) {
                assert_eq!(self.0, position.1 - position.0);
                self.0 += 1;
            }
            fn second_dec(&mut self, position: (u32, u32)) {
                assert_eq!(self.0, position.1 - position.0);
                assert_ne!(position.0, position.1);
                self.0 -= 1;
            }
            fn make_output(&self, _: (u32, u32)) -> u32 {
                self.0
            }
        }

        let points = [
            (31, 41),
            (59, 265),
            (35, 89),
            (79, 323),
            (84, 626),
            (43, 383),
            (27, 95),
            (2, 88),
        ];
        assert_eq!(
            mo::<Subtract>(&points).as_ref(),
            &points.map(|(x, y)| y - x)
        );
    }
}
