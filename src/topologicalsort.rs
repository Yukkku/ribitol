/// トポロジカルソートをする
///
/// # Time complexity
///
/// - *O*(*V* + *E*)
pub fn topological_sort<T: AsRef<[usize]>>(
    edges: &[T],
) -> impl Iterator<Item = usize> + use<'_, T> {
    let n = edges.len();
    let mut count = std::iter::repeat_n(0usize, n).collect::<Box<_>>();
    for v in edges {
        for &u in v.as_ref() {
            count[u] += 1;
        }
    }
    let stack = count
        .iter()
        .enumerate()
        .filter_map(|(i, &c)| (c == 0).then_some(i))
        .collect();

    struct Iter<'b, T: AsRef<[usize]>>(&'b [T], Box<[usize]>, Vec<usize>, usize);
    impl<'b, T: AsRef<[usize]>> Iterator for Iter<'b, T> {
        type Item = usize;
        fn next(&mut self) -> Option<usize> {
            let v = self.2.pop()?;
            self.3 -= 1;
            for &u in self.0[v].as_ref() {
                self.1[u] -= 1;
                if self.1[u] == 0 {
                    self.2.push(u);
                }
            }
            Some(v)
        }
        fn size_hint(&self) -> (usize, Option<usize>) {
            (self.2.len(), Some(self.3))
        }
    }
    Iter(edges, count, stack, n)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let edges = [vec![1], vec![], vec![3, 5], vec![0, 4], vec![1], vec![4]];
        let result = topological_sort(&edges).collect::<Vec<_>>();
        for (v, edges) in edges.iter().enumerate() {
            let idx = result
                .iter()
                .enumerate()
                .find_map(|(i, &s)| (s == v).then_some(i));
            for &u in edges {
                let idx2 = result
                    .iter()
                    .enumerate()
                    .find_map(|(i, &s)| (s == u).then_some(i));
                assert!(idx < idx2);
            }
        }
    }
}
