//! Universal multi-scheme symbol demangling and remangling.

mod codec;
mod error;
mod heuristics;
mod model;
mod schemes;
mod text;

pub use crate::error::Error;
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
