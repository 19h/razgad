use crate::{text, Confidence, Scheme, Symbol, SymbolKind};

pub fn decode(scheme: Scheme, input: &str) -> Option<Symbol> {
    let demangled = crate::swift_demangle::demangle(input).ok()?;
    let display = strip_swift_return(&demangled);
    let projection = strip_argument_labels(display);

    if let Some(path) = constructor_path(display) {
        let mut symbol = Symbol::new(scheme, SymbolKind::Constructor);
        symbol.path = text::parse_names(path, ".");
        symbol.concrete_family = Scheme::Swift;
        return Some(symbol.with_display(display).with_verbatim(input));
    }

    let mut symbol = text::symbol_from_demangled_cpp(
        Symbol::new(scheme, SymbolKind::Function),
        &projection,
        ".",
    );
    symbol.concrete_family = Scheme::Swift;
    Some(symbol.with_display(display).with_verbatim(input))
}

pub fn detect(input: &str) -> Option<(Scheme, Confidence)> {
    (input.starts_with("_$s")
        || input.starts_with("$s")
        || input.starts_with("$S")
        || input.starts_with("_$e")
        || input.starts_with("$e")
        || input.starts_with("_T"))
    .then_some((Scheme::Swift, Confidence::Certain))
}

fn strip_swift_return(display: &str) -> &str {
    display.split(" -> ").next().unwrap_or(display)
}

fn constructor_path(display: &str) -> Option<&str> {
    display.find(".init(").map(|index| &display[..index])
}

fn strip_argument_labels(display: &str) -> String {
    let Some(open) = display.find('(') else {
        return display.to_string();
    };
    let Some(close) = display.rfind(')') else {
        return display.to_string();
    };

    let mut params = Vec::new();
    for param in text::split_qualified(&display[open + 1..close], ",") {
        let trimmed = param.trim();
        let mut depth = 0usize;
        let mut split = None;
        for (index, ch) in trimmed.char_indices() {
            match ch {
                '<' | '(' | '[' => depth += 1,
                '>' | ')' | ']' => depth = depth.saturating_sub(1),
                ':' if depth == 0 => {
                    split = Some(index);
                    break;
                }
                _ => {}
            }
        }
        params.push(match split {
            Some(index) => trimmed[index + 1..].trim().to_string(),
            None => trimmed.to_string(),
        });
    }

    format!("{}({})", &display[..open], params.join(", "))
}
