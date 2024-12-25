use crate::util::HasMin;

use super::radixheap::{Radix, RadixHeap};
use super::util::HasZero;

/// RadixHeapに距離と頂点番号をセットで入れるための型
#[derive(Clone, Copy)]
struct DijkstraItem<T: Radix>(T, usize);
impl<T: Radix> PartialEq for DijkstraItem<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl<T: Radix> PartialOrd for DijkstraItem<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.0.cmp(&other.0))
    }
}
impl<T: Radix> Eq for DijkstraItem<T> {}
impl<T: Radix> Ord for DijkstraItem<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}
impl<T: Radix> HasMin for DijkstraItem<T> {
    fn min_value() -> Self {
        Self(T::min_value(), 0)
    }
}
impl<T: Radix> Radix for DijkstraItem<T> {
    const MAX_DIST: usize = T::MAX_DIST;

    fn radix_dist(&self, rhs: &Self) -> usize {
        self.0.radix_dist(&rhs.0)
    }
}

/// ダイクストラ法を用いて最短経路を求める.
///
/// * `edges` - グラフの隣接リストによる表現 (「「辺のコストと行先の組」の配列」の配列)
/// * `start` - 探索の始点
/// * `goal` - 探索の終点
///
/// # Constraints
///
/// - 辺の情報は非負. 言い換えると`v`をある辺のコスト, `c`を距離としてあり得る値とすると, `c + v >= c` が常に成り立つ
/// - 同じものを足す操作は順序関係を保つ. 正確には`v`をある辺のコスト, `c`, `d`を距離としてあり得る値とすると, `c < d` と `c + v < d + v` は同値
///
/// # Time complexity
///
/// - *O*(*E* log *V*)
#[must_use]
pub fn dijkstra<T: Radix + HasZero + std::ops::Add<Output = T>>(
    edges: &[impl AsRef<[(usize, T)]>],
    start: usize,
    goal: usize,
) -> Option<T> {
    if start == goal {
        return Some(T::zero());
    }
    let mut heap = RadixHeap::new();
    let mut upper = vec![None; edges.len()].into_boxed_slice();
    upper[start] = Some(T::zero());
    for &(i, j) in edges[start].as_ref() {
        upper[i] = Some(j);
        heap.push(DijkstraItem(j, i));
    }
    while let Some(DijkstraItem(distance, v)) = heap.pop() {
        if upper[v].is_none_or(|d| d != distance) {
            continue;
        }
        if v == goal {
            return Some(distance);
        }
        for &(u, cost) in edges[v].as_ref() {
            let distance = distance + cost;
            if upper[u].is_none_or(|d| distance < d) {
                upper[u] = Some(distance);
                heap.push(DijkstraItem(distance, u));
            }
        }
    }
    None
}

/// ダイクストラ法を用いて最短経路木を作成する.
///
/// 返り値は各頂点についての「到達可能なら距離と1つ前の頂点の組, 到達不可能ならNone」の配列で, `start`の「距離と1つ前の頂点」は`start`自身である.
///
/// * `edges` - グラフの隣接リストによる表現 (「「辺のコストと行先の組」の配列」の配列)
/// * `start` - 距離の基準の点
///
/// # Constraints
///
/// - 辺の情報は非負. 言い換えると`v`をある辺のコスト, `c`を距離としてあり得る値とすると, `c + v >= c` が常に成り立つ
/// - 同じものを足す操作は順序関係を保つ. 正確には`v`をある辺のコスト, `c`, `d`を距離としてあり得る値とすると, `c < d` と `c + v < d + v` は同値
///
/// # Time complexity
///
/// - *O*(*E* log *V*)
#[must_use]
pub fn dijkstra_tree<T: Radix + HasZero + std::ops::Add<Output = T>>(
    edges: &[impl AsRef<[(usize, T)]>],
    start: usize,
) -> Box<[Option<(T, usize)>]> {
    let mut heap = RadixHeap::new();
    let mut nodes = vec![None; edges.len()].into_boxed_slice();
    {
        let zero = T::zero();
        nodes[start] = Some((zero, start));
        heap.push(DijkstraItem(zero, start));
    }
    while let Some(DijkstraItem(distance, v)) = heap.pop() {
        if nodes[v].is_none_or(|(d, _)| d != distance) {
            continue;
        }
        for &(u, cost) in edges[v].as_ref() {
            let distance = distance + cost;
            if nodes[u].is_none_or(|(d, _)| distance < d) {
                nodes[u] = Some((distance, v));
                heap.push(DijkstraItem(distance, u));
            }
        }
    }
    nodes
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 以下のようなグラフの隣接リスト表現
    ///
    ///  S--3->*--7->*
    ///  |     |     |
    ///  2     5     2
    ///  v     v     v
    ///  *--4->*--9->G
    fn sample_graph() -> [Vec<(usize, i32)>; 6] {
        [
            vec![(1, 3), (3, 2)],
            vec![(2, 7), (4, 5)],
            vec![(5, 2)],
            vec![(4, 4)],
            vec![(5, 9)],
            vec![],
        ]
    }
    #[test]
    fn distance() {
        assert_eq!(dijkstra(&sample_graph(), 0, 5), Some(12));
    }

    #[test]
    fn tree() {
        assert_eq!(
            dijkstra_tree(&sample_graph(), 0).as_ref(),
            &[
                Some((0, 0)),
                Some((3, 0)),
                Some((10, 1)),
                Some((2, 0)),
                Some((6, 3)),
                Some((12, 2))
            ]
        );
    }
}
