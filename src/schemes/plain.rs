use crate::{Confidence, Name, Scheme, Symbol, SymbolKind, text};

pub fn decode(scheme: Scheme, input: &str) -> Option<Symbol> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }

    let mut symbol = Symbol::new(
        scheme,
        if trimmed.contains("::") {
            SymbolKind::Method
        } else {
            SymbolKind::Function
        },
    );

    if trimmed.contains("::") {
        symbol.path = text::parse_names(trimmed, "::");
    } else {
        symbol.path = vec![Name::identifier(trimmed)];
    }

    Some(symbol.with_display(trimmed).with_verbatim(trimmed))
}

pub fn detect(input: &str) -> Option<(Scheme, Confidence)> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }

    let confidence = if trimmed.contains("::") {
        Confidence::High
    } else {
        Confidence::Medium
    };

    Some((Scheme::Plain, confidence))
}
