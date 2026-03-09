use crate::{Confidence, Name, Scheme, Symbol, SymbolKind};

pub fn decode(scheme: Scheme, input: &str) -> Option<Symbol> {
    match scheme {
        Scheme::UnityIl2Cpp => decode_il2cpp(input),
        Scheme::MonoManaged => decode_mono(input),
        _ => None,
    }
}

pub fn detect(input: &str) -> Option<(Scheme, Confidence)> {
    if looks_like_mono(input) {
        return Some((Scheme::MonoManaged, Confidence::Certain));
    }
    if looks_like_il2cpp(input) {
        return Some((Scheme::UnityIl2Cpp, Confidence::High));
    }
    None
}

fn decode_il2cpp(input: &str) -> Option<Symbol> {
    let base = strip_il2cpp_suffix(input)?;
    let split = base.rfind('_')?;
    let owner = &base[..split];
    let method = &base[split + 1..];

    let mut symbol = Symbol::new(Scheme::UnityIl2Cpp, SymbolKind::Method);
    symbol.path = vec![Name::identifier(owner), Name::identifier(method)];
    Some(
        symbol
            .with_display(format!("{owner}::{method}"))
            .with_verbatim(input),
    )
}

fn decode_mono(input: &str) -> Option<Symbol> {
    let (owner, method) = input.split_once("$$")?;
    let mut symbol = Symbol::new(Scheme::MonoManaged, SymbolKind::Method);
    let mut path = owner
        .split('.')
        .filter(|part| !part.is_empty())
        .map(Name::identifier)
        .collect::<Vec<_>>();
    path.push(Name::identifier(method));
    symbol.path = path;
    Some(
        symbol
            .with_display(format!("{owner}::{method}"))
            .with_verbatim(input),
    )
}

fn looks_like_mono(input: &str) -> bool {
    input.contains("$$") && !input.starts_with("P$")
}

fn looks_like_il2cpp(input: &str) -> bool {
    strip_il2cpp_suffix(input).is_some()
}

fn strip_il2cpp_suffix(input: &str) -> Option<&str> {
    let marker = input.rfind("_m")?;
    let suffix = &input[marker + 2..];
    let hex_len = suffix
        .chars()
        .take_while(|ch| ch.is_ascii_hexdigit())
        .count();
    if hex_len < 16 {
        return None;
    }
    let remainder = &suffix[hex_len..];
    if remainder.is_empty() || remainder.starts_with('_') {
        Some(&input[..marker])
    } else {
        None
    }
}
