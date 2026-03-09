use crate::{Confidence, PlatformDecorations, Scheme, Symbol, SymbolKind};

pub fn decode(scheme: Scheme, input: &str) -> Option<Symbol> {
    match scheme {
        Scheme::MachO => decode_macho(input),
        Scheme::CoffPe => decode_coff(input),
        Scheme::Elf => decode_elf(input),
        _ => None,
    }
}

pub fn detect(input: &str) -> Option<(Scheme, Confidence)> {
    if input.starts_with("__imp_") {
        return Some((Scheme::CoffPe, Confidence::Certain));
    }
    if input.starts_with("__Z") {
        return Some((Scheme::MachO, Confidence::Certain));
    }
    if let Some((_, suffix)) = input.rsplit_once("@@") {
        if !suffix.chars().all(|ch| ch.is_ascii_digit()) {
            return Some((Scheme::Elf, Confidence::Certain));
        }
    }
    None
}

fn decode_macho(input: &str) -> Option<Symbol> {
    let inner = input.strip_prefix('_')?;
    for candidate in macho_candidates(inner) {
        if let Some(mut symbol) = crate::schemes::itanium::decode(Scheme::MachO, candidate) {
            symbol.scheme = Scheme::MachO;
            symbol.concrete_family = Scheme::ItaniumCpp;
            symbol.platform.leading_underscore = true;
            symbol.verbatim = Some(input.to_string());
            return Some(symbol);
        }
    }

    let inner = strip_instance_suffix(inner);
    let mut symbol = Symbol::new(
        Scheme::MachO,
        if embedded_objc_method(inner).is_some() || inner.contains("_block_invoke") {
            SymbolKind::Runtime
        } else {
            SymbolKind::Function
        },
    );
    symbol.concrete_family = Scheme::ItaniumCpp;
    symbol.platform.leading_underscore = true;
    symbol.display = Some(match embedded_objc_method(inner) {
        Some(method) => format!("macho wrapper for {method}"),
        None => inner.to_string(),
    });
    symbol.verbatim = Some(input.to_string());
    Some(symbol)
}

fn decode_coff(input: &str) -> Option<Symbol> {
    let inner = input.strip_prefix("__imp_")?;
    let inner_scheme = if inner.starts_with('?') {
        Scheme::MicrosoftCpp
    } else if inner.starts_with('@') {
        Scheme::Fastcall
    } else if inner.contains("@@") {
        Scheme::Vectorcall
    } else if inner.contains('@') {
        Scheme::Stdcall
    } else {
        Scheme::Cdecl
    };

    let mut symbol = crate::schemes::decode(inner_scheme, inner).ok()?;
    let inner_display = symbol.display();
    symbol.scheme = Scheme::CoffPe;
    symbol.kind = SymbolKind::Import;
    symbol.platform.import_prefix = true;
    symbol.platform.inner_scheme = Some(inner_scheme);
    symbol.display = Some(format!("import thunk for {inner_display}"));
    symbol.verbatim = Some(input.to_string());
    Some(symbol)
}

fn decode_elf(input: &str) -> Option<Symbol> {
    let (inner, version) = input.split_once("@@")?;
    let mut symbol = if inner.starts_with("_Z") {
        crate::schemes::itanium::decode(Scheme::Elf, inner)?
    } else {
        Symbol::new(Scheme::Elf, SymbolKind::Function).with_display(format!("{inner}@{version}"))
    };
    symbol.scheme = Scheme::Elf;
    symbol.platform = PlatformDecorations::default().with_elf_version(version);
    if inner.starts_with("_Z") {
        let inner_display = symbol.display();
        symbol.concrete_family = Scheme::ItaniumCpp;
        symbol.display = Some(format!("{inner_display}@{version}"));
    }
    symbol.verbatim = Some(input.to_string());
    Some(symbol)
}

fn strip_instance_suffix(input: &str) -> &str {
    let mut current = input;
    loop {
        let Some((head, tail)) = current.rsplit_once('_') else {
            return current;
        };
        if !tail.is_empty() && tail.chars().all(|ch| ch.is_ascii_digit()) {
            current = head;
            continue;
        }
        return current;
    }
}

fn strip_vfpthunk_suffix(input: &str) -> &str {
    input.strip_suffix("_vfpthunk_").unwrap_or(input)
}

fn macho_candidates(input: &str) -> Vec<&str> {
    let mut out = vec![strip_instance_suffix(input)];
    let vfpthunk = strip_vfpthunk_suffix(strip_instance_suffix(input));
    if !out.contains(&vfpthunk) {
        out.push(vfpthunk);
    }
    out
}

fn embedded_objc_method(input: &str) -> Option<&str> {
    let start = input.find("-[").or_else(|| input.find("+["))?;
    let suffix = &input[start..];
    let end = suffix.find(']')?;
    Some(&input[start..start + end + 1])
}
