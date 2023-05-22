//! Parse CSS file.
//! We use a basic syntax with custom properties and variable.
//! See [ComputedProperties](crate::properties::ComputedProperties)
//! Variables are defined and used like this.
//!
//! ```
//! variables {
//!   pinky: #F06;
//! }
//! variables.macos {
//!   os-padding: 6;
//! }
//! variables.dark.macos {
//!   dark-margin: 10;
//! }
//!
//! label {
//!   color: var(pinky);
//!   padding: var(os-padding);
//!   margin: var(dark-margin);
//! }
//! ```
//!
//! `system_classes` are applied to resolve the variables only.

mod parser;
mod properties;
mod rules;

pub use parser::parse_css;
pub use properties::{
    Align, Color, ComputedProperties, FontFamily, OptionalProperties, Radius, Sides,
};
pub use rules::Rules;
pub use simplecss::{AttributeOperator, Element, PseudoClass};
