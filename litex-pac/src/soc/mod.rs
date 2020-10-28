
/// Peripherals shared by multiple devices
pub mod peripherals;

/// Peripheral instances shared by multiple devices
pub(crate) mod instances;

/// Metadata
pub mod metadata;

pub use self::instances::ctrl;
pub use self::instances::leds;
