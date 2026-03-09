use crate::{Confidence, Name, Scheme, Signature, Symbol, SymbolKind, Type};

pub fn decode(scheme: Scheme, input: &str) -> Option<Symbol> {
    let body = input.strip_prefix("Java_")?;
    let (qualified, overload) = body
        .split_once("__")
        .map_or((body, None), |(a, b)| (a, Some(b)));
    let parts = qualified
        .split('_')
        .map(decode_jni_ident)
        .collect::<Vec<_>>();
    if parts.is_empty() {
        return None;
    }

    let method_name = parts.last()?.clone();
    let mut symbol = Symbol::new(scheme, SymbolKind::Function);
    symbol.path = parts.into_iter().map(Name::identifier).collect();

    if let Some(overload) = overload {
        symbol.signature = Some(Signature {
            calling_convention: Some(crate::CallingConvention::C),
            parameters: parse_jni_signature(overload),
            return_type: None,
        });
        let owner = symbol.path[..symbol.path.len().saturating_sub(1)]
            .iter()
            .map(|name| match name {
                Name::Identifier(name) => name.clone(),
                Name::Template { name, .. } => name.clone(),
            })
            .collect::<Vec<_>>()
            .join(".");
        let params = symbol
            .signature
            .as_ref()
            .unwrap()
            .parameters
            .iter()
            .map(render_type)
            .collect::<Vec<_>>()
            .join(", ");
        return Some(
            symbol
                .with_display(format!("{owner}.{method_name}({params})"))
                .with_verbatim(input),
        );
    }

    let display = symbol
        .path
        .iter()
        .map(|name| match name {
            Name::Identifier(name) => name.clone(),
            Name::Template { name, .. } => name.clone(),
        })
        .collect::<Vec<_>>()
        .join(".");
    Some(symbol.with_display(display).with_verbatim(input))
}

pub fn detect(input: &str) -> Option<(Scheme, Confidence)> {
    input
        .starts_with("Java_")
        .then_some((Scheme::Jni, Confidence::Certain))
}

fn decode_jni_ident(input: &str) -> String {
    input
        .replace("_1", "_")
        .replace("_2", ";")
        .replace("_3", "[")
}

fn parse_jni_signature(mut input: &str) -> Vec<Type> {
    let mut out = Vec::new();
    while !input.is_empty() {
        if let Some(rest) = input.strip_prefix('I') {
            out.push(Type::int());
            input = rest;
            continue;
        }
        if let Some(rest) = input.strip_prefix("Ljava_lang_String_2") {
            out.push(Type::named(["java", "lang", "String"]));
            input = rest;
            continue;
        }
        if let Some(rest) = input.strip_prefix("Ljava_lang_Object_2") {
            out.push(Type::named(["java", "lang", "Object"]));
            input = rest;
            continue;
        }
        break;
    }
    out
}

fn render_type(ty: &Type) -> String {
    match ty {
        Type::Int => "int".to_string(),
        Type::Named(parts) => parts.join("."),
        other => format!("{other:?}"),
    }
}
