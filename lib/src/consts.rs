use crate::Float;

/// Gravitational constant in terms of light seconds^3 per jupiter mass per hour^2
pub const GRAVITATIONAL_CONSTANT: Float = 0.0609_109;

/// Constants for the [`crate::Float`] type alias.
#[cfg(any(target_arch = "wasm32", not(feature = "f64")))]
pub mod float {
    pub use core::f32::consts::*;
}

/// Constants for the [`crate::Float`] type alias.
#[cfg(all(feature = "f64", not(target_arch = "wasm32")))]
pub mod float {
    pub use core::f64::consts::*;
}
