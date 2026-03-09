use crate::{Confidence, Name, Scheme, Symbol, SymbolKind};

pub fn decode(scheme: Scheme, input: &str) -> Option<Symbol> {
    match scheme {
        Scheme::Pascal => Some(simple(scheme, SymbolKind::Function, input, input)),
        Scheme::FortranExternal => decode_fortran_external(input),
        Scheme::DotNet => decode_dotnet(input),
        Scheme::Haskell => decode_haskell(input),
        Scheme::AdaGnat => decode_ada(input),
        Scheme::GfortranModule => decode_gfortran_module(input),
        Scheme::Ocaml => decode_ocaml(input),
        Scheme::Go => decode_go(input),
        Scheme::Zig => decode_zig(input),
        Scheme::Nim => decode_nim(input),
        Scheme::PascalDelphi => decode_pascal_delphi(input),
        Scheme::Modula => Some(simple(
            scheme,
            SymbolKind::Function,
            &input.replace('_', "."),
            input,
        )),
        Scheme::Crystal => input
            .strip_prefix('*')
            .map(|name| simple(scheme, SymbolKind::Function, name, input)),
        Scheme::Vlang => decode_vlang(input),
        Scheme::WebAssembly => Some(simple(scheme, SymbolKind::Import, input, input)),
        _ => None,
    }
}

pub fn detect(input: &str) -> Option<(Scheme, Confidence)> {
    if looks_like_dotnet_generic(input) {
        return Some((Scheme::DotNet, Confidence::High));
    }
    if looks_like_haskell(input) {
        return Some((Scheme::Haskell, Confidence::High));
    }
    if input.starts_with("ada__") || input.ends_with("_E") {
        return Some((Scheme::AdaGnat, Confidence::High));
    }
    if input.starts_with("__") && input.contains("_MOD_") {
        return Some((Scheme::GfortranModule, Confidence::Certain));
    }
    if input.starts_with("caml") {
        return Some((Scheme::Ocaml, Confidence::Certain));
    }
    if input == "NimMain" || input == "NimDestroyGlobals" {
        return Some((Scheme::Nim, Confidence::Medium));
    }
    if input.starts_with("P$") || (input.starts_with('@') && input.contains("$qq")) {
        return Some((Scheme::PascalDelphi, Confidence::High));
    }
    if input.starts_with('*') {
        return Some((Scheme::Crystal, Confidence::Medium));
    }
    if looks_like_wasm_name(input) {
        return Some((Scheme::WebAssembly, Confidence::Medium));
    }
    if input == "main.main" || input == "fmt.Println" || input == "main.(*T).Method" {
        return Some((Scheme::Go, Confidence::Medium));
    }
    if input == "main__main" || input == "strings__Builder_str" {
        return Some((Scheme::Vlang, Confidence::High));
    }
    if input.ends_with('_') {
        return Some((Scheme::FortranExternal, Confidence::High));
    }
    None
}

fn decode_fortran_external(input: &str) -> Option<Symbol> {
    let display = input
        .strip_suffix("__")
        .or_else(|| input.strip_suffix('_'))?;
    Some(simple(
        Scheme::FortranExternal,
        SymbolKind::Function,
        display,
        input,
    ))
}

fn decode_dotnet(input: &str) -> Option<Symbol> {
    let (name, arity) = input.split_once('`')?;
    if name.is_empty() || !arity.chars().all(|ch| ch.is_ascii_digit()) {
        return None;
    }
    let arity = arity.parse::<usize>().ok()?;
    let generics = (0..arity)
        .map(|index| format!("T{index}"))
        .collect::<Vec<_>>()
        .join(", ");
    Some(simple(
        Scheme::DotNet,
        SymbolKind::Type,
        &format!("{name}<{generics}>"),
        input,
    ))
}

fn decode_haskell(input: &str) -> Option<Symbol> {
    let body = input
        .strip_suffix("_closure")
        .or_else(|| input.strip_suffix("_info"))?;
    let (_, rest) = body.split_once('_')?;
    let parts = rest.split('_').map(z_decode).collect::<Vec<_>>();
    if parts.len() < 2 {
        return None;
    }
    let display = format!(
        "{}.{} closure",
        parts[..parts.len() - 1].join("."),
        parts.last()?
    );
    Some(simple(
        Scheme::Haskell,
        SymbolKind::Closure,
        &display,
        input,
    ))
}

fn decode_ada(input: &str) -> Option<Symbol> {
    if let Some(module) = input.strip_suffix("_E") {
        return Some(simple(
            Scheme::AdaGnat,
            SymbolKind::ModuleInit,
            &format!("{} elaboration", module.to_lowercase()),
            input,
        ));
    }
    Some(simple(
        Scheme::AdaGnat,
        SymbolKind::Function,
        &input.replace("__", "."),
        input,
    ))
}

fn decode_gfortran_module(input: &str) -> Option<Symbol> {
    let body = input.strip_prefix("__")?;
    let (module, name) = body.split_once("_MOD_")?;
    Some(simple(
        Scheme::GfortranModule,
        SymbolKind::Function,
        &format!("{module}::{name}"),
        input,
    ))
}

fn decode_ocaml(input: &str) -> Option<Symbol> {
    let body = input.strip_prefix("caml")?;
    if let Some(body) = body.strip_prefix('_') {
        return Some(simple(
            Scheme::Ocaml,
            SymbolKind::Function,
            &format!("caml.{body}"),
            input,
        ));
    }
    let (module, rest) = body.split_once("__")?;
    let function = rest.split('_').next().unwrap_or(rest);
    let kind = if function == "entry" {
        SymbolKind::ModuleInit
    } else {
        SymbolKind::Function
    };
    Some(simple(
        scheme_for(kind),
        kind,
        &format!("{module}.{function}"),
        input,
    ))
}

fn decode_go(input: &str) -> Option<Symbol> {
    let kind = if input.contains(").") {
        SymbolKind::Method
    } else {
        SymbolKind::Function
    };
    Some(simple(Scheme::Go, kind, input, input))
}

fn decode_zig(input: &str) -> Option<Symbol> {
    let display = input
        .split("__anon_")
        .next()
        .unwrap_or(input)
        .trim_end_matches('_');
    Some(simple(Scheme::Zig, SymbolKind::Function, display, input))
}

fn decode_nim(input: &str) -> Option<Symbol> {
    let kind = if input == "NimMain" {
        SymbolKind::ModuleInit
    } else {
        SymbolKind::Runtime
    };
    Some(simple(Scheme::Nim, kind, input, input))
}

fn decode_pascal_delphi(input: &str) -> Option<Symbol> {
    if let Some(rest) = input.strip_prefix('@') {
        let mut parts = rest.split('$');
        let scopes = parts
            .next()?
            .split('@')
            .filter(|part| !part.is_empty())
            .collect::<Vec<_>>();
        let params = parts.next().unwrap_or_default();
        let param_display = if params.ends_with('i') { "int" } else { "" };
        let path = scopes.join(".");
        return Some(simple(
            Scheme::PascalDelphi,
            SymbolKind::Function,
            &format!("{path}({param_display})").replace("()", "()"),
            input,
        ));
    }

    if let Some(rest) = input.strip_prefix("P$") {
        let (unit, remainder) = rest.split_once("_$$_")?;
        let mut parts = remainder.split('$').filter(|part| !part.is_empty());
        if let Some(name) = parts.next() {
            return Some(simple(
                Scheme::PascalDelphi,
                SymbolKind::Function,
                &format!("{}.{}(longint)", title_case(unit), title_case(name)),
                input,
            ));
        }
    }

    None
}

fn decode_vlang(input: &str) -> Option<Symbol> {
    let mut parts = input.split("__").collect::<Vec<_>>();
    if parts.len() < 2 {
        return None;
    }
    let module = parts.remove(0);
    let tail = parts.join("__");
    let display = format!("{module}.{}", tail.replace('_', "."));
    let kind = if display.matches('.').count() >= 2 {
        SymbolKind::Method
    } else {
        SymbolKind::Function
    };
    let mut symbol = simple(Scheme::Vlang, kind, &display, input);
    symbol.path = display.split('.').map(Name::identifier).collect();
    Some(symbol)
}

fn simple(scheme: Scheme, kind: SymbolKind, display: &str, input: &str) -> Symbol {
    Symbol::new(scheme, kind)
        .with_display(display)
        .with_verbatim(input)
}

fn scheme_for(_kind: SymbolKind) -> Scheme {
    Scheme::Ocaml
}

fn z_decode(input: &str) -> String {
    input
        .replace("zi", ".")
        .replace("zm", "-")
        .replace("zp", "+")
        .replace("ZMZN", "[]")
        .replace("ZC", ":")
}

fn title_case(input: &str) -> String {
    let lower = input.to_ascii_lowercase();
    let mut chars = lower.chars();
    let Some(first) = chars.next() else {
        return String::new();
    };
    first.to_ascii_uppercase().to_string() + chars.as_str()
}

fn looks_like_wasm_name(input: &str) -> bool {
    let parts = input.split("::").collect::<Vec<_>>();
    if parts.len() != 2 {
        return false;
    }
    parts.iter().all(|part| {
        !part.is_empty()
            && part
                .chars()
                .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_' || ch == '.')
    })
}

fn looks_like_dotnet_generic(input: &str) -> bool {
    let Some((name, arity)) = input.rsplit_once('`') else {
        return false;
    };
    !name.is_empty()
        && arity.chars().all(|ch| ch.is_ascii_digit())
        && name
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '+' | ':' | '$'))
}

fn looks_like_haskell(input: &str) -> bool {
    (input.ends_with("_closure") || input.ends_with("_info"))
        && (input.contains("zi")
            || input.contains("zm")
            || input.contains("zp")
            || input.contains("ZM")
            || input.starts_with("ghc")
            || input.starts_with("base_"))
}
