//! Universal multi-scheme symbol demangling and remangling.

mod codec;
mod error;
mod function_names;
mod heuristics;
mod model;
mod schemes;
mod text;

pub use crate::error::Error;
pub use crate::function_names::{
    normalize_symbol_display, parse_function_name, parse_function_name_with_separator,
    parse_template_node, parse_template_node_with_separator, split_argument_name,
    split_argument_name_with_separator, split_scope, split_scope_with_separator, template_depth,
    AccessModifier, ParsedArgument, ParsedFunctionName, TemplateNode, TemplateNodeKind,
};
pub use crate::model::{
    CallingConvention, Confidence, DetectedSymbol, Name, PlatformDecorations, Scheme, Signature,
    SpecialKind, Symbol, SymbolKind, Type,
};

pub fn decode(scheme: Scheme, input: &str) -> Result<Symbol, Error> {
    schemes::decode(scheme, input)
}

pub fn encode(scheme: Scheme, symbol: &Symbol) -> Result<String, Error> {
    codec::encode_symbol(scheme, symbol)
}

pub fn heuristic_decode(input: &str) -> Result<DetectedSymbol, Error> {
    heuristics::heuristic_decode_symbol(input)
}
