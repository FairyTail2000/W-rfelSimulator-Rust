//! AVX2-native fork of [`super::normal_dice`] hard-wired to the vectorized
//! `CollatzWeyl` generators (`CollatzWeyl32Avx2` and `CollatzWeylAvx2`),
//! which have no `rand_core` support: their `next` hands out a full `__m256i`
//! holding fresh words (8 x u32 / 4 x u64 respectively).
//!
//! Routing those vectors through the `Rng` trait would serialize them into
//! bytes only for the sampler to unpack them again, so this module consumes
//! the vectors where they are produced:
//!
//! * the histogram fast path for small dice (sides <= 8) feeds four `next`
//!   vectors straight into the register-histogram kernel — the generator
//!   output never touches memory;
//! * the remaining shapes (and the kernel tail) run the scalar bit-packed
//!   sampler off a small u32 lane cache.
//!
//! Requires AVX2 and x86_64. The caller is responsible for runtime detection;
//! any soundly constructed generator guarantees it (the `new` constructors
//! are `unsafe` for exactly that reason).
//!
//! Deliberately drops the `old_style` per-roll `Vec<u8>` output and the
//! `Rng`-generic interface of the original: this module exists to measure
//! whether the vectorized generator pays off on the histogram hot path.

// Not wired into the CLI yet; only the benchmark consumes it, and the
// binary's module tree shares this file.
#![allow(dead_code)]

use collatz_weyl::{CollatzWeyl32Avx2, CollatzWeylAvx2};
use core::arch::x86_64::*;

/// u32 words handed out by one generator vector.
const WORDS_PER_VECTOR: usize = 8;
/// Generator vectors consumed per histogram-kernel call.
const KERNEL_VECTORS: usize = 4;

/// Common entropy interface of the two AVX2 generators: one `next` call
/// yields a full `__m256i` of fresh words — 8 x u32 for `CollatzWeyl32Avx2`,
/// 4 x u64 for `CollatzWeylAvx2`. The sampler only cares about the 32 bytes,
/// whose u32-half layout is identical either way.
pub trait Avx2Generator {
	/// # Safety
	/// AVX2 must be available on the runtime CPU.
	unsafe fn next_vector(&mut self) -> __m256i;
}

impl Avx2Generator for CollatzWeyl32Avx2 {
	#[inline]
	unsafe fn next_vector(&mut self) -> __m256i {
		unsafe { self.next() }
	}
}

impl Avx2Generator for CollatzWeylAvx2 {
	#[inline]
	unsafe fn next_vector(&mut self) -> __m256i {
		unsafe { self.next() }
	}
}

/**
Contains the information of one histogram roll.
 */
pub struct Results {
	pub counts: Vec<u64>,
	sides: u8,
	count: u64,
}

impl Results {
	pub const fn sides(&self) -> u8 {
		self.sides
	}

	pub const fn count(&self) -> u64 {
		self.count
	}
}

/// Upper bound on dice produced by one kernel call:
/// `KERNEL_VECTORS` x `WORDS_PER_VECTOR` x `32 / B` chunks per lane.
const fn chunks_per_iter<const B: u32>() -> usize {
	KERNEL_VECTORS * WORDS_PER_VECTOR * (32 / B as usize)
}

/// Draws one vector from `rng` and stores its eight u32 lanes.
///
/// # Safety
/// AVX2 must be available on the runtime CPU.
#[inline]
#[target_feature(enable = "avx2")]
unsafe fn next_words<G: Avx2Generator>(rng: &mut G) -> [u32; WORDS_PER_VECTOR] {
	let mut words = [0u32; WORDS_PER_VECTOR];
	unsafe {
		_mm256_storeu_si256(words.as_mut_ptr().cast(), rng.next_vector());
	}
	words
}

/// Serves the scalar sampler from the generator's u32 lanes: `next` produces
/// eight words at once, so a drained cache refills with one vector and the
/// sampler keeps pulling u64 pairs out of it.
struct LaneCache {
	words: [u32; WORDS_PER_VECTOR],
	index: usize,
}

impl LaneCache {
	fn new() -> Self {
		Self {
			words: [0; WORDS_PER_VECTOR],
			index: WORDS_PER_VECTOR,
		}
	}

	/// Combines the next two cached lanes into a u64 word, refilling from the
	/// generator when the vector is drained. `index` is always even, so the
	/// lane pair read below is always in bounds.
	///
	/// # Safety
	/// AVX2 must be available on the runtime CPU.
	#[inline]
	unsafe fn next_u64<G: Avx2Generator>(&mut self, rng: &mut G) -> u64 {
		if self.index == WORDS_PER_VECTOR {
			self.words = unsafe { next_words(rng) };
			self.index = 0;
		}
		let lo = u64::from(self.words[self.index]);
		let hi = u64::from(self.words[self.index + 1]);
		self.index += 2;
		lo | (hi << 32)
	}
}

/// Feeds `amount` die results in `1..=sides` to `emit`, drawing entropy in
/// bit-packed chunks from u64 words instead of spending one word per die.
///
/// Same chunking scheme as [`super::normal_dice`]: `B` is the smallest number
/// of randomness bits per die with `2^B >= sides`, each word is split into
/// `64 / B` chunks, and a chunk is rejected when it is `>= sides`, which
/// removes modulo bias. An accepted chunk already *is* the zero-based face,
/// and rejection stays branch-free (masked face instead of an `if`).
#[inline]
fn for_each_die<const B: u32, F: FnMut(u8) -> bool>(
	amount: usize,
	sides: u8,
	mut next_word: impl FnMut() -> u64,
	mut emit: F,
) {
	debug_assert!(u32::from(sides) <= (1 << B));
	let sides32 = u32::from(sides);
	let mask: u64 = (1 << B) - 1;
	let mut produced = 0usize;
	'words: while produced < amount {
		let mut word = next_word();
		// `64 / B` is a compile-time constant per instantiation, so LLVM fully
		// unrolls this loop (21 iterations for a d6, 12 for a d20).
		for _ in 0..(64 / B) {
			let chunk = (word & mask) as u32;
			word >>= B;
			let accepted = chunk < sides32;
			// Branch-free face: `chunk + 1` when accepted, `0` (the scratch
			// slot) otherwise. `chunk + 1` only overflows a u8 for chunk 255,
			// which is rejected for every valid `sides` and masked to 0 anyway.
			let face = ((chunk + 1) as u8) & (accepted as u8).wrapping_neg();
			produced += emit(face) as usize;
			if produced >= amount {
				break 'words;
			}
		}
	}
}

/// Calls `for_each_die::<B>` with the smallest bit width that can hold `sides`
/// faces, so every die type gets an unrolled, modulo-free sampling path.
macro_rules! dispatch_dice {
	($sides:expr, $amount:expr, $next_word:expr, $emit:expr) => {
		match $sides {
			1 | 2 => for_each_die::<1, _>($amount, $sides, $next_word, $emit),
			3 | 4 => for_each_die::<2, _>($amount, $sides, $next_word, $emit),
			5..=8 => for_each_die::<3, _>($amount, $sides, $next_word, $emit),
			9..=16 => for_each_die::<4, _>($amount, $sides, $next_word, $emit),
			17..=32 => for_each_die::<5, _>($amount, $sides, $next_word, $emit),
			33..=64 => for_each_die::<6, _>($amount, $sides, $next_word, $emit),
			65..=128 => for_each_die::<7, _>($amount, $sides, $next_word, $emit),
			_ => for_each_die::<8, _>($amount, $sides, $next_word, $emit),
		}
	};
}

/// Hand-rolled AVX2 fast path for the histogram roll of dice with at most 8
/// sides (entropy width B <= 3).
///
/// Same peel/pack/count pipeline as the kernel in [`super::normal_dice`]
/// (`_mm256_srl_epi32` layers, `vpackusdw`/`vpackuswb` condensation,
/// `vpcmpeqb` + `vpmovmskb` + `popcnt` per-face counting with rejection kept
/// branch-free), but the entropy arrives in registers straight from four
/// generator vectors — no buffer, no loads, and the RNG itself runs 8 lanes
/// wide instead of as a serial scalar chain.
///
/// # Safety
/// AVX2 must be available on the runtime CPU.
#[target_feature(enable = "avx2")]
unsafe fn histogram_vectors<const B: u32>(
	vectors: [__m256i; KERNEL_VECTORS],
	sides: u32,
	counts: &mut [u64],
) -> usize {
	unsafe {
		let chunk_mask = _mm256_set1_epi32(((1u32 << B) - 1) as i32);
		let face_vecs = [
			_mm256_set1_epi8(0),
			_mm256_set1_epi8(1),
			_mm256_set1_epi8(2),
			_mm256_set1_epi8(3),
			_mm256_set1_epi8(4),
			_mm256_set1_epi8(5),
			_mm256_set1_epi8(6),
			_mm256_set1_epi8(7),
		];
		let [a, b, c, d] = vectors;
		let mut produced = 0usize;
		// Face totals are folded into `counts` only once per call: updating
		// the table inside the hot loop would serialize on store-forwarding
		// chains per histogram slot.
		let mut face_hits = [0u64; 8];
		// Peel two B-bit layers per round so the byte packing has both
		// operands; 32/B layers x 2 register pairs is always even.
		let mut prev: Option<(__m256i, __m256i)> = None;
		for step in 0..(32 / B) {
			let count = _mm_cvtsi32_si128(step as i32 * B as i32);
			for (x, y) in [(a, b), (c, d)] {
				let v0 = _mm256_and_si256(_mm256_srl_epi32(x, count), chunk_mask);
				let v1 = _mm256_and_si256(_mm256_srl_epi32(y, count), chunk_mask);
				match prev.take() {
					Some((w0, w1)) => {
						let bytes = _mm256_packus_epi16(
							_mm256_packus_epi32(w0, w1),
							_mm256_packus_epi32(v0, v1),
						);
						for face in 0..sides {
							let eq = _mm256_cmpeq_epi8(bytes, face_vecs[face as usize]);
							face_hits[face as usize] +=
								_mm256_movemask_epi8(eq).count_ones() as u64;
						}
					}
					None => prev = Some((v0, v1)),
				}
			}
		}
		// 32/B layers x 2 register pairs is even, so no layer is left unpaired.
		debug_assert!(prev.is_none());
		for (face, &hits) in face_hits[..sides as usize].iter().enumerate() {
			// face is a chunk value 0..sides, so face + 1 indexes
			// 1..=sides of the (sides + 1)-long table.
			*counts.get_unchecked_mut(face + 1) += hits;
			produced += hits as usize;
		}
		produced
	}
}

/// Runs histogram-kernel iterations until fewer than a full kernel call's
/// worth of dice remain; the caller drains the rest through the scalar
/// sampler. Since an iteration produces at most `chunks_per_iter::<B>()`
/// accepted dice, the loop can never overshoot `amount`.
fn roll_small_sides<const B: u32, G: Avx2Generator>(
	amount: usize,
	sides: u8,
	rng: &mut G,
	counts: &mut [u64],
) -> usize {
	debug_assert!(u32::from(sides) <= 1 << B);
	debug_assert!((1..=8).contains(&sides));
	let mut produced = 0usize;
	// Calling `next` outside an AVX2-enabled context needs an unsafe block;
	// sound because any constructed generator guarantees AVX2 availability.
	while amount - produced >= chunks_per_iter::<B>() {
		let vectors = [
			unsafe { rng.next_vector() },
			unsafe { rng.next_vector() },
			unsafe { rng.next_vector() },
			unsafe { rng.next_vector() },
		];
		produced += unsafe { histogram_vectors::<B>(vectors, u32::from(sides), counts) };
	}
	produced
}

/// Histogram roll of `amount` dice with faces `1..=sides`, counted into a
/// 1-indexed table (index 0 is an unused scratch slot, kept for parity with
/// [`super::normal_dice`]).
pub fn roll<G: Avx2Generator>(amount: usize, sides: u8, rng: &mut G) -> Results {
	assert!(sides != 0, "sides must be at least 1");
	// Pre-allocate space for each side (1-indexed for convenience).
	// `sides as usize + 1` cannot wrap, unlike `sides + 1` for sides = 255.
	let mut counts = vec![0u64; sides as usize + 1];
	// The register-histogram kernel only pays off for small dice where the
	// per-face compares beat scalar table increments; everything else runs
	// through the scalar bit-packed sampler.
	let mut produced = if sides <= 8 {
		match sides {
			1..=2 => roll_small_sides::<1, G>(amount, sides, rng, &mut counts),
			3..=4 => roll_small_sides::<2, G>(amount, sides, rng, &mut counts),
			_ => roll_small_sides::<3, G>(amount, sides, rng, &mut counts),
		}
	} else {
		0
	};
	let tail = amount - produced;
	if tail > 0 {
		let mut cache = LaneCache::new();
		let next_word = || unsafe { cache.next_u64(rng) };
		// Branch-free sink: rejected chunks bump the unused scratch slot at
		// index 0 instead of taking a mispredict-heavy branch; it is zeroed
		// again once rolling is done.
		let emit = |face: u8| {
			// The sampler only produces faces in 0..=sides, so indexing a
			// (sides + 1)-long table is always in bounds.
			unsafe {
				*counts.get_unchecked_mut(face as usize) += 1;
			}
			face != 0
		};
		dispatch_dice!(sides, tail, next_word, emit);
		produced += tail;
	}
	debug_assert_eq!(produced, amount);
	counts[0] = 0;
	Results {
		counts,
		sides,
		count: amount as u64,
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	const SEEDS: [u32; 8] = [
		0xD1CE, 0xD1CF, 0xD1D0, 0xD1D1, 0xD1D2, 0xD1D3, 0xD1D4, 0xD1D5,
	];

	/// `None` on hosts without AVX2, where every test then silently skips.
	fn make_rng32() -> Option<CollatzWeyl32Avx2> {
		if std::arch::is_x86_feature_detected!("avx2") {
			// SAFETY: feature presence just checked.
			Some(unsafe { CollatzWeyl32Avx2::new(SEEDS) })
		} else {
			None
		}
	}

	/// `None` on hosts without AVX2, where every test then silently skips.
	fn make_rng64() -> Option<CollatzWeylAvx2> {
		if std::arch::is_x86_feature_detected!("avx2") {
			// SAFETY: feature presence just checked.
			Some(unsafe { CollatzWeylAvx2::new(0xD1CE | 1) })
		} else {
			None
		}
	}

	fn histogram_totals<G: Avx2Generator>(rng: &mut G) {
		for sides in [1u8, 2, 3, 6, 7, 8, 20, 100, 255] {
			let results = roll(100_000, sides, rng);
			assert_eq!(results.counts.len(), sides as usize + 1);
			assert_eq!(results.counts[0], 0, "index 0 is the unused slot");
			assert_eq!(results.counts[1..].iter().sum::<u64>(), 100_000);
		}
	}

	#[test]
	fn histogram_totals_and_layout() {
		if let Some(mut rng) = make_rng32() {
			histogram_totals(&mut rng);
		}
		if let Some(mut rng) = make_rng64() {
			histogram_totals(&mut rng);
		}
	}

	fn uniform_faces<G: Avx2Generator>(rng: &mut G) {
		// Chi-square-style sanity check: every face count must stay within 5 sigma
		// of its expectation. The seed is fixed, so the bound is deterministic.
		for sides in [2u8, 3, 6, 7, 20, 100, 255] {
			let rolls: u64 = 600_000;
			let results = roll(rolls as usize, sides, rng);
			let expected = rolls as f64 / f64::from(sides);
			let sigma = (expected * (1.0 - 1.0 / f64::from(sides))).sqrt();
			for &count in &results.counts[1..] {
				let deviation = (count as f64 - expected).abs();
				assert!(
					deviation < 5.0 * sigma,
					"sides {sides}: count {count}, expected {expected:.1}, deviation {deviation:.1}, 5*sigma {:.1}",
					5.0 * sigma
				);
			}
		}
	}

	#[test]
	fn faces_are_uniform() {
		if let Some(mut rng) = make_rng32() {
			uniform_faces(&mut rng);
		}
		if let Some(mut rng) = make_rng64() {
			uniform_faces(&mut rng);
		}
	}

	#[test]
	fn zero_amount_rolls_nothing() {
		if let Some(mut rng) = make_rng32() {
			let results = roll(0, 6, &mut rng);
			assert_eq!(results.counts.iter().sum::<u64>(), 0);
		}
		if let Some(mut rng) = make_rng64() {
			let results = roll(0, 6, &mut rng);
			assert_eq!(results.counts.iter().sum::<u64>(), 0);
		}
	}

	#[test]
	#[should_panic(expected = "sides must be at least 1")]
	fn zero_sides_panics() {
		let mut rng = make_rng32().expect("AVX2 required for this test");
		roll(1, 0, &mut rng);
	}

	fn same_seed_histograms<G: Avx2Generator>(seeded: impl Fn() -> G) {
		// Both calls take the same code path (kernel + scalar tail in the
		// same proportions), so identical inputs must yield identical
		// histograms. 4097 also exercises a kernel/tail boundary.
		for sides in [1u8, 2, 3, 4, 6, 8, 20, 100] {
			let first = roll(40_097, sides, &mut seeded());
			let second = roll(40_097, sides, &mut seeded());
			assert_eq!(first.counts, second.counts, "sides {sides}");
		}
	}

	#[test]
	fn same_seed_same_histogram() {
		if !std::arch::is_x86_feature_detected!("avx2") {
			return;
		}
		same_seed_histograms(|| make_rng32().unwrap());
		same_seed_histograms(|| make_rng64().unwrap());
	}

	fn kernel_boundaries<G: Avx2Generator>(rng: &mut G) {
		// Around kernel-iteration boundaries every amount must produce
		// exactly that many counted dice, whatever the kernel/tail split.
		for sides in [1u8, 2, 3, 4, 5, 6, 7, 8] {
			for amount in [1023usize, 1024, 1025, 4095, 4096, 4097] {
				let results = roll(amount, sides, rng);
				assert_eq!(results.counts[1..].iter().sum::<u64>(), amount as u64);
				assert_eq!(results.counts[0], 0, "scratch slot must stay zeroed");
			}
		}
	}

	#[test]
	fn kernel_boundary_amounts_are_exact() {
		if let Some(mut rng) = make_rng32() {
			kernel_boundaries(&mut rng);
		}
		if let Some(mut rng) = make_rng64() {
			kernel_boundaries(&mut rng);
		}
	}
}
