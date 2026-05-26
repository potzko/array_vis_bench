//! Gap-distribution strategies for `RandomShellSort`.

use std::marker::PhantomData;

use rand::Rng;

/// One random gap value in `[0, len)`. Each [`GapDistribution`] impl
/// shapes the gap density curve differently.
pub trait GapDistribution {
    fn sample<R: Rng>(rng: &mut R, len: usize) -> usize;

    /// If true, `RandomShellSort`'s gap-array builder rejects samples
    /// that duplicate values already in the sequence. Lets a base
    /// distribution opt in via the [`Distinct`] wrapper without
    /// touching the consumer's hot path when dedup isn't wanted.
    const DEDUPE: bool = false;
}

/// Uniform distribution: `gap ~ Uniform[0, len)`.
pub struct UniformDist;

impl GapDistribution for UniformDist {
    #[inline]
    fn sample<R: Rng>(rng: &mut R, len: usize) -> usize {
        rng.gen_range(0..len)
    }
}

/// Parabolic distribution: `u ~ Uniform[0,1)`, `gap = floor(u² · len)`.
/// Density is quadratic near 0 — gap values cluster at the small end.
pub struct ParabolicDist;

impl GapDistribution for ParabolicDist {
    #[inline]
    fn sample<R: Rng>(rng: &mut R, len: usize) -> usize {
        let u: f64 = rng.gen();
        ((u * u) * len as f64) as usize
    }
}

/// Cubic distribution: `u ~ Uniform[0,1)`, `gap = floor(u³ · len)`.
/// Density is cubic near 0 — even sharper skew toward small gaps than
/// parabolic, so most samples are tiny and the few large ones reach
/// near `len`.
pub struct CubicDist;

impl GapDistribution for CubicDist {
    #[inline]
    fn sample<R: Rng>(rng: &mut R, len: usize) -> usize {
        let u: f64 = rng.gen();
        ((u * u * u) * len as f64) as usize
    }
}

/// Log-uniform distribution: `u ~ Uniform[0,1)`, `gap = floor(len^u)`.
/// Order statistics are geometrically spaced (each gap is a constant
/// multiplicative factor above its predecessor on average), so small
/// gap values are pulled apart instead of bunching around 1.
pub struct LogUniformDist;

impl GapDistribution for LogUniformDist {
    #[inline]
    fn sample<R: Rng>(rng: &mut R, len: usize) -> usize {
        if len <= 1 {
            return 0;
        }
        let u: f64 = rng.gen();
        (len as f64).powf(u) as usize
    }
}

/// Wrap any [`GapDistribution`] so the gap-array builder rejects
/// duplicate samples. The underlying distribution shape is preserved,
/// but no two slots end up holding the same gap value — useful for
/// dense-near-zero distributions (parabolic, cubic) that would
/// otherwise waste passes on identical gaps.
pub struct Distinct<D: GapDistribution>(PhantomData<D>);

impl<D: GapDistribution> GapDistribution for Distinct<D> {
    #[inline]
    fn sample<R: Rng>(rng: &mut R, len: usize) -> usize {
        D::sample(rng, len)
    }
    const DEDUPE: bool = true;
}
