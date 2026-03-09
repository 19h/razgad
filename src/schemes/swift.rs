use crate::{text, Confidence, Scheme, Symbol, SymbolKind};

pub fn decode(scheme: Scheme, input: &str) -> Option<Symbol> {
    let demangled = swift_demangle::demangle(input)
        .or_else(|_| {
            input
                .strip_prefix('_')
                .ok_or(())
                .and_then(|inner| swift_demangle::demangle(inner).map_err(|_| ()))
        })
        .ok()?;
    let display = strip_swift_return(demangled);

    if let Some(path) = display.strip_suffix(".init()") {
        let mut symbol = Symbol::new(scheme, SymbolKind::Constructor);
        symbol.path = text::parse_names(path, ".");
        symbol.concrete_family = Scheme::Swift;
        return Some(symbol.with_display(display).with_verbatim(input));
    }

    let mut symbol =
        text::symbol_from_demangled_cpp(Symbol::new(scheme, SymbolKind::Function), display, ".");
    symbol.concrete_family = Scheme::Swift;
    Some(symbol.with_display(display).with_verbatim(input))
}

pub fn detect(input: &str) -> Option<(Scheme, Confidence)> {
    (input.starts_with("_$s")
        || input.starts_with("$s")
        || input.starts_with("$S")
        || input.starts_with("_T"))
    .then_some((Scheme::Swift, Confidence::Certain))
}

fn strip_swift_return(display: &str) -> &str {
    display.split(" -> ").next().unwrap_or(display)
}
