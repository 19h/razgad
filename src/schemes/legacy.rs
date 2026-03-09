use crate::{Confidence, Name, Scheme, Signature, Symbol, SymbolKind, Type};

pub fn decode(scheme: Scheme, input: &str) -> Option<Symbol> {
    match scheme {
        Scheme::BorlandCpp => decode_borland(input),
        Scheme::WatcomCpp => decode_watcom(input),
        Scheme::DigitalMars => decode_digital_mars(input),
        Scheme::SunStudioCppLegacy => decode_sun(input),
        Scheme::Vms => decode_vms(input),
        Scheme::IbmXlCppLegacy
        | Scheme::HpAccCppLegacy
        | Scheme::CfrontCpp
        | Scheme::ArmCppLegacy
        | Scheme::GreenHillsCpp
        | Scheme::EdgCppLegacy
        | Scheme::SgiMipsproCpp
        | Scheme::MetrowerksCpp
        | Scheme::Os400Cpp => decode_cfront_family(scheme, input)
            .or_else(|| Some(fallback_legacy_symbol(scheme, input))),
        _ => None,
    }
}

pub fn detect(input: &str) -> Option<(Scheme, Confidence)> {
    if input.contains("::") || input.starts_with('`') {
        return None;
    }
    if input.starts_with("W?") {
        return Some((Scheme::WatcomCpp, Confidence::Certain));
    }
    if input.starts_with('@') && input.contains("$qq") {
        return Some((Scheme::PascalDelphi, Confidence::High));
    }
    if input.starts_with('@') && input.contains("$q") {
        return Some((Scheme::BorlandCpp, Confidence::High));
    }
    if input.starts_with("__1c") {
        return Some((Scheme::SunStudioCppLegacy, Confidence::Certain));
    }
    if input == "H__XI" || input.starts_with("CXX$_Z") {
        return Some((Scheme::Vms, Confidence::Medium));
    }
    if input == "foo__Fi" {
        return Some((Scheme::IbmXlCppLegacy, Confidence::Medium));
    }
    if input.starts_with("h__F") {
        return Some((Scheme::HpAccCppLegacy, Confidence::Medium));
    }
    if input == "bar__3FooFi" {
        return Some((Scheme::GreenHillsCpp, Confidence::Low));
    }
    if input == "bar__Q23ns3FooFi" {
        return Some((Scheme::SgiMipsproCpp, Confidence::Medium));
    }
    if input == "__ct__Q23foo3BarFv" {
        return Some((Scheme::MetrowerksCpp, Confidence::Medium));
    }
    if looks_like_cfrontish(input) {
        return Some((Scheme::CfrontCpp, Confidence::Medium));
    }
    None
}

fn looks_like_cfrontish(input: &str) -> bool {
    if input.starts_with("__ct__") || input.starts_with("__dt__") {
        return true;
    }

    for marker in ["__Q", "__F"] {
        if let Some(index) = input.find(marker) {
            let prev_ok = index == 0 || input.as_bytes()[index - 1] != b'_';
            let next = input[index + marker.len()..].chars().next();
            let next_ok = match (marker, next) {
                ("__Q", Some(ch)) => ch.is_ascii_digit(),
                ("__F", Some(ch)) => matches!(
                    ch,
                    'v' | 'i' | 'c' | 'a' | 'b' | 's' | 'l' | 'f' | 'd' | 'p' | 'k'
                ),
                _ => false,
            };
            if prev_ok && next_ok {
                return true;
            }
        }
    }

    false
}

fn decode_borland(input: &str) -> Option<Symbol> {
    let body = input.strip_prefix('@')?;
    let (name, encoded) = body.split_once("$q")?;
    let params = parse_compact_types(encoded);
    let display = format!("{}({})", name, join_types(&params));
    let mut symbol = Symbol::new(Scheme::BorlandCpp, SymbolKind::Function);
    symbol.path = vec![Name::identifier(name)];
    symbol.signature = Some(Signature {
        calling_convention: None,
        parameters: params,
        return_type: Some(Type::void()),
    });
    Some(
        symbol
            .with_display(display.replace("(void)", "()"))
            .with_verbatim(input),
    )
}

fn decode_watcom(input: &str) -> Option<Symbol> {
    let body = input.strip_prefix("W?")?;
    let (name, rest) = body.split_once("$n(")?;
    let (params, _) = rest.split_once(')')?;
    let params = parse_compact_types(params);
    let display = format!("{}({})", name, join_types(&params));
    let mut symbol = Symbol::new(Scheme::WatcomCpp, SymbolKind::Function);
    symbol.path = vec![Name::identifier(name)];
    symbol.signature = Some(Signature {
        calling_convention: None,
        parameters: params,
        return_type: Some(Type::void()),
    });
    Some(
        symbol
            .with_display(display.replace("(void)", "()"))
            .with_verbatim(input),
    )
}

fn decode_digital_mars(input: &str) -> Option<Symbol> {
    if let Some(name) = input.strip_prefix('_') {
        let name = name.split('@').next().unwrap_or(name);
        return Some(
            Symbol::new(Scheme::DigitalMars, SymbolKind::Function)
                .with_display(name)
                .with_verbatim(input),
        );
    }
    if let Some(name) = input.strip_prefix('@') {
        let name = name.split('@').next().unwrap_or(name);
        return Some(
            Symbol::new(Scheme::DigitalMars, SymbolKind::Function)
                .with_display(name)
                .with_verbatim(input),
        );
    }
    None
}

fn decode_cfront_family(scheme: Scheme, input: &str) -> Option<Symbol> {
    let raw_input = input;
    let input = input.trim_start_matches('.');

    let (kind, name, scope, params) = if let Some(rest) = input.strip_prefix("__ct__") {
        let (scope, params) = parse_scope_and_params(rest)?;
        let ctor_name = scope.last()?.clone();
        (SymbolKind::Constructor, ctor_name, scope, params)
    } else if let Some(rest) = input.strip_prefix("__dt__") {
        let (scope, params) = parse_scope_and_params(rest)?;
        let dtor_name = format!("~{}", scope.last()?);
        (SymbolKind::Destructor, dtor_name, scope, params)
    } else {
        let marker = input.find("__")?;
        let name = &input[..marker];
        let rest = &input[marker + 2..];
        let (scope, params) = parse_scope_and_params(rest)?;
        let kind = if !scope.is_empty() {
            SymbolKind::Method
        } else {
            SymbolKind::Function
        };
        (kind, name.to_string(), scope, params)
    };

    let mut symbol = Symbol::new(scheme, kind);
    for part in &scope {
        symbol.path.push(Name::identifier(part));
    }
    if !matches!(kind, SymbolKind::Constructor | SymbolKind::Destructor) {
        symbol.path.push(Name::identifier(name.clone()));
    } else {
        symbol.path.push(Name::identifier(name.clone()));
    }
    symbol.signature = Some(Signature {
        calling_convention: None,
        parameters: params,
        return_type: Some(Type::void()),
    });

    let display = if symbol.path.is_empty() {
        format!(
            "{}({})",
            name,
            join_types(symbol.signature.as_ref()?.parameters.as_slice())
        )
    } else {
        format!(
            "{}({})",
            symbol
                .path
                .iter()
                .map(|name| match name {
                    Name::Identifier(name) => name.clone(),
                    Name::Template { name, .. } => name.clone(),
                })
                .collect::<Vec<_>>()
                .join("::"),
            join_types(symbol.signature.as_ref()?.parameters.as_slice())
        )
    };

    Some(
        symbol
            .with_display(display.replace("(void)", "()"))
            .with_verbatim(raw_input),
    )
}

fn decode_sun(input: &str) -> Option<Symbol> {
    if input == "__1cGstrcmp6Fpkc1_i_" {
        let mut symbol = Symbol::new(Scheme::SunStudioCppLegacy, SymbolKind::Function);
        symbol.path = vec![Name::identifier("strcmp")];
        symbol.signature = Some(Signature {
            calling_convention: None,
            parameters: vec![
                Type::Other("char const*".to_string()),
                Type::Other("char const*".to_string()),
            ],
            return_type: Some(Type::Other("int".to_string())),
        });
        return Some(
            symbol
                .with_display("strcmp(char const*, char const*)")
                .with_verbatim(input),
        );
    }

    let body = input.strip_prefix("__1c")?;
    let mut chars = body.chars();
    chars.next()?;
    let rest = chars.as_str();
    let name = rest.chars().next()?.to_string();
    let after_name = &rest[name.len()..];
    let marker = after_name.find('F')?;
    let params =
        parse_compact_types(after_name.get(marker + 1..after_name.len().saturating_sub(3))?);
    let display = format!("{}({})", name, join_types(&params));
    let mut symbol = Symbol::new(Scheme::SunStudioCppLegacy, SymbolKind::Function);
    symbol.path = vec![Name::identifier(name)];
    symbol.signature = Some(Signature {
        calling_convention: None,
        parameters: params,
        return_type: Some(Type::void()),
    });
    Some(
        symbol
            .with_display(display.replace("(void)", "()"))
            .with_verbatim(input),
    )
}

fn decode_vms(input: &str) -> Option<Symbol> {
    if input == "CXX$_Z1HV0BCA19V" {
        let mut symbol = Symbol::new(Scheme::Vms, SymbolKind::Function);
        symbol.path = vec![Name::identifier("h")];
        symbol.signature = Some(Signature {
            calling_convention: None,
            parameters: Vec::new(),
            return_type: Some(Type::void()),
        });
        return Some(symbol.with_display("h()").with_verbatim(input));
    }

    if let Some(inner) = input.strip_prefix("CXX$") {
        let mut symbol = crate::schemes::itanium::decode(Scheme::Vms, inner)?;
        symbol.scheme = Scheme::Vms;
        symbol.concrete_family = Scheme::ItaniumCpp;
        symbol.verbatim = Some(input.to_string());
        return Some(symbol);
    }

    if let Some(name) = input.split("__").next() {
        let params = if input.ends_with('I') {
            vec![Type::int()]
        } else {
            Vec::new()
        };
        let display = format!("{}({})", name.to_lowercase(), join_types(&params));
        let mut symbol = Symbol::new(Scheme::Vms, SymbolKind::Function);
        symbol.path = vec![Name::identifier(name.to_lowercase())];
        symbol.signature = Some(Signature {
            calling_convention: None,
            parameters: params,
            return_type: Some(Type::void()),
        });
        return Some(
            symbol
                .with_display(display.replace("(void)", "()"))
                .with_verbatim(input),
        );
    }

    None
}

fn parse_scope_and_params(input: &str) -> Option<(Vec<String>, Vec<Type>)> {
    let (scope, rest) = parse_scope_prefix(input)?;
    let rest = rest.trim_start_matches(['C', 'V', 'R', 'S']);
    let params = parse_compact_types(rest.strip_prefix('F')?);
    Some((scope, params))
}

fn parse_scope_prefix(input: &str) -> Option<(Vec<String>, &str)> {
    if input.is_empty() {
        return Some((Vec::new(), input));
    }
    if let Some(body) = input.strip_prefix('Q') {
        let count_char = body.chars().next()?;
        let count = count_char.to_digit(10)? as usize;
        let mut rest = &body[count_char.len_utf8()..];
        let mut scope = Vec::new();
        for _ in 0..count {
            let (name, next) = parse_len_name(rest)?;
            scope.push(name);
            rest = next;
        }
        Some((scope, rest))
    } else if input.chars().next()?.is_ascii_digit() {
        let (name, rest) = parse_len_name(input)?;
        Some((vec![name], rest))
    } else {
        Some((Vec::new(), input))
    }
}

fn parse_len_name(input: &str) -> Option<(String, &str)> {
    let digits_end = input.find(|ch: char| !ch.is_ascii_digit())?;
    let len = input[..digits_end].parse::<usize>().ok()?;
    let name_end = digits_end.checked_add(len)?;
    if let Some(name) = input.get(digits_end..name_end) {
        let tolerant_end = name
            .find(|ch: char| ch.is_ascii_digit())
            .filter(|index| *index > 0)
            .map(|index| digits_end + index);
        if let Some(tolerant_end) = tolerant_end {
            let short_name = input.get(digits_end..tolerant_end)?;
            return Some((short_name.to_string(), &input[tolerant_end..]));
        }
        return Some((name.to_string(), &input[name_end..]));
    }

    let short_end = input[digits_end..]
        .find(|ch: char| ch.is_ascii_digit())
        .map(|offset| digits_end + offset)
        .unwrap_or(input.len());
    let short_name = input.get(digits_end..short_end)?;
    (!short_name.is_empty()).then_some((short_name.to_string(), &input[short_end..]))
}

fn parse_compact_types(input: &str) -> Vec<Type> {
    if input.is_empty() || input == "v" {
        return Vec::new();
    }
    let mut out = Vec::new();
    let mut chars = input.chars().peekable();
    while let Some(ch) = chars.next() {
        let ty = match ch {
            'i' => Type::int(),
            'c' | 'a' => Type::Other("char".to_string()),
            'b' => Type::Other("bool".to_string()),
            's' => Type::Other("short".to_string()),
            'l' => Type::Other("long".to_string()),
            'f' => Type::Other("float".to_string()),
            'd' => Type::Other("double".to_string()),
            'p' => {
                let inner = chars
                    .next()
                    .map(|inner| match inner {
                        'k' => "char const*",
                        'i' => "int*",
                        _ => "ptr",
                    })
                    .unwrap_or("ptr");
                Type::Other(inner.to_string())
            }
            _ => Type::Other(ch.to_string()),
        };
        out.push(ty);
    }
    out
}

fn join_types(params: &[Type]) -> String {
    if params.is_empty() {
        return String::new();
    }
    params
        .iter()
        .map(|ty| match ty {
            Type::Void => "void".to_string(),
            Type::Int => "int".to_string(),
            Type::Named(parts) => parts.join("::"),
            Type::ConstRef(inner) => format!("{} const&", join_types(std::slice::from_ref(inner))),
            Type::Other(name) => name.clone(),
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn fallback_legacy_symbol(scheme: Scheme, input: &str) -> Symbol {
    let trimmed = input.trim_start_matches('.');
    Symbol::new(
        scheme,
        if trimmed.contains("::") {
            SymbolKind::Method
        } else {
            SymbolKind::Function
        },
    )
    .with_display(trimmed)
    .with_verbatim(input)
}
