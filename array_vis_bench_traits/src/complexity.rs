//! Compile-time complexity annotations.
//!
//! A `Complexity` is a const-evaluable struct representing one Big-O class
//! of the shape `O(N^a · (log N)^b · √N? · special?)`. The exponents and
//! flags are small enough to fit in a few bytes; `product` / `sum` are
//! pure `const fn`s, so every per-axis annotation and every compositional
//! impl on `QuickSort` / `BeapSort` / etc. is fully evaluated at compile
//! time.
//!
//! No external dep, no nightly features. The closed set of variants
//! covers every complexity class real sorts use; the `Special` escape
//! hatch handles `√N`, exponential, and factorial.

/// Non-polynomial complexity tags. Multiplying with `Special::None` is
/// the identity; combining two non-`None` specials is treated as
/// saturation (see `Special::product`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Special {
    /// Multiplied by `√N`.
    Sqrt,
    /// `O(2^N)` — dominates everything.
    Exponential,
    /// `O(N!)` — dominates `Exponential`.
    Factorial,
}

impl Special {
    /// Big-O product of two special tags. `Factorial` > `Exponential` >
    /// `Sqrt`; combining picks the dominant tag. The `Option<Special>`
    /// wrapper handles the `None` (no special) identity.
    const fn product(a: Option<Special>, b: Option<Special>) -> Option<Special> {
        match (a, b) {
            (None, x) | (x, None) => x,
            (Some(Special::Factorial), _) | (_, Some(Special::Factorial)) => Some(Special::Factorial),
            (Some(Special::Exponential), _) | (_, Some(Special::Exponential)) => Some(Special::Exponential),
            (Some(Special::Sqrt), Some(Special::Sqrt)) => {
                // √N · √N = N: callers fold this in by bumping n_pow.
                // Returning None here would be wrong because we'd lose
                // the contribution. Instead `Complexity::product` checks
                // for the double-Sqrt case explicitly.
                Some(Special::Sqrt)
            }
        }
    }

    /// Big-O sum (max). Same dominance order as `product`.
    const fn sum(a: Option<Special>, b: Option<Special>) -> Option<Special> {
        match (a, b) {
            (None, x) | (x, None) => x,
            (Some(Special::Factorial), _) | (_, Some(Special::Factorial)) => Some(Special::Factorial),
            (Some(Special::Exponential), _) | (_, Some(Special::Exponential)) => Some(Special::Exponential),
            (Some(Special::Sqrt), Some(Special::Sqrt)) => Some(Special::Sqrt),
        }
    }
}

/// `O(N^n_pow · (log N)^log_pow · special?)`.
///
/// `product`: exponents add, `special` tags combine.
/// `sum`: lex-max on `(special-rank, n_pow, log_pow)` — Big-O sum is
/// `O(max(f, g))`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Complexity {
    pub n_pow: u8,
    pub log_pow: u8,
    pub special: Option<Special>,
}

impl Complexity {
    /// `O(1)` — independent of input size.
    pub const CONST: Self = Self { n_pow: 0, log_pow: 0, special: None };
    /// `O(log N)`.
    pub const LOG_N: Self = Self { n_pow: 0, log_pow: 1, special: None };
    /// `O(log² N)`.
    pub const LOG_SQUARED: Self = Self { n_pow: 0, log_pow: 2, special: None };
    /// `O(√N)`.
    pub const SQRT_N: Self = Self { n_pow: 0, log_pow: 0, special: Some(Special::Sqrt) };
    /// `O(N)` — linear.
    pub const N1: Self = Self { n_pow: 1, log_pow: 0, special: None };
    /// `O(N log N)` — the canonical comparison-sort lower bound.
    pub const N_LOG_N: Self = Self { n_pow: 1, log_pow: 1, special: None };
    /// `O(N log² N)`.
    pub const N_LOG_SQUARED: Self = Self { n_pow: 1, log_pow: 2, special: None };
    /// `O(N √N)`.
    pub const N_SQRT_N: Self = Self { n_pow: 1, log_pow: 0, special: Some(Special::Sqrt) };
    /// `O(N²)` — naive comparison sorts in the worst case.
    pub const N_SQUARED: Self = Self { n_pow: 2, log_pow: 0, special: None };
    /// `O(N² log N)`.
    pub const N_SQUARED_LOG_N: Self = Self { n_pow: 2, log_pow: 1, special: None };
    /// `O(N³)`.
    pub const CUBIC: Self = Self { n_pow: 3, log_pow: 0, special: None };
    /// `O(2^N)` — slow sort.
    pub const EXPONENTIAL: Self = Self { n_pow: 0, log_pow: 0, special: Some(Special::Exponential) };
    /// `O(N!)` — bogosort.
    pub const FACTORIAL: Self = Self { n_pow: 0, log_pow: 0, special: Some(Special::Factorial) };

    /// `O(f) · O(g)` — exponents add, special tags combine. `√N · √N`
    /// collapses to one factor of `N`.
    pub const fn product(a: Self, b: Self) -> Self {
        let double_sqrt = matches!(a.special, Some(Special::Sqrt))
            && matches!(b.special, Some(Special::Sqrt));
        Self {
            n_pow: a.n_pow + b.n_pow + if double_sqrt { 1 } else { 0 },
            log_pow: a.log_pow + b.log_pow,
            special: if double_sqrt { None } else { Special::product(a.special, b.special) },
        }
    }

    /// `O(f) + O(g) = O(max(f, g))`. Treats `Sqrt` as half an `n_pow`
    /// step (so `N √N` ranks above `N log N` and above `N log² N`,
    /// matching asymptotics), then breaks ties by `log_pow`.
    pub const fn sum(a: Self, b: Self) -> Self {
        match (a.special, b.special) {
            (Some(Special::Factorial), _) => a,
            (_, Some(Special::Factorial)) => b,
            (Some(Special::Exponential), _) => a,
            (_, Some(Special::Exponential)) => b,
            _ => {
                // Effective n-exponent in halves: `2*n_pow + sqrt_bit`.
                // Sqrt only ever appears on one side at a tie (if both
                // had Sqrt at the same n_pow they'd be equal here), so
                // the log_pow tiebreak below only fires when sqrt
                // status matches.
                let a_eff = 2 * a.n_pow + sqrt_bit(a.special);
                let b_eff = 2 * b.n_pow + sqrt_bit(b.special);
                if a_eff > b_eff {
                    a
                } else if b_eff > a_eff {
                    b
                } else if a.log_pow > b.log_pow {
                    a
                } else if b.log_pow > a.log_pow {
                    b
                } else {
                    Self {
                        n_pow: a.n_pow,
                        log_pow: a.log_pow,
                        special: Special::sum(a.special, b.special),
                    }
                }
            }
        }
    }

    /// "In-place" = no allocation that grows with N. Equivalent to
    /// `n_pow == 0 && special != Some(Sqrt | Exponential | Factorial)`.
    /// `log_pow ≥ 1` (recursion-stack depth) is fine.
    pub const fn is_in_place(self) -> bool {
        self.n_pow == 0 && self.special.is_none()
    }

    /// Display as `"O(...)"`. Returns a `&'static str` so it can feed
    /// `AlgorithmEntry`'s string-typed field without allocation.
    pub const fn as_str(self) -> &'static str {
        match (self.n_pow, self.log_pow, self.special) {
            (0, 0, None) => "O(1)",
            (0, 1, None) => "O(log N)",
            (0, 2, None) => "O(log² N)",
            (0, 0, Some(Special::Sqrt)) => "O(√N)",
            (1, 0, None) => "O(N)",
            (1, 1, None) => "O(N log N)",
            (1, 2, None) => "O(N log² N)",
            (1, 0, Some(Special::Sqrt)) => "O(N √N)",
            (2, 0, None) => "O(N²)",
            (2, 0, Some(Special::Sqrt)) => "O(N^2.5)",
            (2, 1, None) => "O(N² log N)",
            (3, 0, None) => "O(N³)",
            (0, 0, Some(Special::Exponential)) => "O(2^N)",
            (0, 0, Some(Special::Factorial)) => "O(N!)",
            _ => "O(?)",
        }
    }

    /// Parse a legacy Big-O string literal that appeared in a `family!`
    /// invocation before the type-level annotation system existed.
    /// Covers every variant currently used in the codebase. Panics at
    /// compile time on an unknown input — the call site is `const`, so
    /// the panic surfaces as a build error pointing at the call.
    pub const fn from_str(s: &str) -> Self {
        if str_eq(s, "O(1)") || str_eq(s, "O(K)") {
            // O(K) appears on bounded-leaf small-sorts; treated as O(1)
            // because K is a compile-time constant in those contexts.
            Self::CONST
        } else if str_eq(s, "O(log N)") {
            Self::LOG_N
        } else if str_eq(s, "O(log² N)") {
            Self::LOG_SQUARED
        } else if str_eq(s, "O(N)") {
            Self::N1
        } else if str_eq(s, "O(N log N)") || str_eq(s, "O(N Log(N))") || str_eq(s, "O(N log(N))") {
            Self::N_LOG_N
        } else if str_eq(s, "O(N log² N)") {
            Self::N_LOG_SQUARED
        } else if str_eq(s, "O(N sqrt(N))")
            || str_eq(s, "O(N^1.5)")
            || str_eq(s, "O(N^(3/2))")
        {
            Self::N_SQRT_N
        } else if str_eq(s, "O(N^(4/3))") || str_eq(s, "O(N^(4/3)) (empirical)") {
            // Sedgewick / Ciura / Optimized256 shell sort: empirically
            // N^1.33. Closest bucket below N^1.5; round up to N_SQRT_N
            // for a conservative (correct) Big-O upper bound.
            Self::N_SQRT_N
        } else if str_eq(s, "O(N log N) (empirical)") {
            Self::N_LOG_N
        } else if str_eq(s, "O(N²)") || str_eq(s, "O(N^2)") {
            Self::N_SQUARED
        } else if str_eq(s, "O(N^2.5)") {
            // N² · √N — captured exactly in the struct.
            Self { n_pow: 2, log_pow: 0, special: Some(Special::Sqrt) }
        } else if str_eq(s, "O(N^2.71)") {
            // Stooge sort: log₃ / log₁.₅ ≈ 2.71. Approximate as N^2.5
            // bucket — closer than N² or N³, and we already represent it.
            Self { n_pow: 2, log_pow: 0, special: Some(Special::Sqrt) }
        } else if str_eq(s, "O(N³)") || str_eq(s, "O(N^3)") || str_eq(s, "O(N^3?)") {
            Self::CUBIC
        } else if str_eq(s, "O(N^?)") {
            // Bad heap sort: empirically ~N² but unanalysed. Bucket as N².
            Self::N_SQUARED
        } else if str_eq(s, "O(N^logN)") {
            // Slow-sort family: super-polynomial, treat as exponential.
            Self::EXPONENTIAL
        } else if str_eq(s, "O(2^N)") {
            Self::EXPONENTIAL
        } else if str_eq(s, "O(N!)") {
            Self::FACTORIAL
        } else {
            panic!("Complexity::from_str: unrecognized complexity string")
        }
    }
}

/// 1 when the special tag contributes a `√N` factor, 0 otherwise.
/// Used by `Complexity::sum` to compare `(n_pow, sqrt)` as a unified
/// effective exponent (in halves).
const fn sqrt_bit(s: Option<Special>) -> u8 {
    match s {
        Some(Special::Sqrt) => 1,
        _ => 0,
    }
}

/// Byte-wise const string equality. `str::eq` isn't const, hence the
/// loop. Kept private since the public API is `Complexity::from_str`.
const fn str_eq(a: &str, b: &str) -> bool {
    let ab = a.as_bytes();
    let bb = b.as_bytes();
    if ab.len() != bb.len() {
        return false;
    }
    let mut i = 0;
    while i < ab.len() {
        if ab[i] != bb[i] {
            return false;
        }
        i += 1;
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn product_basics() {
        assert_eq!(Complexity::product(Complexity::N1, Complexity::LOG_N), Complexity::N_LOG_N);
        assert_eq!(Complexity::product(Complexity::CONST, Complexity::N_LOG_N), Complexity::N_LOG_N);
        assert_eq!(Complexity::product(Complexity::N1, Complexity::N1), Complexity::N_SQUARED);
        assert_eq!(Complexity::product(Complexity::SQRT_N, Complexity::SQRT_N), Complexity::N1);
    }

    #[test]
    fn sum_basics() {
        assert_eq!(Complexity::sum(Complexity::LOG_N, Complexity::N1), Complexity::N1);
        assert_eq!(Complexity::sum(Complexity::N1, Complexity::N_LOG_N), Complexity::N_LOG_N);
        assert_eq!(Complexity::sum(Complexity::CONST, Complexity::CONST), Complexity::CONST);
        assert_eq!(Complexity::sum(Complexity::N_SQUARED, Complexity::N_LOG_N), Complexity::N_SQUARED);
    }

    #[test]
    fn sum_sqrt_ranking() {
        // N √N (= N^1.5) dominates any pure log factor at the same n_pow.
        assert_eq!(
            Complexity::sum(Complexity::N_SQRT_N, Complexity::N_LOG_N),
            Complexity::N_SQRT_N,
        );
        assert_eq!(
            Complexity::sum(Complexity::N_SQRT_N, Complexity::N_LOG_SQUARED),
            Complexity::N_SQRT_N,
        );
        // Same one tier down.
        assert_eq!(
            Complexity::sum(Complexity::SQRT_N, Complexity::LOG_N),
            Complexity::SQRT_N,
        );
        assert_eq!(
            Complexity::sum(Complexity::SQRT_N, Complexity::LOG_SQUARED),
            Complexity::SQRT_N,
        );
        // But the next polynomial tier still wins over Sqrt.
        assert_eq!(
            Complexity::sum(Complexity::N_SQUARED, Complexity::N_SQRT_N),
            Complexity::N_SQUARED,
        );
        assert_eq!(
            Complexity::sum(Complexity::N1, Complexity::SQRT_N),
            Complexity::N1,
        );
    }

    #[test]
    fn in_place() {
        assert!(Complexity::CONST.is_in_place());
        assert!(Complexity::LOG_N.is_in_place());
        assert!(Complexity::LOG_SQUARED.is_in_place());
        assert!(!Complexity::N1.is_in_place());
        assert!(!Complexity::SQRT_N.is_in_place());
    }

    #[test]
    fn display() {
        assert_eq!(Complexity::N_LOG_N.as_str(), "O(N log N)");
        assert_eq!(Complexity::N_SQUARED.as_str(), "O(N²)");
        assert_eq!(Complexity::FACTORIAL.as_str(), "O(N!)");
    }

    #[test]
    fn from_str_round_trip() {
        // Every legacy string in use parses back to its expected class.
        assert_eq!(Complexity::from_str("O(1)"), Complexity::CONST);
        assert_eq!(Complexity::from_str("O(K)"), Complexity::CONST);
        assert_eq!(Complexity::from_str("O(N)"), Complexity::N1);
        assert_eq!(Complexity::from_str("O(N log N)"), Complexity::N_LOG_N);
        assert_eq!(Complexity::from_str("O(N Log(N))"), Complexity::N_LOG_N);
        assert_eq!(Complexity::from_str("O(N²)"), Complexity::N_SQUARED);
        assert_eq!(Complexity::from_str("O(N^2)"), Complexity::N_SQUARED);
        assert_eq!(Complexity::from_str("O(N^2.5)").n_pow, 2);
        assert_eq!(Complexity::from_str("O(N^logN)"), Complexity::EXPONENTIAL);
    }

    #[test]
    fn from_str_evaluates_in_const_context() {
        // Forces compile-time evaluation — the const initializer would
        // panic if the parser misbehaved.
        const _: Complexity = Complexity::from_str("O(N log N)");
        const _: Complexity = Complexity::from_str("O(N²)");
    }
}
