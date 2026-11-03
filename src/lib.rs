#![doc = include_str!("../README.md")]
#![deny(missing_docs)]

/// Regex based url rewrite
pub mod regex;
/// Replace url write
pub mod replace;
/// Strip prefix uri rewrite
pub mod strip_prefix;
/// Strip suffix uri rewrite
pub mod strip_suffix;
#[cfg(test)]
mod tests;
