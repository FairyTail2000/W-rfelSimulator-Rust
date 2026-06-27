use std::convert::Infallible;
use std::hint::black_box;

use collatz_weyl::CollatzWeyl64;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use rand::rngs::StdRng;
use rand::{SeedableRng, TryRng};
use wuerfeln::dice::normal_dice::roll;

/// Number of dice rolled per iteration. Large enough that RNG generation dominates,
/// which is exactly the hot path the `u8` sampling change targets.
const AMOUNT: usize = 1_000_000;

/// splitmix64 — about the fastest u64 PRNG there is (a single add + two mul/xorshift
/// rounds, no buffer, no SIMD). Used purely as a probe: it's a strict upper bound on
/// what any scalar non-crypto generator (Collatz-Weyl, xoshiro, PCG, ...) could buy us
/// versus ChaCha12-based `StdRng`. If this doesn't beat StdRng here, none of them will.
struct SplitMix64 {
    state: u64,
}

// Implement only TryRng<Error = Infallible>; rand_core's blanket then provides Rng.
impl TryRng for SplitMix64 {
    type Error = Infallible;

    #[inline]
    fn try_next_u64(&mut self) -> Result<u64, Self::Error> {
        self.state = self.state.wrapping_add(0x9E3779B97F4A7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        Ok(z ^ (z >> 31))
    }

    #[inline]
    fn try_next_u32(&mut self) -> Result<u32, Self::Error> {
        Ok((self.try_next_u64()? >> 32) as u32)
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), Self::Error> {
        let mut chunks = dest.chunks_exact_mut(8);
        for chunk in &mut chunks {
            chunk.copy_from_slice(&self.try_next_u64()?.to_le_bytes());
        }
        let rem = chunks.into_remainder();
        if !rem.is_empty() {
            let bytes = self.try_next_u64()?.to_le_bytes();
            rem.copy_from_slice(&bytes[..rem.len()]);
        }
        Ok(())
    }
}

fn bench_roll(c: &mut Criterion) {
    let mut group = c.benchmark_group("normal_dice_roll");
    group.throughput(Throughput::Elements(AMOUNT as u64));

    for &sides in &[6u8, 20, 100] {
        // Default path: histogram counting (the optimized hot path).
        group.bench_with_input(BenchmarkId::new("histogram", sides), &sides, |b, &sides| {
            let mut rng = StdRng::seed_from_u64(0xD1CE);
            b.iter(|| black_box(roll(black_box(AMOUNT), black_box(sides), false, &mut rng)));
        });

        // old_style path: stores every individual roll in a Vec<u8>.
        group.bench_with_input(BenchmarkId::new("old_style", sides), &sides, |b, &sides| {
            let mut rng = StdRng::seed_from_u64(0xD1CE);
            b.iter(|| black_box(roll(black_box(AMOUNT), black_box(sides), true, &mut rng)));
        });
    }

    group.finish();
}

/// Probe: same histogram hot path, StdRng (ChaCha12) vs splitmix64. Tells us whether
/// raw RNG cost is actually a meaningful fraction of the per-die budget.
fn bench_rng_choice(c: &mut Criterion) {
    let mut group = c.benchmark_group("rng_choice");
    group.throughput(Throughput::Elements(AMOUNT as u64));

    for &sides in &[6u8, 20, 100] {
        group.bench_with_input(BenchmarkId::new("std_chacha", sides), &sides, |b, &sides| {
            let mut rng = StdRng::seed_from_u64(0xD1CE);
            b.iter(|| black_box(roll(black_box(AMOUNT), black_box(sides), false, &mut rng)));
        });

        group.bench_with_input(BenchmarkId::new("splitmix64", sides), &sides, |b, &sides| {
            let mut rng = SplitMix64 { state: 0xD1CE };
            b.iter(|| black_box(roll(black_box(AMOUNT), black_box(sides), false, &mut rng)));
        });

        // The real candidate, via the crate's own rand-core impl. Seed must be odd
        // (CollatzWeyl64::new asserts it).
        group.bench_with_input(BenchmarkId::new("collatz_weyl", sides), &sides, |b, &sides| {
            let mut rng = CollatzWeyl64::new(0xD1CE | 1);
            b.iter(|| black_box(roll(black_box(AMOUNT), black_box(sides), false, &mut rng)));
        });
    }

    group.finish();
}

criterion_group!(benches, bench_roll, bench_rng_choice);
criterion_main!(benches);
