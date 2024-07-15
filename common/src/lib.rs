use std::arch::asm;
use directories::ProjectDirs;
use std::fs::create_dir_all;
use std::path::{PathBuf, MAIN_SEPARATOR};
use std::process::exit;

pub use macros;

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
use cpuid::CpuFeature;


pub fn settings_path(file: &str) -> PathBuf {
	let dir = match ProjectDirs::from("", "", "würfeln") {
		None => PathBuf::from(format!(".{}{}", MAIN_SEPARATOR, file)),
		Some(dirs) => {
			if !dirs.data_dir().exists() {
				match create_dir_all(dirs.data_dir()) {
					Ok(_) => {}
					Err(e) => {
						eprintln!("{}", e);
						exit(-1);
					}
				}
			}
			PathBuf::from(format!(
				"{}{}{}",
				dirs.data_dir().to_str().unwrap(),
				MAIN_SEPARATOR,
				file
			))
		}
	};

    macros::dbgprintln!("Loading from file: {}", dir.to_str().unwrap());

	dir
}

/// Generates a random number using the hardware random number generator if available, otherwise returns None
#[inline]
pub fn random() -> Option<u64> {
	let mut value: u64;
	let success: u8;

	#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
	unsafe {
		asm!(
		"rdrand {0}; setc {1}",
		out(reg) value,
		out(reg_byte) success,
		options(nostack, nomem)
		);
	}

	#[cfg(target_arch = "aarch64")]
	unsafe {
		asm!(
		"mrs {0}, RNDR",
		"cset {1}, NE",  // Set success to 1 if RNDR succeeded, otherwise 0
		out(reg) value,
		out(reg) success,
		options(nostack, nomem)
		);
	}

	if success != 0 {
		Some(value)
	} else {
		None
	}
}

/// Checks if the hardware random number generator is available on the current architecture

pub fn hwrng_available() -> Result<bool, String> {
	let available: bool;
	#[cfg(target_arch = "aarch64")]
	unsafe {
		let mut reg: u64;

		asm!("mrs {0}, ID_AA64ISAR0_EL1", out(reg) reg);

		// RNDR is indicated by bits [59:56] in ID_AA64ISAR0_EL1
		// 0b0001 indicates support for RNDR and RNDRRS
		available = ((reg >> 56) & 0xF) == 0b0001;
	}

	#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
	{
		available = cpuid::identify()?.has_feature(CpuFeature::RDRAND);
	}

	#[cfg(not(any(target_arch = "x86_64", target_arch = "x86", target_arch = "aarch64")))]
	{
		eprintln!("Hardware random number generator not available on this architecture.");
		available = false;
	}
	Ok(available)
}

pub trait ToByteArray {
	fn to_byte_array(self) -> [u8; 8];
}

impl ToByteArray for u64 {
	fn to_byte_array(self) -> [u8; 8] {
		[
			(self & 0xFF) as u8,
			((self >> 8) & 0xFF) as u8,
			((self >> 16) & 0xFF) as u8,
			((self >> 24) & 0xFF) as u8,
			((self >> 32) & 0xFF) as u8,
			((self >> 40) & 0xFF) as u8,
			((self >> 48) & 0xFF) as u8,
			((self >> 56) & 0xFF) as u8,
		]
	}
}

pub trait Loadable<T> {
	fn load(file: Option<&str>) -> T;
}

pub trait Rollable<T> {
    fn roll(&self, use_hw_rng: bool) -> &T;
}