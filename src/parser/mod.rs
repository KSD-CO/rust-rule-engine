/// GRL (Grule Rule Language) parser implementation
pub mod grl;
/// GRL parser module for parsing rule files
pub mod grl_parser;

pub use grl::GRLParser;
pub use grl_parser::GRLParser as SimpleGRLParser;
