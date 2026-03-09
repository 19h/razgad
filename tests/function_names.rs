use razgad::{
    decode, normalize_symbol_display, parse_function_name, parse_function_name_with_separator,
    parse_template_node, parse_template_node_with_separator, template_depth, AccessModifier,
    CallingConvention, Name, Scheme, TemplateNodeKind, Type,
};

#[test]
fn parse_function_name_captures_broad_signature_parts() {
    let parsed = parse_function_name(
        "public: std::vector<int> __cdecl demo::Widget::run(std::map<int, std::string> const& values, char *name) const",
    )
    .unwrap();

    assert_eq!(parsed.access, Some(AccessModifier::Public));
    assert_eq!(parsed.return_type.as_deref(), Some("std::vector<int>"));
    assert_eq!(parsed.calling_convention.as_deref(), Some("__cdecl"));
    assert_eq!(parsed.callable_name.as_deref(), Some("demo::Widget::run"));
    assert_eq!(
        parsed.callable_path,
        vec!["demo", "Widget", "run"]
            .into_iter()
            .map(str::to_string)
            .collect::<Vec<_>>()
    );
    assert_eq!(parsed.leaf_name.as_deref(), Some("run"));
    assert_eq!(parsed.arguments.len(), 2);
    assert_eq!(
        parsed.arguments[0].type_text,
        "std::map<int, std::string> const&"
    );
    assert_eq!(parsed.arguments[0].name.as_deref(), Some("values"));
    assert_eq!(parsed.arguments[1].type_text, "char *");
    assert_eq!(parsed.arguments[1].name.as_deref(), Some("name"));
    assert_eq!(parsed.trailing_qualifiers.as_deref(), Some("const"));
    assert_eq!(template_depth(parsed.return_type.as_deref().unwrap()), 1);
}

#[test]
fn parse_function_name_handles_return_location_and_usercall() {
    let parsed =
        parse_function_name("__int64 __usercall Foo::bar@<rax>(int a, char const *msg)").unwrap();

    assert_eq!(parsed.return_type.as_deref(), Some("__int64"));
    assert_eq!(parsed.calling_convention.as_deref(), Some("__usercall"));
    assert_eq!(parsed.return_location.as_deref(), Some("@<rax>"));
    assert_eq!(parsed.callable_name.as_deref(), Some("Foo::bar"));
    assert_eq!(parsed.arguments.len(), 2);
    assert_eq!(parsed.arguments[0].type_text, "int");
    assert_eq!(parsed.arguments[0].name.as_deref(), Some("a"));
    assert_eq!(parsed.arguments[1].type_text, "char const *");
    assert_eq!(parsed.arguments[1].name.as_deref(), Some("msg"));
}

#[test]
fn parse_function_name_handles_function_pointer_return_style() {
    let parsed = parse_function_name("void (__cdecl *demo::signal(int))(char const *)").unwrap();

    assert!(parsed.has_signature());
    assert_eq!(parsed.callable_name.as_deref(), Some("demo::signal"));
    assert_eq!(parsed.callable_path, vec!["demo", "signal"]);
    assert_eq!(parsed.arguments.len(), 1);
    assert_eq!(parsed.arguments[0].type_text, "int");
    assert_eq!(
        parsed.return_type.as_deref(),
        Some("void (__cdecl *)(char const *)")
    );
    assert!(parsed.calling_convention.is_none());
}

#[test]
fn parse_function_name_handles_pointer_to_member_return_style() {
    let parsed = parse_function_name("int (demo::Widget::*demo::Factory::slot())").unwrap();

    assert!(parsed.has_signature());
    assert_eq!(parsed.callable_name.as_deref(), Some("demo::Factory::slot"));
    assert_eq!(parsed.callable_path, vec!["demo", "Factory", "slot"]);
    assert!(parsed.arguments.is_empty());
    assert_eq!(parsed.return_type.as_deref(), Some("int (demo::Widget::*)"));
}

#[test]
fn parse_function_name_handles_member_function_pointer_return_style() {
    let parsed =
        parse_function_name("void (demo::Widget::*demo::Factory::signal(int))(char const *)")
            .unwrap();

    assert!(parsed.has_signature());
    assert_eq!(
        parsed.callable_name.as_deref(),
        Some("demo::Factory::signal")
    );
    assert_eq!(parsed.arguments.len(), 1);
    assert_eq!(parsed.arguments[0].type_text, "int");
    assert_eq!(
        parsed.return_type.as_deref(),
        Some("void (demo::Widget::*)(char const *)")
    );
}

#[test]
fn normalize_symbol_display_decodes_rust_escape_sequences() {
    let normalized = normalize_symbol_display(
        "core::ptr::drop_in_place$LT$sqlparser..ast..Expr$GT$::h1234567890abcdef",
    );
    assert_eq!(normalized, "core::ptr::drop_in_place<sqlparser::ast::Expr>");
}

#[test]
fn parse_template_node_handles_nested_templates() {
    let node = parse_template_node("std::map<std::string, std::vector<int>>").unwrap();

    assert_eq!(node.kind, TemplateNodeKind::Template);
    assert_eq!(node.label, "std::map");
    assert_eq!(node.path, vec!["std", "map"]);
    assert_eq!(node.args.len(), 2);
    assert_eq!(node.args[0].label, "std::string");
    assert_eq!(node.args[1].label, "std::vector");
    assert_eq!(node.args[1].args[0].label, "int");
}

#[test]
fn plain_scheme_decode_uses_broad_function_name_parser() {
    let symbol = decode(
        Scheme::Plain,
        "private: int __fastcall demo::Widget::run(std::string const& name)",
    )
    .unwrap();

    assert_eq!(symbol.kind, razgad::SymbolKind::Method);
    assert_eq!(
        symbol.path,
        vec![
            Name::identifier("demo"),
            Name::identifier("Widget"),
            Name::identifier("run"),
        ]
    );
    let signature = symbol.signature.unwrap();
    assert_eq!(
        signature.calling_convention,
        Some(CallingConvention::Fastcall)
    );
    assert_eq!(signature.return_type, Some(Type::int()));
    assert_eq!(
        signature.parameters,
        vec![Type::const_ref(Type::named(["std", "string"]))]
    );
}

#[test]
fn plain_decode_projects_function_pointer_return_style() {
    let symbol = decode(
        Scheme::Plain,
        "void (__cdecl *demo::signal(int))(char const *)",
    )
    .unwrap();

    assert_eq!(symbol.kind, razgad::SymbolKind::Method);
    assert_eq!(
        symbol.path,
        vec![Name::identifier("demo"), Name::identifier("signal")]
    );
    let signature = symbol.signature.unwrap();
    assert_eq!(signature.parameters, vec![Type::int()]);
    assert_eq!(
        signature.return_type,
        Some(Type::Other("void (__cdecl *)(char const *)".to_string()))
    );
}

#[test]
fn plain_decode_projects_pointer_to_member_return_style() {
    let symbol = decode(Scheme::Plain, "int (demo::Widget::*demo::Factory::slot())").unwrap();

    assert_eq!(symbol.kind, razgad::SymbolKind::Method);
    assert_eq!(
        symbol.path,
        vec![
            Name::identifier("demo"),
            Name::identifier("Factory"),
            Name::identifier("slot"),
        ]
    );
    let signature = symbol.signature.unwrap();
    assert!(signature.parameters.is_empty());
    assert_eq!(
        signature.return_type,
        Some(Type::Other("int (demo::Widget::*)".to_string()))
    );
}

#[test]
fn parse_function_name_handles_callable_only_input() {
    let parsed = parse_function_name("AnimEventLoader::LoadAnimationEventDatabase").unwrap();
    assert_eq!(
        parsed.callable_name.as_deref(),
        Some("AnimEventLoader::LoadAnimationEventDatabase")
    );
    assert!(parsed.return_type.is_none());
    assert!(parsed.arguments.is_empty());
}

#[test]
fn parse_function_name_supports_non_cpp_scope_separators() {
    let parsed = parse_function_name_with_separator(
        "Swift.Int Demo.Widget.run(Swift.String name, Swift.Bool)",
        ".",
    )
    .unwrap();

    assert_eq!(parsed.return_type.as_deref(), Some("Swift.Int"));
    assert_eq!(parsed.callable_name.as_deref(), Some("Demo.Widget.run"));
    assert_eq!(parsed.callable_path, vec!["Demo", "Widget", "run"]);
    assert_eq!(parsed.arguments.len(), 2);
    assert_eq!(parsed.arguments[0].type_text, "Swift.String");
    assert_eq!(parsed.arguments[0].name.as_deref(), Some("name"));
    assert_eq!(parsed.arguments[1].type_text, "Swift.Bool");
}

#[test]
fn parse_template_node_supports_non_cpp_scope_separators() {
    let node = parse_template_node_with_separator("Swift.Array<Demo.Widget>", ".").unwrap();

    assert_eq!(node.label, "Swift.Array");
    assert_eq!(node.path, vec!["Swift", "Array"]);
    assert_eq!(node.args[0].label, "Demo.Widget");
    assert_eq!(node.args[0].path, vec!["Demo", "Widget"]);
}

#[test]
fn parse_function_name_does_not_treat_go_receivers_as_signatures() {
    let parsed = parse_function_name_with_separator("main.(*T).Method", ".").unwrap();

    assert_eq!(parsed.callable_name.as_deref(), Some("main.(*T).Method"));
    assert_eq!(parsed.callable_path, vec!["main", "(*T)", "Method"]);
    assert!(!parsed.has_signature());
    assert!(parsed.arguments.is_empty());
    assert!(parsed.return_type.is_none());
}
