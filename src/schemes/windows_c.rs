use crate::{Confidence, Scheme, Symbol, SymbolKind};

pub fn decode(scheme: Scheme, input: &str) -> Option<Symbol> {
    match scheme {
        Scheme::Cdecl => decode_cdecl(input),
        Scheme::Stdcall => decode_stdcall(input),
        Scheme::Fastcall => decode_fastcall(input),
        Scheme::Vectorcall => decode_vectorcall(input),
        _ => None,
    }
}

pub fn detect(input: &str) -> Option<(Scheme, Confidence)> {
    if input.starts_with("_Z")
        || input.starts_with("__Z")
        || input.starts_with("_D")
        || input.starts_with("__")
        || input.starts_with("__ct__")
        || input.starts_with("__dt__")
        || input.starts_with("__1c")
        || input.contains("::")
        || input.starts_with('`')
    {
        return None;
    }
    if input.starts_with('@') && input.rmatches('@').count() >= 2 && !input.contains('$') {
        return Some((Scheme::Fastcall, Confidence::Certain));
    }
    if let Some((_, suffix)) = input.rsplit_once("@@") {
        if suffix.chars().all(|ch| ch.is_ascii_digit()) && !input.starts_with('?') {
            return Some((Scheme::Vectorcall, Confidence::Certain));
        }
    }
    if input.starts_with('_') && input.contains('@') {
        return Some((Scheme::Stdcall, Confidence::Certain));
    }
    if input.starts_with('_') {
        return Some((
            Scheme::Cdecl,
            if input == "_f" {
                Confidence::Medium
            } else {
                Confidence::High
            },
        ));
    }
    None
}

fn decode_cdecl(input: &str) -> Option<Symbol> {
    let name = input.strip_prefix('_')?;
    Some(
        Symbol::new(Scheme::Cdecl, SymbolKind::Function)
            .with_display(name)
            .with_verbatim(input),
    )
}

fn decode_stdcall(input: &str) -> Option<Symbol> {
    let stripped = input.strip_prefix('_')?;
    let (name, _) = stripped.rsplit_once('@')?;
    Some(
        Symbol::new(Scheme::Stdcall, SymbolKind::Function)
            .with_display(name)
            .with_verbatim(input),
    )
}

fn decode_fastcall(input: &str) -> Option<Symbol> {
    let stripped = input.strip_prefix('@')?;
    let (name, _) = stripped.rsplit_once('@')?;
    Some(
        Symbol::new(Scheme::Fastcall, SymbolKind::Function)
            .with_display(name)
            .with_verbatim(input),
    )
}

fn decode_vectorcall(input: &str) -> Option<Symbol> {
    let (name, _) = input.rsplit_once("@@")?;
    Some(
        Symbol::new(Scheme::Vectorcall, SymbolKind::Function)
            .with_display(name)
            .with_verbatim(input),
    )
}
