use crate::{Name, Symbol, SymbolKind, Type};

pub fn split_qualified(input: &str, separator: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut start = 0usize;
    let mut angle = 0usize;
    let mut paren = 0usize;
    let mut brace = 0usize;
    let mut i = 0usize;

    while i < input.len() {
        let rest = &input[i..];
        if angle == 0 && paren == 0 && brace == 0 && rest.starts_with(separator) {
            parts.push(input[start..i].to_string());
            i += separator.len();
            start = i;
            continue;
        }

        let ch = rest.chars().next().unwrap();
        match ch {
            '<' => angle += 1,
            '>' => angle = angle.saturating_sub(1),
            '(' => paren += 1,
            ')' => paren = paren.saturating_sub(1),
            '{' => brace += 1,
            '}' => brace = brace.saturating_sub(1),
            _ => {}
        }
        i += ch.len_utf8();
    }

    if start <= input.len() {
        parts.push(input[start..].to_string());
    }

    parts.into_iter().filter(|part| !part.is_empty()).collect()
}

pub fn symbol_from_demangled_cpp(
    mut symbol: Symbol,
    display: &str,
    path_separator: &str,
) -> Symbol {
    if let Some(rest) = display.strip_prefix("vtable for ") {
        symbol.kind = SymbolKind::VTable;
        symbol.path = parse_names(rest, path_separator);
        return symbol.with_display(display);
    }

    if let Some(rest) = display.strip_prefix("typeinfo for ") {
        symbol.kind = SymbolKind::Metadata;
        symbol.path = parse_names(rest, path_separator);
        return symbol.with_display(display);
    }

    if let Some(rest) = display
        .strip_prefix("non-virtual thunk to ")
        .or_else(|| display.strip_prefix("virtual thunk to "))
    {
        symbol.kind = SymbolKind::Thunk;
        let path = split_signature(rest).map(|(path, _)| path).unwrap_or(rest);
        symbol.path = parse_names(path, path_separator);
        return symbol.with_display(display);
    }

    if let Some((path, params)) = split_signature(display) {
        let names = parse_names(path, path_separator);
        symbol.kind = classify_member_kind(&names);
        symbol.path = names;
        symbol.signature = Some(crate::Signature {
            calling_convention: None,
            parameters: parse_params(params, path_separator),
            return_type: None,
        });
        return symbol.with_display(display);
    }

    symbol.kind = classify_member_kind(&parse_names(display, path_separator));
    symbol.path = parse_names(display, path_separator);
    symbol.with_display(display)
}

pub fn symbol_from_qualified_display(
    mut symbol: Symbol,
    display: &str,
    path_separator: &str,
) -> Symbol {
    symbol.path = parse_names(display, path_separator);
    symbol.kind = classify_member_kind(&symbol.path);
    symbol.with_display(display)
}

pub fn parse_names(input: &str, separator: &str) -> Vec<Name> {
    split_qualified(input, separator)
        .into_iter()
        .map(|part| parse_name(&part, separator))
        .collect()
}

pub fn parse_name(input: &str, separator: &str) -> Name {
    let trimmed = input.trim();
    if let Some((base, args)) = split_template(trimmed) {
        let args = split_qualified(args, ",")
            .into_iter()
            .map(|arg| parse_type(arg.trim(), separator))
            .collect();
        Name::template(base, args)
    } else {
        Name::identifier(trimmed)
    }
}

pub fn parse_type(input: &str, separator: &str) -> Type {
    let trimmed = input.trim();
    match trimmed {
        "void" | "()" => Type::void(),
        "int" | "Swift.Int" => Type::int(),
        "char" => Type::Other("char".to_string()),
        "bool" => Type::Other("bool".to_string()),
        "float" => Type::Other("float".to_string()),
        "double" => Type::Other("double".to_string()),
        _ if trimmed.ends_with(" const&") => {
            let inner = trimmed.trim_end_matches(" const&");
            Type::const_ref(parse_type(inner, separator))
        }
        _ if trimmed.contains(separator) => Type::named(split_qualified(trimmed, separator)),
        _ => Type::Other(trimmed.to_string()),
    }
}

pub fn split_signature(display: &str) -> Option<(&str, &str)> {
    let mut depth = 0usize;
    for (index, ch) in display.char_indices() {
        match ch {
            '<' => depth += 1,
            '>' => depth = depth.saturating_sub(1),
            '(' if depth == 0 => {
                let end = find_matching_paren(display, index)?;
                return Some((&display[..index], &display[index + 1..end]));
            }
            _ => {}
        }
    }
    None
}

pub fn parse_params(params: &str, separator: &str) -> Vec<Type> {
    let trimmed = params.trim();
    if trimmed.is_empty() || trimmed == "void" {
        return Vec::new();
    }
    split_qualified(trimmed, ",")
        .into_iter()
        .map(|param| parse_type(param.trim(), separator))
        .collect()
}

fn split_template(input: &str) -> Option<(&str, &str)> {
    let start = input.find('<')?;
    let end = input.rfind('>')?;
    if end <= start {
        return None;
    }
    Some((&input[..start], &input[start + 1..end]))
}

fn find_matching_paren(input: &str, open_index: usize) -> Option<usize> {
    let mut depth = 0usize;
    for (index, ch) in input[open_index..].char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some(open_index + index);
                }
            }
            _ => {}
        }
    }
    None
}

fn classify_member_kind(path: &[Name]) -> SymbolKind {
    if path.len() >= 2 {
        if let (Some(last), Some(prev)) = (path.last(), path.get(path.len() - 2)) {
            if let (Name::Identifier(last), Name::Identifier(prev)) = (last, prev) {
                if last == prev {
                    return SymbolKind::Constructor;
                }
                if last.starts_with('~') {
                    return SymbolKind::Destructor;
                }
            }
        }
    }
    if path.len() >= 3 {
        SymbolKind::Method
    } else {
        SymbolKind::Function
    }
}
