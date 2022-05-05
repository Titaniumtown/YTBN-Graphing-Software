use crate::function_entry::Riemann;
use std::ops::RangeInclusive;

use const_format::formatc;
use epaint::Color32;
use shadow_rs::shadow;
shadow!(build);

// Constant string that has a string containing information about the build.
pub const BUILD_INFO: &str = formatc!(
	"Commit: {} ({})\nBuild Date: {}\nPackage Version: {}\nRust Channel: {}\nRust Version: {}",
	&build::SHORT_COMMIT,
	&build::BRANCH,
	&build::BUILD_TIME,
	&build::PKG_VERSION,
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

const_assert!(INTEGRAL_X_MAX > INTEGRAL_X_MIN);

/// Range of acceptable x coordinates for calculating an integral
pub const INTEGRAL_X_RANGE: RangeInclusive<f64> = INTEGRAL_X_MIN..=INTEGRAL_X_MAX;

// Default values

/// Default Riemann Sum to calculate
pub const DEFAULT_RIEMANN: Riemann = Riemann::Left;

/// Default minimum X value to display
pub const DEFAULT_MIN_X: f64 = -10.0;

/// Default Maxmimum X value to display
pub const DEFAULT_MAX_X: f64 = 10.0;

const_assert!(DEFAULT_MAX_X > DEFAULT_MIN_X);

/// Default number of integral boxes
pub const DEFAULT_INTEGRAL_NUM: usize = 100;

/// Colors used for plotting
// Colors commented out are used elsewhere and are not included here for better user experience
pub const COLORS: &[Color32; 13] = &[
	Color32::RED,
	// Color32::GREEN,
	// Color32::YELLOW,
	// Color32::BLUE,
	Color32::BROWN,
	Color32::GOLD,
	Color32::GRAY,
	Color32::WHITE,
	Color32::LIGHT_YELLOW,
	Color32::LIGHT_GREEN,
	// Color32::LIGHT_BLUE,
	Color32::LIGHT_GRAY,
	Color32::LIGHT_RED,
	Color32::DARK_GRAY,
	// Color32::DARK_RED,
	Color32::KHAKI,
	Color32::DARK_GREEN,
	Color32::DARK_BLUE,
];

#[cfg(target_arch = "wasm32")]
lazy_static::lazy_static! {
	static ref IS_MOBILE: bool = {
		// from https://github.com/emilk/egui/blob/fda8189cbab18e0acab8db972400e4a4ca0d915e/egui_web/src/text_agent.rs#L194
		fn is_mobile() -> Option<bool> {
			const MOBILE_DEVICE: [&str; 6] = ["Android", "iPhone", "iPad", "iPod", "webOS", "BlackBerry"];

			let user_agent = web_sys::window()?.navigator().user_agent().ok()?;
			Some(MOBILE_DEVICE.iter().any(|&name| user_agent.contains(name)))
		}

		is_mobile().unwrap_or_default()
	};
}

#[inline]
#[cfg(target_arch = "wasm32")]
pub fn is_mobile() -> bool { return *IS_MOBILE; }

#[cfg(not(target_arch = "wasm32"))]
pub const fn is_mobile() -> bool { false }
