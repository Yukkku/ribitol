use super::unionfind::UnionFind;

/// 最小全域木を構築する. 連結でないグラフが与えられた場合は最小全域森 (連結成分数を変えないまま辺のコストの和を最小化したもの) を構築する.
///
/// * `n` - グラフの頂点数
/// * `edges` - 辺の情報. `(頂点, 頂点, 辺のコスト)`というタプルの配列
///
/// 返り値は最小全域木の辺のコストを吐き出すイテレータを返す.
#[must_use]
pub fn kruskal<'a, T: Ord>(
    n: usize,
    edges: &'a [(usize, usize, T)],
) -> impl Iterator<Item = usize> + use<'a, T> {
    let mut iv = (0..edges.len()).collect::<Vec<_>>();
    iv.sort_unstable_by_key(|&i| &edges[i].2);
    let mut uf = UnionFind::new(n);

    iv.into_iter().filter(move |&i| {
        let edge = &edges[i];
        if uf.same(edge.0, edge.1) {
            false
        } else {
            uf.union(edge.0, edge.1);
            true
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let edges = kruskal(
            4,
            &[
                (0, 1, 4),
                (0, 2, 2),
                (0, 3, 3),
                (1, 2, 6),
                (1, 3, 8),
                (2, 3, 1),
                (1, 1, 0),
            ],
        )
        .collect::<Box<_>>();
        assert_eq!(edges.as_ref(), &[5, 0, 1]);
    }
}
