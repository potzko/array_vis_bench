/// A gap sequence for shell sort.
///
/// Implementations return gaps in **descending** order (largest first) so the
/// caller can iterate them directly without reversing.
pub trait GapSequence {
    const NAME: &'static str;
    const BIG_O: &'static str;
    fn gaps(len: usize) -> Vec<usize>;
}

// ---------------------------------------------------------------------------
// Shell's original sequence: n/2, n/4, ..., 1
// ---------------------------------------------------------------------------
pub struct Classic;
impl GapSequence for Classic {
    const NAME: &'static str = "classic";
    const BIG_O: &'static str = "O(N^2)";

    fn gaps(len: usize) -> Vec<usize> {
        let mut gaps = Vec::new();
        let mut gap = len / 2;
        while gap >= 1 {
            gaps.push(gap);
            gap /= 2;
        }
        gaps
    }
}

// ---------------------------------------------------------------------------
// Knuth sequence: 1, 4, 13, 40, 121, ...  (3k+1)
// ---------------------------------------------------------------------------
pub struct Knuth;
impl GapSequence for Knuth {
    const NAME: &'static str = "knuth";
    const BIG_O: &'static str = "O(N^(3/2))";

    fn gaps(len: usize) -> Vec<usize> {
        let mut k = 1usize;
        while k < len {
            k = 3 * k + 1;
        }
        k = (k - 1) / 3;
        let mut gaps = Vec::new();
        while k >= 1 {
            gaps.push(k);
            k = (k - 1) / 3;
        }
        gaps
    }
}

// ---------------------------------------------------------------------------
// Hibbard sequence: 1, 3, 7, 15, 31, ...  (2^k - 1)
// ---------------------------------------------------------------------------
pub struct Hibbard;
impl GapSequence for Hibbard {
    const NAME: &'static str = "hibbard";
    const BIG_O: &'static str = "O(N^(3/2))";

    fn gaps(len: usize) -> Vec<usize> {
        let mut k = 1usize;
        while k < len {
            k = 2 * k + 1;
        }
        k = (k - 1) / 2;
        let mut gaps = Vec::new();
        while k >= 1 {
            gaps.push(k);
            k = (k - 1) / 2;
        }
        gaps
    }
}

// ---------------------------------------------------------------------------
// Sedgewick 1986 sequence: 1, 8, 23, 77, 281, ...
//   iter=0: 1
//   iter≥1: 4^iter + 3·2^(iter-1) + 1
// ---------------------------------------------------------------------------
pub struct Sedgewick;
impl GapSequence for Sedgewick {
    const NAME: &'static str = "sedgewick";
    const BIG_O: &'static str = "O(N^(4/3))";

    fn gaps(len: usize) -> Vec<usize> {
        let mut gaps_asc = Vec::new();
        let mut i = 0usize;
        loop {
            let num = if i == 0 {
                1
            } else {
                4_usize.pow(i as u32) + 3 * 2_usize.pow(i as u32 - 1) + 1
            };
            if num >= len {
                break;
            }
            gaps_asc.push(num);
            i += 1;
        }
        gaps_asc.reverse();
        gaps_asc
    }
}

// ---------------------------------------------------------------------------
// Ciura 2001 empirical sequence: 1, 4, 10, 23, 57, 132, 301, 701, ...
// Extended beyond 701 using the ×2.25 approximation.
// ---------------------------------------------------------------------------
pub struct Ciura;
impl GapSequence for Ciura {
    const NAME: &'static str = "ciura";
    const BIG_O: &'static str = "O(N log N) (empirical)";

    fn gaps(len: usize) -> Vec<usize> {
        let mut gaps: Vec<usize> = vec![1, 4, 10, 23, 57, 132, 301, 701];
        loop {
            let last = *gaps.last().unwrap();
            let next = (last * 9 + 2) / 4; // ≈ last × 2.25
            if next <= last || next >= len {
                break;
            }
            gaps.push(next);
        }
        gaps.retain(|&g| g < len);
        gaps.reverse();
        gaps
    }
}
