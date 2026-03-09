use crate::{text, Confidence, Scheme, SpecialKind, Symbol, SymbolKind};
use msvc_demangler::DemangleFlags;

pub fn decode(scheme: Scheme, input: &str) -> Option<Symbol> {
    if !input.starts_with('?') {
        return None;
    }

    if input.starts_with("??_R0") {
        let path_text = parse_rtti_type_descriptor(input)?;
        let mut symbol = Symbol::special(
            scheme,
            SpecialKind::RttiTypeDescriptor,
            std::iter::empty::<&str>(),
        );
        symbol.concrete_family = Scheme::MicrosoftCpp;
        symbol.path = text::parse_names(&path_text, "::");
        return Some(
            symbol
                .with_display(format!("RTTI Type Descriptor for {path_text}"))
                .with_verbatim(input),
        );
    }

    if let Some(full) = msvc_demangler::demangle(input, DemangleFlags::llvm()).ok() {
        if input.starts_with("??_7") {
            let path_text = extract_before_marker(&full, "::`vftable'")?;
            let mut symbol =
                Symbol::special(scheme, SpecialKind::Vftable, std::iter::empty::<&str>());
            symbol.concrete_family = Scheme::MicrosoftCpp;
            symbol.path = text::parse_names(path_text, "::");
            return Some(
                symbol
                    .with_display(format!("vftable for {path_text}"))
                    .with_verbatim(input),
            );
        }

        if let Some(name_only) = msvc_demangler::demangle(input, DemangleFlags::NAME_ONLY).ok() {
            let display = full
                .find(&name_only)
                .map(|index| full[index..].to_string())
                .unwrap_or(name_only);
            let display = normalize_msvc_display(&display);

            let mut symbol = text::symbol_from_demangled_cpp(
                Symbol::new(scheme, SymbolKind::Function),
                &display,
                "::",
            );
            symbol.concrete_family = Scheme::MicrosoftCpp;
            return Some(symbol.with_verbatim(input));
        }
    }

    fallback_decode(scheme, input)
}

pub fn detect(input: &str) -> Option<(Scheme, Confidence)> {
    input
        .starts_with('?')
        .then_some((Scheme::MicrosoftCpp, Confidence::Certain))
}

fn extract_before_marker<'a>(input: &'a str, marker: &str) -> Option<&'a str> {
    let index = input.find(marker)?;
    Some(strip_msvc_type_prefix(&input[..index]))
}

fn strip_msvc_type_prefix(input: &str) -> &str {
    input
        .strip_prefix("const ")
        .or_else(|| input.strip_prefix("struct "))
        .or_else(|| input.strip_prefix("class "))
        .or_else(|| input.strip_prefix("union "))
        .or_else(|| input.strip_prefix("enum "))
        .unwrap_or(input)
}

fn normalize_msvc_display(display: &str) -> String {
    display
        .replace("(void)", "()")
        .replace("`anonymous namespace'", "anonymous namespace")
}

fn parse_rtti_type_descriptor(input: &str) -> Option<String> {
    let input = strip_numeric_clone_suffix(input);
    let body = input.strip_prefix("??_R0?A")?;
    let body = body
        .strip_prefix('U')
        .or_else(|| body.strip_prefix('V'))
        .or_else(|| body.strip_prefix('W'))
        .unwrap_or(body);
    let body = body.strip_suffix("@@@8")?;
    let mut parts = body
        .split('@')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();
    parts.reverse();
    Some(parts.join("::"))
}

fn strip_numeric_clone_suffix(input: &str) -> &str {
    let Some((head, tail)) = input.rsplit_once('_') else {
        return input;
    };
    if !tail.is_empty() && tail.chars().all(|ch| ch.is_ascii_digit()) {
        head
    } else {
        input
    }
}

fn fallback_decode(scheme: Scheme, input: &str) -> Option<Symbol> {
    let tokens = extract_raw_tokens(input);
    if tokens.is_empty() {
        return None;
    }

    let (kind, display, path) = if input.starts_with("??R") && tokens.len() >= 2 {
        let mut path = vec![
            tokens[1].clone(),
            tokens[0].clone(),
            "operator()".to_string(),
        ];
        let display = path.join("::");
        let path = path
            .drain(..)
            .map(crate::Name::identifier)
            .collect::<Vec<_>>();
        (SymbolKind::Method, display, path)
    } else {
        let name = tokens.first()?.clone();
        let mut scopes = tokens[1..].to_vec();
        scopes.reverse();
        scopes.push(name);
        let display = scopes.join("::");
        let kind = if scopes.len() >= 2 {
            SymbolKind::Method
        } else {
            SymbolKind::Function
        };
        let path = scopes
            .into_iter()
            .map(crate::Name::identifier)
            .collect::<Vec<_>>();
        (kind, display, path)
    };

    let mut symbol = Symbol::new(scheme, kind);
    symbol.concrete_family = Scheme::MicrosoftCpp;
    symbol.path = path;
    Some(symbol.with_display(display).with_verbatim(input))
}

fn extract_raw_tokens(input: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    for segment in input.split('@') {
        if !tokens.is_empty() && starts_type_encoding(segment) {
            break;
        }
        if let Some(token) = normalize_raw_segment(segment) {
            tokens.push(token);
            continue;
        }
        if !tokens.is_empty() && looks_like_stop_segment(segment) {
            break;
        }
    }
    tokens
}

fn normalize_raw_segment(segment: &str) -> Option<String> {
    let trimmed = segment.trim_matches('?');
    if trimmed.is_empty() {
        return None;
    }

    let mut tail = trimmed;
    if let Some((_, stripped)) = tail.rsplit_once('$') {
        tail = stripped;
    }
    if let Some((_, stripped)) = tail.rsplit_once('?') {
        tail = stripped;
    }
    let tail = strip_storage_prefixes(tail);
    let tail = tail.strip_prefix("R_").unwrap_or(tail);

    if tail.is_empty() || tail.chars().all(|ch| ch.is_ascii_digit()) {
        return None;
    }
    if tail
        .chars()
        .all(|ch| ch.is_ascii_uppercase() || ch.is_ascii_digit())
    {
        return None;
    }
    Some(tail.to_string())
}

fn strip_storage_prefixes(mut input: &str) -> &str {
    for prefix in [
        "AEBV", "AEBU", "AEBW", "PEAV", "PEAU", "PEAW", "PEBV", "P6", "QEAA", "QEBA", "UEAA",
        "UEBA", "YA", "CA", "EA", "SA", "MA", "V", "U", "W",
    ] {
        if input.starts_with(prefix) && input.len() > prefix.len() {
            input = &input[prefix.len()..];
            break;
        }
    }
    input
}

fn looks_like_stop_segment(segment: &str) -> bool {
    let trimmed = segment.trim_matches('?');
    if trimmed.is_empty() {
        return false;
    }
    let compact = trimmed
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .collect::<String>();
    compact.len() >= 4
        && compact
            .chars()
            .all(|ch| ch.is_ascii_uppercase() || ch.is_ascii_digit())
}

fn starts_type_encoding(segment: &str) -> bool {
    let trimmed = segment.trim_matches('?');
    if trimmed.is_empty() {
        return false;
    }
    if trimmed.starts_with('P')
        && trimmed
            .chars()
            .all(|ch| ch.is_ascii_uppercase() || ch.is_ascii_digit())
    {
        return true;
    }
    ["Y", "Q", "A", "C", "E", "S", "M", "U"]
        .iter()
        .any(|prefix| {
            trimmed.starts_with(prefix)
                && (trimmed.contains("PEA")
                    || trimmed.contains("AEB")
                    || trimmed.contains("QEA")
                    || trimmed.contains("YA")
                    || trimmed.contains("P6")
                    || trimmed.contains('?')
                    || trimmed.contains('$'))
        })
}
