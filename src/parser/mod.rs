#![allow(deprecated)]
pub mod grl_helpers;
pub mod grl_no_regex;
/// GRL (Grule Rule Language) parser implementation
// Core parsing modules (no regex dependency)
pub mod literal_search;
pub mod parallel;
pub mod simd_search;
pub mod zero_copy;

// Legacy regex-based parser (deprecated, behind feature flag)
#[cfg(feature = "legacy-regex-parser")]
pub mod grl;

// Re-export the recommended parser as GRLParser
pub use grl_no_regex::GRLParserNoRegex as GRLParser;
pub use grl_no_regex::ParsedGRL;

// Also export the legacy parser when feature is enabled
#[cfg(feature = "legacy-regex-parser")]
pub use grl::GRLParser as LegacyGRLParser;
