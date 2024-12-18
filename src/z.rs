/// Z配列を作成する
///
/// # Time complexity
///
/// - *O*(*N*)
#[must_use]
pub fn z(slice: &[impl Eq]) -> Vec<usize> {
    let n = slice.len();
    let mut z = Vec::with_capacity(n);
    z.push(n);
    let mut l = 1;
    let mut r = 1;
    for i in 1..n {
        if l < i && z[i - l] + i < r {
            z.push(z[i - l]);
            continue;
        }
        if r < i {
            r = i;
        }
        l = i;
        while slice.get(r) == slice.get(r - i) {
            r += 1;
        }
        z.push(r - l);
    }
    z
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        fn naive_z(slice: &[u8]) -> Vec<usize> {
            let mut z = vec![];
            z.push(slice.len());
            for i in 1..slice.len() {
                for j in 0.. {
                    if slice.get(j) != slice.get(i + j) {
                        z.push(j);
                        break;
                    }
                }
            }
            z
        }

        let a = b"atatata_and_atatata";
        let b = b"tentekotenten";
        let c = b"atatatatatatatatatata";
        let d = b"okayamaken_okayamashi";
        assert_eq!(z(a), naive_z(a));
        assert_eq!(z(b), naive_z(b));
        assert_eq!(z(c), naive_z(c));
        assert_eq!(z(d), naive_z(d));
    }
}
