/// 強連結成分分解を行い,「1つの強連結成分の要素の配列」のイテレータを返す
///
/// # Time complexity
///
/// - *O*(*N*)
///   ここで, *N*は頂点数
pub fn scc(edges: &[impl AsRef<[usize]>]) -> impl Iterator<Item = Vec<usize>> {
    let n = edges.len();
    let mut v = std::iter::repeat_n(0u8, n).collect::<Box<_>>();
    let mut rs = Vec::with_capacity(n);
    for i in 0..n {
        if v[i] != 0 {
            continue;
        }
        let mut stack = vec![i];
        v[i] = 1;
        while let Some(i) = stack.pop() {
            if v[i] == 2 {
                rs.push(i);
                continue;
            }
            v[i] = 2;
            stack.push(i);
            for &j in edges[i].as_ref() {
                if v[j] == 0 {
                    stack.push(j);
                    v[j] = 1;
                }
            }
        }
    }

    let mut rev = (0..n).map(|_| vec![]).collect::<Box<_>>();
    for (i, edges) in edges.iter().enumerate() {
        for &j in edges.as_ref() {
            rev[j].push(i);
        }
    }
    let mut flgs = vec![0u64; (n + 63) >> 6].into_boxed_slice();
    std::iter::from_fn(move || {
        let v = loop {
            let v = rs.pop()?;
            if (flgs[v >> 6] >> (v & 63)) & 1 == 0 {
                break v;
            }
        };
        let mut r = vec![v];
        flgs[v >> 6] |= 1 << (v & 63);
        let mut stack = vec![v];
        while let Some(k) = stack.pop() {
            for &u in &rev[k] {
                if (flgs[u >> 6] >> (u & 63)) & 1 == 1 {
                    continue;
                }
                flgs[u >> 6] |= 1 << (u & 63);
                stack.push(u);
                r.push(u);
            }
        }
        Some(r)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        // Library Checkerの入出力例の問題
        // https://judge.yosupo.jp/problem/scc

        let mut result = scc(&[[3].as_ref(), &[4], &[], &[0], &[1, 2], &[2, 5]])
            .map(|mut v| {
                v.sort();
                v
            })
            .collect::<Vec<_>>();
        result.sort();
        assert_eq!(result, [vec![0, 3], vec![1, 4], vec![2], vec! {5}]);
    }
}
