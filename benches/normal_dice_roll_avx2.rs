use std::convert::Infallible;
use std::hint::black_box;

use collatz_weyl::CollatzWeyl64;
#[cfg(target_arch = "x86_64")]
use collatz_weyl::{CollatzWeyl32Avx2, CollatzWeylAvx2};
use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rand::rngs::StdRng;
use rand::{SeedableRng, TryRng};
use wuerfeln::dice::normal_dice::roll;
#[cfg(target_arch = "x86_64")]
use wuerfeln::dice::normal_dice_avx2::roll as roll_avx2;

/// Number of dice rolled per iteration. Large enough that RNG generation dominates,
/// which is exactly what the vectorized `CollatzWeyl32Avx2` generator targets.
const AMOUNT: usize = 1_000_000;

/// One seed per lane; the generator docs ask for distinct seeds so the eight
/// lanes produce unique streams.
#[cfg(target_arch = "x86_64")]
const AVX2_SEEDS: [u32; 8] = [
	0xD1CE, 0xD1CF, 0xD1D0, 0xD1D1, 0xD1D2, 0xD1D3, 0xD1D4, 0xD1D5,
];

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

/// Same histogram hot path as the `rng_choice` group in `normal_dice_roll`,
/// plus the candidates: the two vectorized Collatz-Weyl AVX2 generators,
/// which have no `rand_core` support. They therefore run through
/// `normal_dice_avx2::roll`, a fork of the roll pipeline that consumes the
/// generators' `__m256i` output in place (register histogram for d1-d8, the
/// scalar bit-packed sampler fed from a lane cache for wider dice).
#[cfg(target_arch = "x86_64")]
fn bench_rng_choice_avx2(c: &mut Criterion) {
	assert!(
		std::arch::is_x86_feature_detected!("avx2"),
		"this benchmark requires AVX2"
	);

	let mut group = c.benchmark_group("normal_dice_roll_avx2");
	group.throughput(Throughput::Elements(AMOUNT as u64));

	for &sides in &[6u8, 20, 100] {
		// Baseline: histogram path, ChaCha12-based StdRng.
		group.bench_with_input(
			BenchmarkId::new("std_chacha", sides),
			&sides,
			|b, &sides| {
				let mut rng = StdRng::seed_from_u64(0xD1CE);
				b.iter(|| black_box(roll(black_box(AMOUNT), black_box(sides), false, &mut rng)));
			},
		);

		// Baseline: histogram path, scalar Collatz-Weyl via its rand-core impl.
		group.bench_with_input(
			BenchmarkId::new("collatz_weyl64", sides),
			&sides,
			|b, &sides| {
				let mut rng = CollatzWeyl64::new(0xD1CE | 1);
				b.iter(|| black_box(roll(black_box(AMOUNT), black_box(sides), false, &mut rng)));
			},
		);

		// Candidate #1: 4-lane u64 AVX2 generator consumed vector-in-place.
		group.bench_with_input(
			BenchmarkId::new("collatz_weyl64_avx2", sides),
			&sides,
			|b, &sides| {
				let mut rng = unsafe {
					// SAFETY: AVX2 presence asserted above.
					CollatzWeylAvx2::new(0xD1CE | 1)
				};
				b.iter(|| black_box(roll_avx2(black_box(AMOUNT), black_box(sides), &mut rng)));
			},
		);

		// Baseline: histogram path, splitmix64 probe (scalar upper bound).
		group.bench_with_input(
			BenchmarkId::new("splitmix64", sides),
			&sides,
			|b, &sides| {
				let mut rng = SplitMix64 { state: 0xD1CE };
				b.iter(|| black_box(roll(black_box(AMOUNT), black_box(sides), false, &mut rng)));
			},
		);

		// Candidate #2: 8-lane u32 AVX2 generator consumed vector-in-place.
		group.bench_with_input(
			BenchmarkId::new("collatz_weyl32_avx2", sides),
			&sides,
			|b, &sides| {
				let mut rng = unsafe {
					// SAFETY: AVX2 presence asserted above.
					CollatzWeyl32Avx2::new(AVX2_SEEDS)
				};
				b.iter(|| black_box(roll_avx2(black_box(AMOUNT), black_box(sides), &mut rng)));
			},
		);
	}

	group.finish();
}

criterion_group! {
	benches,
	bench_rng_choice_avx2,
}
criterion_main!(benches);
