//! Breadify — turns Matvare Expressen's daily bread-order export into printed
//! A4 picking lists.
//!
//! The specifications live in `docs/`: `excel-format.md` describes the input
//! file and the validation a loader must perform, `print-spec.md` describes
//! what the printed page must say, and `print-layout.md` is the decision log
//! those two are derived from.

pub mod order;
pub mod sheet;
pub mod validate;
