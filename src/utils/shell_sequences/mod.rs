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

// ---------------------------------------------------------------------------
// Sedgewick 1982 branching sequence: 1, 5, 19, 41, 109, 209, 505, ...
//   iter even: 9 × (2^iter - 2^(iter/2)) + 1
//   iter odd:  8 × 2^iter - 6 × 2^((iter+1)/2) + 1
// ---------------------------------------------------------------------------
pub struct SedgewickBranching;
impl GapSequence for SedgewickBranching {
    const NAME: &'static str = "sedgewick-branching";
    const BIG_O: &'static str = "O(N^(4/3))";

    fn gaps(len: usize) -> Vec<usize> {
        let mut gaps_asc = Vec::new();
        let mut i = 0usize;
        loop {
            let num = if i % 2 == 1 {
                8 * 2_usize.pow(i as u32) - 6 * 2_usize.pow((i as u32 + 1) / 2) + 1
            } else {
                9 * (2_usize.pow(i as u32) - 2_usize.pow(i as u32 / 2)) + 1
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
// Optimized sequence for arrays up to 256 elements: 84, 25, 1
// Pre-computed optimal gaps for small arrays.
// ---------------------------------------------------------------------------
pub struct Optimized256;
impl GapSequence for Optimized256 {
    const NAME: &'static str = "optimized-256";
    const BIG_O: &'static str = "O(N^1.5)";

    fn gaps(len: usize) -> Vec<usize> {
        let all_gaps = vec![84, 25, 1];
        all_gaps.into_iter().filter(|&g| g < len).collect()
    }
}

// ---------------------------------------------------------------------------
// Tokuda sequence: 1, 4, 9, 20, 46, 103, 233, 525, 1182, ...
//   h(k) = ceil((9 × (9/4)^k - 4) / 5)
// Empirically performs very well in practice.
// ---------------------------------------------------------------------------
pub struct Tokuda;
impl GapSequence for Tokuda {
    const NAME: &'static str = "tokuda";
    const BIG_O: &'static str = "O(N^(4/3)) (empirical)";

    fn gaps(len: usize) -> Vec<usize> {
        let mut gaps_asc = Vec::new();
        let mut k = 0;
        loop {
            let frac = (9.0 / 4.0_f64).powi(k);
            let gap = ((9.0 * frac - 4.0) / 5.0).ceil() as usize;
            if gap >= len {
                break;
            }
            gaps_asc.push(gap);
            k += 1;
        }
        gaps_asc.reverse();
        gaps_asc
    }
}

// ---------------------------------------------------------------------------
// Pratt sequence: 1, 2, 3, 4, 6, 8, 9, 12, 16, 18, 24, 27, 32, ...
// All numbers of the form 2^p × 3^q (products of powers of 2 and 3).
// Proven to achieve O(N log² N) comparisons, which is optimal for Shell sort.
// ---------------------------------------------------------------------------
pub struct Pratt;
impl GapSequence for Pratt {
    const NAME: &'static str = "pratt";
    const BIG_O: &'static str = "O(N log² N)";

    fn gaps(len: usize) -> Vec<usize> {
        let mut gaps_asc = Vec::new();
        
        // Generate all 2^p × 3^q < len
        let mut pow2 = 1usize;
        while pow2 < len {
            let mut pow23 = pow2;
            while pow23 < len {
                gaps_asc.push(pow23);
                // Check for overflow before multiplying
                if pow23 > len / 3 {
                    break;
                }
                pow23 *= 3;
            }
            // Check for overflow before multiplying
            if pow2 > len / 2 {
                break;
            }
            pow2 *= 2;
        }
        
        gaps_asc.sort_unstable();
        gaps_asc.dedup();
        gaps_asc.reverse();
        gaps_asc
    }
}
