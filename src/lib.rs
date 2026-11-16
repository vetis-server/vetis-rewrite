#![doc = include_str!("../README.md")]
#![deny(missing_docs)]

/// Query rewrite module
pub mod query;
/// Regex rewrite module
pub mod regex;
/// Replace rewrite module
pub mod replace;
/// Replace req_headers module
pub mod req_headers;
/// Replace res_headers module
pub mod res_headers;
/// Strip prefix rewrite module
pub mod strip_prefix;
/// Strip suffix rewrite module
pub mod strip_suffix;
#[cfg(test)]
mod tests;
