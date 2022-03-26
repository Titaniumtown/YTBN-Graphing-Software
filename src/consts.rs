use crate::function::Riemann;
use std::ops::RangeInclusive;

use const_format::formatc;
use shadow_rs::shadow;
shadow!(build);

// Constant string that has a string containing information about the build.
pub const BUILD_INFO: &str = formatc!(
	"Commit: {} ({})\nBuild Date: {}\nRust Channel: {}\nRust Version: {}",
	&build::SHORT_COMMIT,
	&build::BRANCH,
	&build::BUILD_TIME,
	&build::RUST_CHANNEL,
	&build::RUST_VERSION,
);

// Hard-Coded limits

/// Range of acceptable input values for integral_num
pub const INTEGRAL_NUM_RANGE: RangeInclusive<usize> = 1..=50000;
/// Minimum X value for calculating an Integral
pub const INTEGRAL_X_MIN: f64 = -1000.0;
/// Maximum X value for calculating an Integral

pub const INTEGRAL_X_MAX: f64 = 1000.0;
/// Range of acceptable x coordinates for calculating an integral
pub const INTEGRAL_X_RANGE: RangeInclusive<f64> = INTEGRAL_X_MIN..=INTEGRAL_X_MAX;

// Default values

/// Default Riemann Sum to calculate
pub const DEFAULT_RIEMANN: Riemann = Riemann::Left;

/// Default minimum X value to display
pub const DEFAULT_MIN_X: f64 = -10.0;

/// Default Maxmimum X value to display

pub const DEFAULT_MAX_X: f64 = 10.0;

/// Default number of integral boxes
pub const DEFAULT_INTEGRAL_NUM: usize = 100;
