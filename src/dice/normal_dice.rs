use crate::common::{Loadable, settings_path};
use crate::dbgprintln;
use ansi_term::Colour;
use rand::Rng;
use rand::distr::Uniform;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter};
use std::path::Path;
use std::vec::Vec;

const NORMAL_DICES_FILE: &str = "normal.yaml";

/**
Contains the information of one result
 */
pub struct Results {
	counts: Option<Vec<u64>>,
	data: Option<Vec<u8>>,
	sides: u8,
	count: u64,
}

fn calculate_variance(counts: &[u64], total_samples: u32, sides: u32) -> f64 {
	let mean = total_samples as f64 / sides as f64;
	let variance: f64 = counts
		.iter()
		.map(|&count| {
			let diff = count as f64 - mean;
			diff * diff
		})
		.sum::<f64>()
		/ sides as f64;
	variance
}

impl Results {
	pub fn print_results(&self, old_style: bool, no_summary: bool) {
		println!("\n");
		if let Some(counts) = &self.counts {
			if self.sides == 6 {
				dbgprintln!("Misserfolge: {}", counts[1]); // 1
				dbgprintln!("Misserfolge (improvisert): {}", counts[1] + counts[2]); // 1 + 2
				dbgprintln!(
					"Misserfolge (Pechphiole): {}",
					counts[1] + counts[2] + counts[3]
				); // 1 + 2 + 3
				dbgprintln!(
					"Erfolge (Wealthphiole): {}",
					counts[3] + counts[4] + counts[5] + counts[6]
				); // 3 + 4 + 5 + 6
				dbgprintln!(
					"Erfolge (Glücksphiole): {}",
					counts[4] + counts[5] + counts[6]
				); // 4 + 5 + 6
				dbgprintln!("Erfolge: {}", counts[5] + counts[6]); // 5 + 6
			} else {
				// counts[0] is the unused scratch slot of the 1-indexed histogram
				let sum: u64 = counts[1..]
					.iter()
					.enumerate()
					.map(|(index, &count)| (index + 1) as u64 * count)
					.sum();

				if old_style {
					for (index, result) in counts[1..].iter().enumerate() {
						if *result != 0 {
							dbgprintln!("Augenzahl: {}\tErgebnis: {}", index + 1, result);
						}
					}
					println!();
				}
				dbgprintln!("Summe: {}", sum);
			}
		} else if let Some(data) = &self.data {
			if self.sides == 6 {
				let mut accumulated: [u64; 6] = [0, 0, 0, 0, 0, 0];
				for result in data {
					debug_assert!(*result <= 6);
					accumulated[(*result - 1) as usize] += 1;
				}

				if old_style {
					for (index, result) in data.iter().enumerate() {
						dbgprintln!("{}: {}", index + 1, result);
					}
					dbgprintln!("\n");
				}

				for (index, datapoint) in accumulated.iter().enumerate() {
					dbgprintln!("{}: {}", index + 1, datapoint);
				}

				dbgprintln!("Misserfolge: {}", accumulated[0]); // 1
				dbgprintln!(
					"Misserfolge (improvisert): {}",
					accumulated[0] + accumulated[1]
				); // 1 + 2
				dbgprintln!(
					"Misserfolge (Pechphiole): {}",
					accumulated[0] + accumulated[1] + accumulated[2]
				); // 1 + 2 + 3
				dbgprintln!(
					"Erfolge (Wealthphiole): {}",
					accumulated[2] + accumulated[3] + accumulated[4] + accumulated[5]
				); // 3 + 4 + 5 + 6
				dbgprintln!(
					"Erfolge (Glücksphiole): {}",
					accumulated[3] + accumulated[4] + accumulated[5]
				); // 4 + 5 + 6
				dbgprintln!("Erfolge: {}", accumulated[4] + accumulated[5]); // 5 + 6
			} else {
				let sum: u64 = data.iter().map(|x| *x as u64).sum();

				if old_style {
					for (index, result) in data.iter().enumerate() {
						if *result != 0 {
							dbgprintln!("Augenzahl: {}\tErgebnis: {}", index + 1, result);
						}
					}
					println!();
				}
				dbgprintln!("Summe: {}", sum);
			}
		}

		if !no_summary {
			dbgprintln!(
				"Es wurde mit {} {} gewürfelt {} {} {} {}\n",
				self.count,
				if self.count == 1 {
					"Würfel"
				} else {
					"Würfeln"
				},
				if self.count == 1 { "welcher" } else { "welche" },
				self.sides,
				if self.sides == 1 { "Seite" } else { "Seiten" },
				if self.sides == 1 { "hatte" } else { "hatten" }
			);
		}
		if cfg!(debug_assertions) {
			if let Some(counts) = &self.counts {
				dbgprintln!(
					"Varianz: {}",
					calculate_variance(
						&counts[1..],
						counts[1..].iter().sum::<u64>() as u32,
						self.sides as u32
					)
				);
			} else if let Some(data) = &self.data {
				let mut counts = vec![0u64; self.sides as usize];
				for &value in data {
					if value > 0 && value <= self.sides {
						counts[(value - 1) as usize] += 1;
					}
				}
				dbgprintln!(
					"Varianz: {}",
					calculate_variance(&counts, data.len() as u32, self.sides as u32)
				);
			}
		}
	}
}

/// Feeds `amount` die results in `1..=sides` to `emit`, drawing entropy in
/// bit-packed chunks from `rng`'s u64 words instead of spending one word per die.
///
/// `B` is the number of randomness bits per die: the smallest number with
/// `2^B >= sides`. Each 64-bit word is split into `64 / B` chunks of `B` bits,
/// and a chunk is rejected when it is `>= sides`, which removes modulo bias.
/// Because `B` is minimal, the acceptance threshold is exactly `sides`, so an
/// accepted chunk already *is* the zero-based face: no modulo, no widening
/// multiply, and 64 bits of RNG output yield up to 21 dice (d6), 12 (d20) or
/// 9 (d100) instead of just one.
///
/// `emit` must return whether the face was accepted; it receives `0` for a
/// rejected chunk. Keeping rejection branch-free (masked face instead of an
/// `if`) matters: rejection happens on 6-37% of chunks, far too often for the
/// branch predictor to handle.
#[inline]
fn for_each_die<const B: u32, F: FnMut(u8) -> bool>(
	amount: usize,
	sides: u8,
	rng: &mut impl Rng,
	mut emit: F,
) {
	debug_assert!(u32::from(sides) <= (1 << B));
	let sides32 = u32::from(sides);
	let mask: u64 = (1 << B) - 1;
	let mut produced = 0usize;
	'words: while produced < amount {
		let mut word = rng.next_u64();
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
	($sides:expr, $amount:expr, $rng:expr, $emit:expr) => {
		match $sides {
			1 | 2 => for_each_die::<1, _>($amount, $sides, $rng, $emit),
			3 | 4 => for_each_die::<2, _>($amount, $sides, $rng, $emit),
			5..=8 => for_each_die::<3, _>($amount, $sides, $rng, $emit),
			9..=16 => for_each_die::<4, _>($amount, $sides, $rng, $emit),
			17..=32 => for_each_die::<5, _>($amount, $sides, $rng, $emit),
			33..=64 => for_each_die::<6, _>($amount, $sides, $rng, $emit),
			65..=128 => for_each_die::<7, _>($amount, $sides, $rng, $emit),
			_ => for_each_die::<8, _>($amount, $sides, $rng, $emit),
		}
	};
}

/// Hand-rolled AVX2 fast path for the histogram roll of dice with at most 8
/// sides (entropy width B <= 3).
///
/// The RNG stays scalar (CollatzWeyl64 is a serial dependency chain), but
/// extraction and counting move into registers:
///
/// * One YMM holds eight u32 halves of four u64 words; `_mm256_srl_epi32`
///   peels one B-bit chunk off every lane, so 16 words fill four YMMs.
/// * `vpackusdw` + `vpackuswb` condense two peeled layers into a 32-byte
///   vector holding 32 chunks (one per byte).
/// * The histogram never touches memory: for each face value,
///   `vpcmpeqb` + `vpmovmskb` + `popcnt` counts the matching chunks.
///   Rejected chunks (`>= sides`) simply match no compare — rejection stays
///   branch-free exactly like in the scalar path.
///
/// Register counting is what pays off: with at most 8 faces the per-face
/// compares are cheaper than the scalar table increments they replace, and
/// they also eliminate the store-forwarding chains between increments hitting
/// the same histogram slot.
#[cfg(target_arch = "x86_64")]
mod simd {
	use super::*;
	use core::arch::x86_64::*;

	/// u64 words buffered per RNG fill (32 KiB, L1-resident).
	const BUF_WORDS: usize = 4096;
	/// u64 words consumed per SIMD iteration.
	const ITER_WORDS: usize = 16;

	/// Upper bound on dice produced by one SIMD iteration:
	/// 16 words x 2 u32 halves x 32/B chunks each.
	const fn chunks_per_iter<const B: u32>() -> usize {
		ITER_WORDS * 2 * (32 / B as usize)
	}

	/// Consumes all `ITER_WORDS` words, counting accepted chunks of value
	/// `0..sides` into `counts[1..=sides]`. Returns how many dice were produced;
	/// the caller only invokes this while at least `chunks_per_iter::<B>()`
	/// dice remain, so it can never overshoot the requested amount.
	#[target_feature(enable = "avx2")]
	unsafe fn histogram_words<const B: u32>(
		words: &[u64; ITER_WORDS],
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
			let a = _mm256_loadu_si256(words[0..4].as_ptr().cast());
			let b = _mm256_loadu_si256(words[4..8].as_ptr().cast());
			let c = _mm256_loadu_si256(words[8..12].as_ptr().cast());
			let d = _mm256_loadu_si256(words[12..16].as_ptr().cast());
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

	/// Produces exactly `amount` faces in `1..=sides` into the 1-indexed
	/// `counts` table and returns the produced count. Large batches run through
	/// the AVX2 kernel; the sub-iteration tail falls back to the scalar sampler.
	pub(super) fn roll_small_sides<const B: u32>(
		amount: usize,
		sides: u8,
		rng: &mut impl Rng,
		counts: &mut [u64],
	) -> usize {
		debug_assert!(u32::from(sides) <= 1 << B);
		debug_assert!((1..=8).contains(&sides));
		let chunks_per_word = 2 * (32 / B as usize);
		let mut produced = 0usize;
		let mut buf = [0u64; BUF_WORDS];
		while amount - produced >= chunks_per_iter::<B>() {
			// Fill enough words for the remaining dice (plus slack for rejected
			// chunks), capped at the buffer size.
			let dice_left = amount - produced;
			let words = (dice_left / chunks_per_word + 1).min(BUF_WORDS);
			for word in &mut buf[..words] {
				*word = rng.next_u64();
			}
			let mut offset = 0;
			while amount - produced >= chunks_per_iter::<B>() && offset + ITER_WORDS <= words {
				let window: &[u64; ITER_WORDS] =
					buf[offset..offset + ITER_WORDS].try_into().unwrap();
				produced += unsafe { histogram_words::<B>(window, u32::from(sides), counts) };
				offset += ITER_WORDS;
			}
		}
		// Scalar tail: too few dice left to fill another SIMD iteration.
		// It may draw one extra word beyond what the last batch consumed;
		// faces stay uniform either way.
		let tail = amount - produced;
		if tail > 0 {
			let emit = |face: u8| {
				unsafe {
					*counts.get_unchecked_mut(face as usize) += 1;
				}
				face != 0
			};
			for_each_die::<B, _>(tail, sides, rng, emit);
			produced += tail;
		}
		produced
	}
}

pub fn roll(amount: usize, sides: u8, old_style: bool, rng: &mut impl Rng) -> Results {
	if old_style {
		// Constructed for input validation parity: rejects sides == 0 with the
		// same panic the sampling path used to produce.
		let _distribution =
			Uniform::<u8>::new_inclusive(1, sides).expect("Failed to create uniform distribution");
		let mut data: Vec<u8> = Vec::with_capacity(amount);
		let ptr = data.as_mut_ptr();
		let mut len = 0usize;
		// Branch-free sink: every chunk stores at `len`; rejected chunks write a
		// placeholder that the next accepted chunk overwrites, and only accepted
		// chunks advance `len`. The loop ends right after the `amount`-th
		// accepted store, so `data[0..amount]` is fully written.
		let emit = |face: u8| {
			unsafe {
				*ptr.add(len) = face;
			}
			let accepted = face != 0;
			len += accepted as usize;
			accepted
		};
		dispatch_dice!(sides, amount, rng, emit);
		unsafe {
			debug_assert!(len <= amount);
			data.set_len(len);
		}
		Results {
			counts: None,
			data: Some(data),
			sides,
			count: amount as u64,
		}
	} else {
		let _distribution = Uniform::<u8>::new_inclusive(1, sides)
			.expect("Failed to create uniform distribution for dice rolls");
		// Pre-allocate space for each side (1-indexed for convenience).
		// `sides as usize + 1` cannot wrap, unlike `sides + 1` for sides = 255.
		let mut counts = vec![0u64; sides as usize + 1];
		// The AVX2 kernel only pays off for large batches of small dice where
		// the register histogram beats scalar table increments; route to it
		// when available and fall through to the scalar sampler otherwise.
		let mut simd_done = false;
		#[cfg(target_arch = "x86_64")]
		if amount >= 1024 && matches!(sides, 1..=8) && std::arch::is_x86_feature_detected!("avx2") {
			let produced = match sides {
				1..=2 => simd::roll_small_sides::<1>(amount, sides, rng, &mut counts),
				3..=4 => simd::roll_small_sides::<2>(amount, sides, rng, &mut counts),
				_ => simd::roll_small_sides::<3>(amount, sides, rng, &mut counts),
			};
			debug_assert_eq!(produced, amount);
			simd_done = true;
		}
		if !simd_done {
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
			dispatch_dice!(sides, amount, rng, emit);
		}
		counts[0] = 0;
		Results {
			counts: Some(counts),
			data: None,
			sides,
			count: amount as u64,
		}
	}
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Dices {
	pub dices: Vec<u8>,
}

impl Default for Dices {
	fn default() -> Self {
		Dices {
			dices: vec![2, 3, 4, 6, 8, 10, 20, 100],
		}
	}
}

impl Loadable<Self> for Dices {
	fn load(file: Option<&str>) -> Self {
		let default_path = settings_path(NORMAL_DICES_FILE);
		let file_name = file
			.map_or(default_path, |s| Path::new(s).to_path_buf())
			.to_string_lossy()
			.to_string();
		let path = Path::new(&file_name);
		Dices::load_from_path(path).unwrap_or_else(|e| {
			dbgprintln!(
				"{}",
				Colour::RGB(255, 0, 0).paint(format!(
					"Error loading dices: {}. Attempting to create default.",
					e
				))
			);
			let default_dices = Dices::default();
			let save_result = default_dices.save_to_path(path);
			match save_result {
				Ok(_) => dbgprintln!("New default dices created at: {}", path.display()),
				Err(err) => dbgprintln!(
					"{}",
					Colour::RGB(255, 0, 0)
						.paint(format!("Error creating or saving default dices: {}", err))
				),
			}
			default_dices
		})
	}
}

impl Dices {
	fn load_from_path(path: &Path) -> Result<Self, Box<dyn Error>> {
		if !path.exists() {
			return Err("File does not exist".into());
		}

		let file = File::open(path)?;
		let buf_reader = BufReader::new(file);
		let dices = serde_yaml::from_reader(buf_reader)?;
		Ok(dices)
	}

	fn save_to_path(&self, path: &Path) -> Result<(), Box<dyn Error>> {
		if let Some(parent) = path.parent() {
			std::fs::create_dir_all(parent)?;
		}

		let file = OpenOptions::new()
			.write(true)
			.create(true)
			.truncate(true)
			.open(path)?;
		let writer = BufWriter::new(file);
		serde_yaml::to_writer(writer, self)?;
		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use collatz_weyl::CollatzWeyl64;

	fn make_rng() -> CollatzWeyl64 {
		CollatzWeyl64::new(0xD1CE | 1)
	}

	#[test]
	fn histogram_totals_and_layout() {
		for sides in [1u8, 2, 3, 6, 7, 20, 100, 255] {
			let mut rng = make_rng();
			let results = roll(100_000, sides, false, &mut rng);
			let counts = results
				.counts
				.expect("histogram path should produce counts");
			assert_eq!(counts.len(), sides as usize + 1);
			assert_eq!(counts[0], 0, "index 0 is the unused slot");
			assert_eq!(counts[1..].iter().sum::<u64>(), 100_000);
		}
	}

	#[test]
	fn old_style_values_in_range() {
		for sides in [1u8, 2, 6, 20, 100] {
			let mut rng = make_rng();
			let results = roll(100_000, sides, true, &mut rng);
			let data = results.data.expect("old-style path should produce data");
			assert_eq!(data.len(), 100_000);
			assert!(
				data.iter().all(|&value| (1..=sides).contains(&value)),
				"every roll must land in 1..={sides}"
			);
		}
	}

	#[test]
	fn faces_are_uniform() {
		// Chi-square-style sanity check: every face count must stay within 5 sigma
		// of its expectation. The seed is fixed, so the bound is deterministic.
		for sides in [2u8, 3, 6, 7, 20, 100, 255] {
			let rolls: u64 = 600_000;
			let mut rng = make_rng();
			let results = roll(rolls as usize, sides, false, &mut rng);
			let counts = results
				.counts
				.expect("histogram path should produce counts");
			let expected = rolls as f64 / f64::from(sides);
			let sigma = (expected * (1.0 - 1.0 / f64::from(sides))).sqrt();
			for &count in &counts[1..] {
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
	fn zero_amount_rolls_nothing() {
		let mut rng = make_rng();
		let results = roll(0, 6, false, &mut rng);
		assert_eq!(results.counts.expect("counts").iter().sum::<u64>(), 0);
	}

	#[test]
	#[should_panic(expected = "Failed to create uniform distribution")]
	fn zero_sides_still_panics() {
		let mut rng = make_rng();
		roll(1, 0, false, &mut rng);
	}

	#[test]
	fn same_seed_same_histogram() {
		// Both calls take the same code path (AVX2 kernel or scalar sampler,
		// depending on the host), so identical inputs must yield identical
		// histograms. 4097 also exercises a buffer-refill boundary.
		for sides in [1u8, 2, 3, 4, 6, 8] {
			let first = roll(40_097, sides, false, &mut make_rng());
			let second = roll(40_097, sides, false, &mut make_rng());
			assert_eq!(first.counts, second.counts, "sides {sides}");
		}
	}

	#[test]
	fn scalar_path_uniform() {
		// amount below the SIMD threshold pins the scalar sampler even on
		// AVX2 hosts; the AVX2 path itself is covered by faces_are_uniform.
		for sides in [2u8, 3, 6, 7, 8] {
			let rolls: u64 = 1023;
			let mut rng = make_rng();
			let results = roll(rolls as usize, sides, false, &mut rng);
			let counts = results
				.counts
				.expect("histogram path should produce counts");
			assert_eq!(counts[1..].iter().sum::<u64>(), rolls);
			let expected = rolls as f64 / f64::from(sides);
			let sigma = (expected * (1.0 - 1.0 / f64::from(sides))).sqrt();
			for &count in &counts[1..] {
				assert!(
					((count as f64 - expected).abs()) < 5.0 * sigma,
					"sides {sides}: count {count} vs expected {expected:.1}"
				);
			}
		}
	}

	#[test]
	fn simd_threshold_amounts_are_exact() {
		// Around the 1024-dispatch threshold every amount must produce exactly
		// that many counted dice, whatever path it lands on.
		for sides in [1u8, 2, 3, 4, 5, 6, 7, 8] {
			for amount in [1023usize, 1024, 1025, 4095, 4096, 4097] {
				let mut rng = make_rng();
				let results = roll(amount, sides, false, &mut rng);
				let counts = results
					.counts
					.expect("histogram path should produce counts");
				assert_eq!(counts[1..].iter().sum::<u64>(), amount as u64);
				assert_eq!(counts[0], 0, "scratch slot must stay zeroed");
			}
		}
	}
}
