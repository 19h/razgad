use std::mem;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AccessModifier {
    Public,
    Protected,
    Private,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TemplateNodeKind {
    Leaf,
    Template,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TemplateNode {
    pub label: String,
    pub path: Vec<String>,
    pub kind: TemplateNodeKind,
    pub args: Vec<TemplateNode>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParsedArgument {
    pub full: String,
    pub type_text: String,
    pub name: Option<String>,
    pub template: Option<TemplateNode>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParsedFunctionName {
    pub full: String,
    pub normalized: String,
    pub signature_present: bool,
    pub access: Option<AccessModifier>,
    pub return_type: Option<String>,
    pub return_template: Option<TemplateNode>,
    pub calling_convention: Option<String>,
    pub return_location: Option<String>,
    pub callable_name: Option<String>,
    pub callable_path: Vec<String>,
    pub callable_template: Option<TemplateNode>,
    pub leaf_name: Option<String>,
    pub arguments: Vec<ParsedArgument>,
    pub trailing_qualifiers: Option<String>,
}

impl ParsedFunctionName {
    pub fn has_signature(&self) -> bool {
        self.signature_present
    }
}

pub fn parse_function_name(input: &str) -> Option<ParsedFunctionName> {
    parse_function_name_with_separator(input, "::")
}

pub fn parse_function_name_with_separator(
    input: &str,
    path_separator: &str,
) -> Option<ParsedFunctionName> {
    let normalized = normalize_symbol_display_for_separator(input, path_separator);
    let decl = normalized.trim();
    if decl.is_empty() {
        return None;
    }

    let mut access = None;
    let mut signature_present = false;
    let mut return_type = None;
    let mut return_template = None;
    let mut calling_convention = None;
    let mut return_location = None;
    let mut callable_name = None;
    let mut arguments = Vec::new();
    let mut trailing_qualifiers = None;

    if let Some((open, close)) = find_outer_signature_parens(decl) {
        let suffix = decl[close + 1..].trim();
        if suffix.is_empty() || looks_like_signature_suffix(suffix) {
            let raw_head = decl[..open].trim();
            let args_block = decl[open + 1..close].trim();
            if !suffix.is_empty() {
                trailing_qualifiers = Some(suffix.to_string());
            }

            let (parsed_access, head_without_access) = strip_access_modifier(raw_head);
            access = parsed_access;

            if let Some(parts) = parse_function_pointer_style_signature(
                head_without_access,
                args_block,
                path_separator,
            ) {
                signature_present = true;
                return_type = Some(parts.return_type.clone());
                return_template = parse_type_template(&parts.return_type, path_separator);
                if let Some(qualifiers) = parts.trailing_qualifiers {
                    trailing_qualifiers = Some(qualifiers);
                }
                let (callable_without_retloc, parsed_retloc) =
                    strip_return_location(&parts.callable);
                return_location = parsed_retloc;
                callable_name = Some(callable_without_retloc);
                arguments = parts.arguments;
            } else if let Some(parts) =
                parse_embedded_declarator_signature(head_without_access, args_block, path_separator)
            {
                signature_present = true;
                return_type = Some(parts.return_type.clone());
                return_template = parse_type_template(&parts.return_type, path_separator);
                if let Some(qualifiers) = parts.trailing_qualifiers {
                    trailing_qualifiers = Some(qualifiers);
                }
                let (callable_without_retloc, parsed_retloc) =
                    strip_return_location(&parts.callable);
                return_location = parsed_retloc;
                callable_name = Some(callable_without_retloc);
                arguments = parts.arguments;
            } else if !head_without_access.contains("(*") {
                signature_present = true;

                let (head_without_cc, parsed_cc) = strip_calling_convention(head_without_access);
                calling_convention = parsed_cc;

                let parts = split_signature_head(&head_without_cc, path_separator);
                if !parts.return_type.is_empty() {
                    return_type = Some(parts.return_type.clone());
                    return_template = parse_type_template(&parts.return_type, path_separator);
                }
                if !parts.callable.is_empty() {
                    let (callable_without_retloc, parsed_retloc) =
                        strip_return_location(&parts.callable);
                    return_location = parsed_retloc;
                    callable_name = Some(callable_without_retloc);
                }

                arguments = parse_arguments(args_block, path_separator);
            }
        }
    }

    if callable_name.is_none() {
        callable_name = Some(decl.to_string());
    }

    let callable_name = callable_name.and_then(|name| {
        let trimmed = name.trim().to_string();
        (!trimmed.is_empty()).then_some(trimmed)
    });
    let callable_path = callable_name
        .as_deref()
        .map(|name| split_scope_with_separator(name, path_separator))
        .unwrap_or_default();
    let leaf_name = callable_path.last().cloned();
    let callable_template = callable_name
        .as_deref()
        .and_then(|name| parse_template_node_with_separator(name, path_separator));

    Some(ParsedFunctionName {
        full: decl.to_string(),
        normalized: decl.to_string(),
        signature_present,
        access,
        return_type,
        return_template,
        calling_convention,
        return_location,
        callable_name,
        callable_path,
        callable_template,
        leaf_name,
        arguments,
        trailing_qualifiers,
    })
}

pub fn normalize_symbol_display(input: &str) -> String {
    normalize_symbol_display_for_separator(input, "::")
}

fn normalize_symbol_display_for_separator(input: &str, path_separator: &str) -> String {
    let mut value = input.trim().to_string();
    if value.is_empty() {
        return value;
    }

    for (from, to) in [
        ("$LT$", "<"),
        ("$GT$", ">"),
        ("$LP$", "("),
        ("$RP$", ")"),
        ("$C$", ","),
        ("$u20$", " "),
        ("$u5b$", "["),
        ("$u5d$", "]"),
        ("$u7b$", "{"),
        ("$u7d$", "}"),
        ("$u27$", "'"),
        ("$u3d$", "="),
        ("$u3a$", ":"),
        ("$u2b$", "+"),
        ("$u21$", "!"),
        ("$u26$", "&"),
        ("$u2f$", "/"),
        ("$u5c$", "\\"),
    ] {
        value = value.replace(from, to);
    }

    if path_separator == "::" {
        value = value.replace("..", "::");
        if let Some(stripped) = strip_rust_hash_suffix(&value) {
            value = stripped.to_string();
        }
    }

    value.trim().to_string()
}

pub fn split_scope(label: &str) -> Vec<String> {
    split_scope_with_separator(label, "::")
}

pub fn split_scope_with_separator(label: &str, separator: &str) -> Vec<String> {
    split_top_level(label, separator)
}

pub fn parse_template_node(text: &str) -> Option<TemplateNode> {
    parse_template_node_with_separator(text, "::")
}

pub fn parse_template_node_with_separator(
    text: &str,
    path_separator: &str,
) -> Option<TemplateNode> {
    let node = parse_template_tree(text, path_separator)?;
    matches!(node.kind, TemplateNodeKind::Template).then_some(node)
}

fn parse_template_tree(text: &str, path_separator: &str) -> Option<TemplateNode> {
    let value = text.trim();
    if value.is_empty() {
        return None;
    }
    let Some(open) = find_top_level_char(value, '<') else {
        return Some(TemplateNode {
            label: value.to_string(),
            path: split_scope_with_separator(value, path_separator),
            kind: TemplateNodeKind::Leaf,
            args: Vec::new(),
        });
    };
    let close = find_matching_delimiter(value, open, '<', '>')?;
    let label = value[..open].trim();
    let inner = value[open + 1..close].trim();
    Some(TemplateNode {
        label: label.to_string(),
        path: split_scope_with_separator(label, path_separator),
        kind: TemplateNodeKind::Template,
        args: split_top_level(inner, ",")
            .into_iter()
            .filter_map(|arg| parse_template_tree(&arg, path_separator))
            .collect(),
    })
}

pub fn split_argument_name(text: &str) -> ParsedArgument {
    split_argument_name_with_separator(text, "::")
}

pub fn split_argument_name_with_separator(text: &str, path_separator: &str) -> ParsedArgument {
    let value = text.trim();
    if value.is_empty() || value == "..." {
        return ParsedArgument {
            full: value.to_string(),
            type_text: value.to_string(),
            name: None,
            template: parse_type_template(value, path_separator),
        };
    }

    let mut angle = 0usize;
    let mut paren = 0usize;
    let mut bracket = 0usize;
    for (index, ch) in value.char_indices().rev() {
        match ch {
            '>' => angle += 1,
            '<' => angle = angle.saturating_sub(1),
            ')' => paren += 1,
            '(' => paren = paren.saturating_sub(1),
            ']' => bracket += 1,
            '[' => bracket = bracket.saturating_sub(1),
            c if c.is_whitespace() && angle == 0 && paren == 0 && bracket == 0 => {
                let suffix = value[index..].trim();
                if is_argument_suffix(suffix) {
                    let sigil = suffix
                        .chars()
                        .take_while(|ch| *ch == '*' || *ch == '&')
                        .count();
                    let type_prefix = value[..index].trim();
                    let sigil_text = &suffix[..sigil];
                    let arg_name = suffix[sigil..].trim();
                    let type_text = if sigil_text.is_empty() {
                        type_prefix.to_string()
                    } else {
                        format!("{type_prefix} {sigil_text}")
                    };
                    return ParsedArgument {
                        full: value.to_string(),
                        type_text: type_text.trim().to_string(),
                        name: Some(arg_name.to_string()),
                        template: parse_type_template(&type_text, path_separator),
                    };
                }
                break;
            }
            _ => {}
        }
    }

    ParsedArgument {
        full: value.to_string(),
        type_text: value.to_string(),
        name: None,
        template: parse_type_template(value, path_separator),
    }
}

pub fn template_depth(text: &str) -> usize {
    let mut depth = 0usize;
    let mut max_depth = 0usize;
    for ch in text.chars() {
        match ch {
            '<' => {
                depth += 1;
                max_depth = max_depth.max(depth);
            }
            '>' => depth = depth.saturating_sub(1),
            _ => {}
        }
    }
    max_depth
}

pub(crate) fn parse_calling_convention_token(value: &str) -> Option<crate::CallingConvention> {
    match value {
        "__cdecl" => Some(crate::CallingConvention::Cdecl),
        "__stdcall" => Some(crate::CallingConvention::Stdcall),
        "__fastcall" => Some(crate::CallingConvention::Fastcall),
        "__vectorcall" => Some(crate::CallingConvention::Vectorcall),
        "__thiscall" => Some(crate::CallingConvention::Thiscall),
        "__swiftcall" => Some(crate::CallingConvention::Swiftcall),
        "__golang" => Some(crate::CallingConvention::Golang),
        "__usercall" => Some(crate::CallingConvention::Usercall),
        "__userpurge" => Some(crate::CallingConvention::Userpurge),
        "__pascal" => Some(crate::CallingConvention::Pascal),
        _ => None,
    }
}

struct SignatureHeadParts {
    return_type: String,
    callable: String,
}

struct FunctionPointerStyleParts {
    return_type: String,
    callable: String,
    arguments: Vec<ParsedArgument>,
    trailing_qualifiers: Option<String>,
}

fn parse_arguments(args_block: &str, path_separator: &str) -> Vec<ParsedArgument> {
    let trimmed = args_block.trim();
    if trimmed.is_empty() || trimmed == "void" {
        return Vec::new();
    }
    split_top_level(trimmed, ",")
        .into_iter()
        .map(|arg| split_argument_name_with_separator(&arg, path_separator))
        .collect()
}

fn looks_like_signature_suffix(text: &str) -> bool {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return true;
    }

    trimmed.starts_with("->")
        || trimmed.starts_with('&')
        || trimmed.starts_with("const")
        || trimmed.starts_with("volatile")
        || trimmed.starts_with("noexcept")
        || trimmed.starts_with("override")
        || trimmed.starts_with("final")
        || trimmed.starts_with("requires")
        || trimmed.starts_with("throws")
        || trimmed.starts_with("rethrows")
        || trimmed.starts_with("where")
}

fn parse_function_pointer_style_signature(
    head: &str,
    tail_args: &str,
    path_separator: &str,
) -> Option<FunctionPointerStyleParts> {
    let head = head.trim();
    let (outer_open, outer_close) = find_outer_signature_parens(head)?;
    if outer_close + 1 != head.len() {
        return None;
    }

    let prefix = head[..outer_open].trim();
    let inner = head[outer_open + 1..outer_close].trim();
    let (call_open, call_close) = find_outer_signature_parens(inner)?;
    let inner_suffix = inner[call_close + 1..].trim();
    if !inner_suffix.is_empty() && !looks_like_signature_suffix(inner_suffix) {
        return None;
    }

    let declarator = inner[..call_open].trim();
    let callable_args = inner[call_open + 1..call_close].trim();
    let marker = find_last_declarator_marker(declarator)?;
    let return_head = declarator[..=marker].trim();
    let callable = declarator[marker + 1..].trim();
    if callable.is_empty() || !is_callable_fragment(callable, path_separator) {
        return None;
    }

    let mut return_type = String::new();
    if !prefix.is_empty() {
        return_type.push_str(prefix);
        return_type.push(' ');
    }
    return_type.push('(');
    return_type.push_str(return_head);
    return_type.push(')');
    return_type.push('(');
    return_type.push_str(tail_args.trim());
    return_type.push(')');

    Some(FunctionPointerStyleParts {
        return_type: collapse_whitespace(&return_type),
        callable: callable.to_string(),
        arguments: parse_arguments(callable_args, path_separator),
        trailing_qualifiers: (!inner_suffix.is_empty()).then_some(inner_suffix.to_string()),
    })
}

fn parse_embedded_declarator_signature(
    return_prefix: &str,
    declarator_with_args: &str,
    path_separator: &str,
) -> Option<FunctionPointerStyleParts> {
    let return_prefix = return_prefix.trim();
    let declarator_with_args = declarator_with_args.trim();
    let (call_open, call_close) = find_outer_signature_parens(declarator_with_args)?;
    if call_close + 1 != declarator_with_args.len() {
        return None;
    }

    let declarator = declarator_with_args[..call_open].trim();
    let callable_args = declarator_with_args[call_open + 1..call_close].trim();
    let marker = find_last_declarator_marker(declarator)?;
    let return_head = declarator[..=marker].trim();
    let callable = declarator[marker + 1..].trim();
    if callable.is_empty() || !is_callable_fragment(callable, path_separator) {
        return None;
    }

    let mut return_type = String::new();
    if !return_prefix.is_empty() {
        return_type.push_str(return_prefix);
        return_type.push(' ');
    }
    return_type.push('(');
    return_type.push_str(return_head);
    return_type.push(')');

    Some(FunctionPointerStyleParts {
        return_type: collapse_whitespace(&return_type),
        callable: callable.to_string(),
        arguments: parse_arguments(callable_args, path_separator),
        trailing_qualifiers: None,
    })
}

fn strip_access_modifier(input: &str) -> (Option<AccessModifier>, &str) {
    for (prefix, access) in [
        ("public:", AccessModifier::Public),
        ("protected:", AccessModifier::Protected),
        ("private:", AccessModifier::Private),
    ] {
        if let Some(rest) = input.strip_prefix(prefix) {
            return (Some(access), rest.trim());
        }
        let uppercase = prefix.to_ascii_uppercase();
        if input
            .to_ascii_lowercase()
            .starts_with(&prefix[..prefix.len() - 1])
            && input[prefix.len() - 1..].starts_with(':')
        {
            let _ = uppercase;
        }
    }

    if input.len() >= 7 {
        let lower = input.to_ascii_lowercase();
        for (prefix, access) in [
            ("public:", AccessModifier::Public),
            ("protected:", AccessModifier::Protected),
            ("private:", AccessModifier::Private),
        ] {
            if lower.starts_with(prefix) {
                return (Some(access), input[prefix.len()..].trim());
            }
        }
    }

    (None, input.trim())
}

fn strip_calling_convention(input: &str) -> (String, Option<String>) {
    for token in [
        "__thiscall",
        "__stdcall",
        "__fastcall",
        "__swiftcall",
        "__userpurge",
        "__usercall",
        "__vectorcall",
        "__pascal",
        "__golang",
        "__cdecl",
    ] {
        if let Some(index) = input.find(token) {
            let mut out = input.to_string();
            out.replace_range(index..index + token.len(), " ");
            return (collapse_whitespace(&out), Some(token.to_string()));
        }
    }

    if let Some(index) = input.find("__cc(") {
        let end = find_matching_delimiter(input, index + 4, '(', ')');
        if let Some(end) = end {
            let token = input[index..=end].to_string();
            let mut out = input.to_string();
            out.replace_range(index..=end, " ");
            return (collapse_whitespace(&out), Some(token));
        }
    }

    (input.trim().to_string(), None)
}

fn strip_return_location(input: &str) -> (String, Option<String>) {
    let mut output = String::with_capacity(input.len());
    let mut found = None;
    let mut i = 0usize;
    while i < input.len() {
        let rest = &input[i..];
        if found.is_none() && rest.starts_with("@<") {
            if let Some(end_rel) = rest.find('>') {
                found = Some(rest[..=end_rel].to_string());
                i += end_rel + 1;
                continue;
            }
        }
        let ch = rest.chars().next().unwrap();
        output.push(ch);
        i += ch.len_utf8();
    }
    (collapse_whitespace(&output), found)
}

fn split_signature_head(head: &str, path_separator: &str) -> SignatureHeadParts {
    let text = head.trim();
    if text.is_empty() {
        return SignatureHeadParts {
            return_type: String::new(),
            callable: String::new(),
        };
    }

    let tokens = split_top_level_whitespace(text);
    for index in (1..tokens.len()).rev() {
        let candidate = tokens[index..].join(" ");
        if !is_callable_fragment(&candidate, path_separator) {
            continue;
        }
        return SignatureHeadParts {
            return_type: tokens[..index].join(" ").trim().to_string(),
            callable: candidate.trim().to_string(),
        };
    }

    if find_last_top_level_scope(text, path_separator) >= 0 {
        return SignatureHeadParts {
            return_type: String::new(),
            callable: text.to_string(),
        };
    }

    SignatureHeadParts {
        return_type: text.to_string(),
        callable: String::new(),
    }
}

fn find_last_declarator_marker(text: &str) -> Option<usize> {
    let mut depth = DepthState::default();
    let mut last = None;
    for (index, ch) in text.char_indices() {
        if matches!(ch, '*' | '&') && depth.is_top_level() {
            last = Some(index);
        }
        depth.observe(ch);
    }
    last
}

fn parse_type_template(text: &str, path_separator: &str) -> Option<TemplateNode> {
    let trimmed = strip_type_noise(text);
    let open = trimmed.find('<')?;
    let close = find_matching_delimiter(&trimmed, open, '<', '>')?;
    let template_text = &trimmed[..=close];
    parse_template_tree(template_text, path_separator)
}

fn strip_type_noise(text: &str) -> String {
    let mut out = collapse_whitespace(text.trim());
    for prefix in ["class ", "struct ", "enum ", "union "] {
        while out.starts_with(prefix) {
            out = out[prefix.len()..].trim().to_string();
        }
    }
    out.replace("std::__1::", "std::")
}

fn strip_rust_hash_suffix(input: &str) -> Option<&str> {
    let (head, tail) = input.rsplit_once("::h")?;
    (!tail.is_empty() && tail.len() >= 8 && tail.chars().all(|ch| ch.is_ascii_hexdigit()))
        .then_some(head)
}

fn split_top_level_whitespace(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut current = String::new();
    let mut depth = DepthState::default();
    for ch in text.chars() {
        if ch.is_whitespace() && depth.is_top_level() {
            if !current.is_empty() {
                out.push(mem::take(&mut current));
            }
            continue;
        }
        current.push(ch);
        depth.observe(ch);
    }
    if !current.is_empty() {
        out.push(current);
    }
    out
}

fn split_top_level(text: &str, separator: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut start = 0usize;
    let mut depth = DepthState::default();
    let mut index = 0usize;
    while index < text.len() {
        let rest = &text[index..];
        if depth.is_top_level() && rest.starts_with(separator) {
            let part = text[start..index].trim();
            if !part.is_empty() {
                out.push(part.to_string());
            }
            index += separator.len();
            start = index;
            continue;
        }
        let ch = rest.chars().next().unwrap();
        depth.observe(ch);
        index += ch.len_utf8();
    }

    let tail = text[start..].trim();
    if !tail.is_empty() {
        out.push(tail.to_string());
    }
    out
}

fn is_callable_fragment(text: &str, path_separator: &str) -> bool {
    let value = text.trim();
    if value.is_empty() {
        return false;
    }
    if [
        "const", "volatile", "signed", "unsigned", "short", "long", "class", "struct", "enum",
        "union",
    ]
    .contains(&value)
    {
        return false;
    }
    if value.starts_with("operator") {
        return true;
    }
    if is_simple_identifier(value) {
        return true;
    }
    find_last_top_level_scope(value, path_separator) >= 0
}

fn is_simple_identifier(text: &str) -> bool {
    let mut chars = text.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first == '~' || first == '_' || first.is_ascii_alphabetic()) {
        return false;
    }
    chars.all(|ch| ch == '_' || ch.is_ascii_alphanumeric())
}

fn is_argument_suffix(text: &str) -> bool {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return false;
    }
    let ident = trimmed.trim_start_matches(['*', '&']);
    !ident.is_empty() && is_simple_identifier(ident)
}

fn find_last_top_level_scope(text: &str, separator: &str) -> isize {
    let mut depth = DepthState::default();
    let mut index = 0usize;
    let mut last = -1isize;
    while index < text.len() {
        let rest = &text[index..];
        let ch = rest.chars().next().unwrap();
        let ch_len = ch.len_utf8();
        if rest.starts_with(separator) && depth.is_top_level() {
            last = index as isize;
            index += separator.len();
            continue;
        }
        depth.observe(ch);
        index += ch_len;
    }
    last
}

fn find_outer_signature_parens(text: &str) -> Option<(usize, usize)> {
    let close = text.rfind(')')?;
    let mut depth = 0usize;
    for (index, ch) in text[..=close].char_indices().rev() {
        match ch {
            ')' => depth += 1,
            '(' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some((index, close));
                }
            }
            _ => {}
        }
    }
    None
}

fn find_top_level_char(text: &str, target: char) -> Option<usize> {
    let mut depth = DepthState::default();
    for (index, ch) in text.char_indices() {
        if ch == target && depth.is_top_level() {
            return Some(index);
        }
        depth.observe(ch);
    }
    None
}

fn find_matching_delimiter(
    text: &str,
    open_index: usize,
    open: char,
    close: char,
) -> Option<usize> {
    let mut depth = 0usize;
    for (offset, ch) in text[open_index..].char_indices() {
        if ch == open {
            depth += 1;
        } else if ch == close {
            depth = depth.saturating_sub(1);
            if depth == 0 {
                return Some(open_index + offset);
            }
        }
    }
    None
}

fn collapse_whitespace(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[derive(Default)]
struct DepthState {
    angle: usize,
    paren: usize,
    bracket: usize,
    brace: usize,
}

impl DepthState {
    fn observe(&mut self, ch: char) {
        match ch {
            '<' => self.angle += 1,
            '>' => self.angle = self.angle.saturating_sub(1),
            '(' => self.paren += 1,
            ')' => self.paren = self.paren.saturating_sub(1),
            '[' => self.bracket += 1,
            ']' => self.bracket = self.bracket.saturating_sub(1),
            '{' => self.brace += 1,
            '}' => self.brace = self.brace.saturating_sub(1),
            _ => {}
        }
    }

    fn is_top_level(&self) -> bool {
        self.angle == 0 && self.paren == 0 && self.bracket == 0 && self.brace == 0
    }
}
