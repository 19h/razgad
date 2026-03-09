use crate::{text, Confidence, Scheme, Symbol};

pub fn decode(scheme: Scheme, input: &str) -> Option<Symbol> {
    for candidate in rust_candidates(input) {
        let Ok(demangled) = rustc_demangle::try_demangle(candidate) else {
            continue;
        };
        let display = normalize_rust_display(&format!("{demangled:#}"));
        if display == candidate {
            continue;
        }

        let mut symbol = text::symbol_from_qualified_display(
            Symbol::new(scheme, crate::SymbolKind::Function),
            &display,
            "::",
        );
        symbol.concrete_family = match scheme {
            Scheme::RustLegacy => Scheme::RustLegacy,
            Scheme::RustV0 => Scheme::RustV0,
            _ => scheme,
        };
        return Some(symbol.with_display(&display).with_verbatim(input));
    }
    None
}

fn normalize_rust_display(display: &str) -> String {
    if let Some(inner) = display.strip_prefix('<') {
        if let Some((inner, tail)) = inner.split_once(">::") {
            let base = inner.split(" as ").next().unwrap_or(inner);
            return format!("{base}::{tail}");
        }
    }
    display.to_string()
}

pub fn detect(input: &str) -> Option<(Scheme, Confidence)> {
    if is_valid_rust_v0(input) {
        return Some((Scheme::RustV0, Confidence::Certain));
    }
    if is_valid_rust_legacy(input) {
        return Some((Scheme::RustLegacy, Confidence::Certain));
    }
    None
}

fn is_valid_rust_v0(input: &str) -> bool {
    (input.starts_with("__RN") || input.starts_with("_R"))
        && rust_candidates(input).into_iter().any(demangles_as_rust)
}

fn is_valid_rust_legacy(input: &str) -> bool {
    (input.starts_with("__ZN") || input.starts_with("_ZN"))
        && rust_candidates(input).into_iter().any(demangles_as_rust)
}

fn demangles_as_rust(input: &str) -> bool {
    rustc_demangle::try_demangle(input)
        .ok()
        .map(|demangled| demangled.to_string() != input)
        .unwrap_or(false)
}

fn rust_candidates(input: &str) -> Vec<&str> {
    let mut candidates = vec![input];
    if let Some(stripped) = strip_numeric_clone_suffix(input) {
        if stripped != input {
            candidates.push(stripped);
        }
    }
    if let Some(stripped) = strip_llvm_suffix(input) {
        if stripped != input && !candidates.contains(&stripped) {
            candidates.push(stripped);
        }
    }
    candidates
}

fn strip_numeric_clone_suffix(input: &str) -> Option<&str> {
    let (head, tail) = input.rsplit_once('_')?;
    (!tail.is_empty() && tail.chars().all(|ch| ch.is_ascii_digit())).then_some(head)
}

fn strip_llvm_suffix(input: &str) -> Option<&str> {
    input.find(".llvm.").map(|index| &input[..index])
}
