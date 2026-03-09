use crate::{CallingConvention, Confidence, Name, Scheme, Signature, Symbol, SymbolKind, Type};

pub fn decode(scheme: Scheme, input: &str) -> Option<Symbol> {
    let mut cursor = input.strip_prefix("_D")?;
    if !cursor.chars().next()?.is_ascii_digit() || input.contains('@') {
        return None;
    }
    let mut path = Vec::new();

    while !cursor.is_empty() && !cursor.starts_with('F') {
        let (name, rest) = parse_len_name(cursor)?;
        path.push(Name::identifier(name));
        cursor = rest;
    }

    let mut symbol = Symbol::new(scheme, SymbolKind::Function);
    symbol.path = path;

    if let Some(rest) = cursor.strip_prefix('F') {
        let (params, rest) = parse_params(rest)?;
        let return_type = parse_type(rest)?;
        symbol.signature = Some(Signature {
            calling_convention: Some(CallingConvention::D),
            parameters: params,
            return_type: Some(return_type),
        });
    }

    Some(symbol.with_verbatim(input))
}

pub fn detect(input: &str) -> Option<(Scheme, Confidence)> {
    decode(Scheme::Dlang, input)
        .is_some()
        .then_some((Scheme::Dlang, Confidence::Certain))
}

fn parse_len_name(input: &str) -> Option<(String, &str)> {
    let digits_end = input.find(|ch: char| !ch.is_ascii_digit())?;
    let len = input[..digits_end].parse::<usize>().ok()?;
    let name_start = digits_end;
    let name_end = name_start.checked_add(len)?;
    let name = input.get(name_start..name_end)?;
    Some((name.to_string(), &input[name_end..]))
}

fn parse_params(mut input: &str) -> Option<(Vec<Type>, &str)> {
    let mut params = Vec::new();
    while !input.is_empty() && !input.starts_with('Z') {
        let ty = parse_one_type(&mut input)?;
        params.push(ty);
    }
    let rest = input.strip_prefix('Z')?;
    Some((params, rest))
}

fn parse_type(input: &str) -> Option<Type> {
    let mut input = input;
    let ty = parse_one_type(&mut input)?;
    input.is_empty().then_some(ty)
}

fn parse_one_type(input: &mut &str) -> Option<Type> {
    if let Some(rest) = input.strip_prefix('i') {
        *input = rest;
        return Some(Type::int());
    }
    if let Some(rest) = input.strip_prefix('v') {
        *input = rest;
        return Some(Type::void());
    }
    if let Some(rest) = input.strip_prefix('a') {
        *input = rest;
        return Some(Type::Other("char".to_string()));
    }
    if let Some(rest) = input.strip_prefix('b') {
        *input = rest;
        return Some(Type::Other("bool".to_string()));
    }
    None
}
