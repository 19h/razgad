use crate::{function_names, text, Confidence, Name, Scheme, Signature, Symbol, SymbolKind, Type};

pub fn decode(scheme: Scheme, input: &str) -> Option<Symbol> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }

    let parsed = function_names::parse_function_name(trimmed)?;
    let mut symbol = Symbol::new(
        scheme,
        parsed
            .callable_name
            .as_deref()
            .map(classify_plain_kind)
            .unwrap_or(SymbolKind::Function),
    );

    if let Some(callable) = parsed.callable_name.as_deref() {
        if callable.contains("::") {
            symbol.path = parsed
                .callable_path
                .iter()
                .map(|part| text::parse_name(part, "::"))
                .collect();
        } else {
            symbol.path = vec![text::parse_name(callable, "::")];
        }
    } else {
        symbol.path = vec![Name::identifier(parsed.normalized.clone())];
    }

    if parsed.has_signature() {
        symbol.signature = Some(Signature {
            calling_convention: parsed
                .calling_convention
                .as_deref()
                .and_then(map_calling_convention),
            parameters: parsed
                .arguments
                .iter()
                .map(|arg| parse_plain_type(&arg.type_text))
                .collect(),
            return_type: parsed.return_type.as_deref().map(parse_plain_type),
        });
    }

    Some(symbol.with_display(parsed.full).with_verbatim(trimmed))
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

fn parse_plain_type(text: &str) -> Type {
    let trimmed = text.trim();
    if trimmed == "..." {
        Type::Other("...".to_string())
    } else {
        text::parse_type(trimmed, "::")
    }
}

fn classify_plain_kind(callable: &str) -> SymbolKind {
    let path = text::parse_names(callable, "::");
    if path.len() >= 2 {
        if let (Some(Name::Identifier(last)), Some(Name::Identifier(prev))) =
            (path.last(), path.get(path.len() - 2))
        {
            if last == prev {
                return SymbolKind::Constructor;
            }
            if last.starts_with('~') {
                return SymbolKind::Destructor;
            }
        }
    }
    if path.len() >= 2 {
        SymbolKind::Method
    } else {
        SymbolKind::Function
    }
}

fn map_calling_convention(value: &str) -> Option<crate::CallingConvention> {
    function_names::parse_calling_convention_token(value)
}
