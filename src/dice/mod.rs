pub mod colored_dice;
pub mod crit_dice;
pub mod normal_dice;
#[cfg(target_arch = "x86_64")]
pub mod normal_dice_avx2;
