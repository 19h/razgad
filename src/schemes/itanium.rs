use crate::{text, Confidence, Scheme, Symbol};
use cpp_demangle::{DemangleOptions, ParseOptions};

pub fn decode(scheme: Scheme, input: &str) -> Option<Symbol> {
    if !input.starts_with("_Z") && !input.starts_with("__Z") {
        return None;
    }

    let core = strip_optimizer_suffix(input);

    for candidate in [input, core] {
        if let Some(demangled) = parse_and_demangle(candidate) {
            let demangled = normalize_demangled(&demangled);

            let mut symbol = text::symbol_from_demangled_cpp(
                Symbol::new(scheme, crate::SymbolKind::Function),
                &demangled,
                "::",
            );
            if let Some(signature) = &mut symbol.signature {
                if signature.return_type.is_none() {
                    signature.return_type = Some(crate::Type::void());
                }
            }
            if matches!(
                scheme,
                Scheme::CrayCpp | Scheme::CarbonCpp | Scheme::IntelNativeCpp
            ) {
                symbol.concrete_family = Scheme::ItaniumCpp;
            }
            if scheme == Scheme::ItaniumCpp {
                symbol.concrete_family = Scheme::ItaniumCpp;
            }
            return Some(symbol.with_verbatim(input));
        }
    }

    fallback_decode(scheme, core).map(|symbol| symbol.with_verbatim(input))
}

fn parse_and_demangle(input: &str) -> Option<String> {
    let parse_options = ParseOptions::default().recursion_limit(4096);
    let demangle_options = DemangleOptions::default().recursion_limit(4096);
    cpp_demangle::Symbol::new_with_options(input.as_bytes(), &parse_options)
        .ok()?
        .demangle_with_options(&demangle_options)
        .ok()
}

pub fn detect(input: &str) -> Option<(Scheme, Confidence)> {
    let core = strip_optimizer_suffix(input);
    if (input.starts_with("_Z") || input.starts_with("__Z"))
        && (parse_and_demangle(core).is_some()
            || fallback_decode(Scheme::ItaniumCpp, core).is_some())
    {
        Some((Scheme::ItaniumCpp, Confidence::Certain))
    } else {
        None
    }
}

fn normalize_demangled(input: &str) -> String {
    if let Some(inner) = input
        .strip_prefix("{vtable(")
        .and_then(|inner| inner.strip_suffix(")}"))
    {
        return format!("vtable for {inner}");
    }
    if let Some(inner) = input
        .strip_prefix("{typeinfo(")
        .and_then(|inner| inner.strip_suffix(")}"))
    {
        return format!("typeinfo for {inner}");
    }
    if let Some(inner) = input
        .strip_prefix("{virtual override thunk({offset(")
        .and_then(|inner| inner.split_once("}, "))
        .and_then(|(_, rest)| rest.strip_suffix(")}"))
    {
        return format!("non-virtual thunk to {inner}");
    }
    input.to_string()
}

fn fallback_decode(scheme: Scheme, input: &str) -> Option<Symbol> {
    let mut cursor = input.strip_prefix("__").unwrap_or(input);
    cursor = cursor.strip_prefix("_Z")?;

    let path = if let Some(rest) = cursor.strip_prefix('N') {
        let (names, _) = parse_length_names(rest)?;
        names
    } else {
        let (name, _) = parse_length_name(cursor)?;
        vec![name]
    };

    if path.is_empty() {
        return None;
    }

    let kind = if path.len() >= 2 {
        crate::SymbolKind::Method
    } else {
        crate::SymbolKind::Function
    };
    let display = path.join("::");
    let mut symbol = Symbol::new(scheme, kind);
    symbol.path = path.into_iter().map(crate::Name::identifier).collect();
    symbol.concrete_family = Scheme::ItaniumCpp;
    Some(symbol.with_display(display))
}

fn parse_length_names(mut input: &str) -> Option<(Vec<String>, &str)> {
    let mut out = Vec::new();
    while !input.is_empty() {
        if !input.chars().next()?.is_ascii_digit() {
            break;
        }
        let (name, rest) = parse_length_name(input)?;
        out.push(name);
        input = rest;
    }
    (!out.is_empty()).then_some((out, input))
}

fn parse_length_name(input: &str) -> Option<(String, &str)> {
    let digits_end = input.find(|ch: char| !ch.is_ascii_digit())?;
    let len = input[..digits_end].parse::<usize>().ok()?;
    let name_end = digits_end.checked_add(len)?;
    let name = input.get(digits_end..name_end)?;
    Some((name.to_string(), &input[name_end..]))
}

fn strip_optimizer_suffix(input: &str) -> &str {
    [
        ".isra.",
        ".constprop.",
        ".part.",
        ".cold",
        ".clone.",
        ".llvm.",
    ]
    .iter()
    .filter_map(|marker| input.find(marker))
    .min()
    .map(|index| &input[..index])
    .unwrap_or(input)
}
