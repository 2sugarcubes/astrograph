pub mod body;
#[allow(clippy::excessive_precision)] // Constants should work with up to f128 precision
pub mod consts;
pub mod dynamic;
pub mod output;
pub mod program;
pub mod projection;
#[allow(unused_imports)] // Macro_use is required here
#[macro_use]
extern crate assert_float_eq;

type Float = f32;
