use const_format::formatc;
use epaint::Color32;
use shadow_rs::shadow;
shadow!(build);

/// Constant string that has a string containing information about the build.
pub const BUILD_INFO: &str = formatc!(
	"Commit: {} ({})\nBuild Date: {}\nPackage Version: {}\nRust Channel: {}\nRust Version: {}",
	&build::SHORT_COMMIT,
	&build::BRANCH,
	&build::BUILD_TIME,
	&build::PKG_VERSION,
	&build::RUST_CHANNEL,
	&build::RUST_VERSION,
);

pub const FONT_SIZE: f32 = 14.0;

// Default values
/// Default minimum X value to display
pub const DEFAULT_MIN_X: f64 = -10.0;

/// Default Maxmimum X value to display
pub const DEFAULT_MAX_X: f64 = 10.0;

const_assert!(DEFAULT_MAX_X > DEFAULT_MIN_X);

/// Default number of integral boxes
pub const DEFAULT_INTEGRAL_NUM: usize = 100;

/// Colors used for plotting
// Colors commented out are used elsewhere and are not included here for better user experience
pub const COLORS: [Color32; 13] = [
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

const_assert!(!COLORS.is_empty());
