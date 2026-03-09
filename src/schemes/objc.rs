use crate::{Confidence, Name, Scheme, SpecialKind, Symbol, SymbolKind};

pub fn decode(scheme: Scheme, input: &str) -> Option<Symbol> {
    if let Some(class_name) = input.strip_prefix("_OBJC_CLASS_$_") {
        let mut symbol = Symbol::special(scheme, SpecialKind::ObjectiveCClass, [class_name]);
        symbol.concrete_family = Scheme::ObjectiveC;
        return Some(
            symbol
                .with_display(format!("Objective-C class {class_name}"))
                .with_verbatim(input),
        );
    }

    if input == "v@:" {
        return Some(
            Symbol::new(scheme, SymbolKind::TypeEncoding)
                .with_display("void self selector")
                .with_verbatim(input),
        );
    }

    if let Some(method) = parse_block_invoke(input) {
        let mut symbol = Symbol::new(scheme, SymbolKind::Runtime);
        if let Some((class_name, selector)) = parse_method(method) {
            symbol.path = vec![Name::identifier(class_name), Name::identifier(selector)];
        }
        return Some(
            symbol
                .with_display(format!("block invoke for {method}"))
                .with_verbatim(input),
        );
    }

    if let Some(method) = parse_cold_clone(input) {
        let mut symbol = Symbol::new(scheme, SymbolKind::Runtime);
        if let Some((class_name, selector)) = parse_method(method) {
            symbol.path = vec![Name::identifier(class_name), Name::identifier(selector)];
        }
        return Some(
            symbol
                .with_display(format!("cold clone of {method}"))
                .with_verbatim(input),
        );
    }

    if let Some(method) = parse_numeric_clone(input) {
        let mut symbol = Symbol::new(scheme, SymbolKind::Runtime);
        if let Some((class_name, selector)) = parse_method(method) {
            symbol.path = vec![Name::identifier(class_name), Name::identifier(selector)];
        }
        return Some(
            symbol
                .with_display(format!("clone of {method}"))
                .with_verbatim(input),
        );
    }

    if let Some((class_name, selector)) = parse_method(input) {
        let mut symbol = Symbol::new(scheme, SymbolKind::Method);
        symbol.path = vec![Name::identifier(class_name), Name::identifier(selector)];
        return Some(symbol.with_display(input).with_verbatim(input));
    }

    None
}

pub fn detect(input: &str) -> Option<(Scheme, Confidence)> {
    if input.starts_with("_OBJC_")
        || input.starts_with("OBJC_")
        || input.starts_with("-[")
        || input.starts_with("+[")
    {
        return Some((Scheme::ObjectiveC, Confidence::Certain));
    }
    if (input.contains("-[") || input.contains("+["))
        && (input.contains("_block_invoke") || input.contains(".cold"))
    {
        return Some((Scheme::ObjectiveC, Confidence::High));
    }
    if input == "v@:" {
        return Some((Scheme::ObjectiveC, Confidence::Medium));
    }
    None
}

fn parse_method(input: &str) -> Option<(&str, &str)> {
    let body = input
        .strip_prefix("-[")
        .or_else(|| input.strip_prefix("+["))?;
    let body = body.strip_suffix(']')?;
    let (class_name, selector) = body.split_once(' ')?;
    Some((class_name, selector))
}

fn parse_block_invoke(input: &str) -> Option<&str> {
    let start = input.find("-[").or_else(|| input.find("+["))?;
    let suffix = &input[start..];
    let end = suffix.find(']')?;
    suffix[end + 1..]
        .contains("_block_invoke")
        .then_some(&input[start..start + end + 1])
}

fn parse_cold_clone(input: &str) -> Option<&str> {
    if let Some((method, _)) = input.split_once(".cold") {
        if method.starts_with("-[") || method.starts_with("+[") {
            return Some(method);
        }
    }
    None
}

fn parse_numeric_clone(input: &str) -> Option<&str> {
    let (method, suffix) = input.rsplit_once('_')?;
    if !suffix.is_empty()
        && suffix.chars().all(|ch| ch.is_ascii_digit())
        && (method.starts_with("-[") || method.starts_with("+["))
    {
        Some(method)
    } else {
        None
    }
}
