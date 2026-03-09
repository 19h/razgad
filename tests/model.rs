use symdem::{
    CallingConvention, Confidence, Name, PlatformDecorations, Scheme, SpecialKind, Symbol,
    SymbolKind, Type, decode, encode, heuristic_decode,
};

#[test]
fn itanium_template_method_projects_into_universal_model() {
    let symbol = decode(Scheme::ItaniumCpp, "_ZN4demo3VecIiE4pushERKi").unwrap();

    assert_eq!(symbol.kind, SymbolKind::Method);
    assert_eq!(symbol.path.len(), 3);
    assert_eq!(symbol.path[0], Name::identifier("demo"));
    assert_eq!(symbol.path[1], Name::template("Vec", vec![Type::int()]));
    assert_eq!(symbol.path[2], Name::identifier("push"));

    let signature = symbol.signature.as_ref().expect("signature");
    assert_eq!(signature.parameters, vec![Type::const_ref(Type::int())]);
    assert_eq!(signature.return_type, Some(Type::void()));
}

#[test]
fn msvc_metadata_and_tables_fit_the_same_symbol_tree() {
    let vftable = decode(Scheme::MicrosoftCpp, "??_7Widget@demo@@6B@").unwrap();
    let rtti = decode(Scheme::MicrosoftCpp, "??_R0?AUWidget@demo@@@8").unwrap();

    assert_eq!(vftable.kind, SymbolKind::VTable);
    assert_eq!(vftable.special, Some(SpecialKind::Vftable));
    assert_eq!(vftable.display(), "vftable for demo::Widget");

    assert_eq!(rtti.kind, SymbolKind::Metadata);
    assert_eq!(rtti.special, Some(SpecialKind::RttiTypeDescriptor));
    assert_eq!(rtti.display(), "RTTI Type Descriptor for demo::Widget");
}

#[test]
fn wrappers_remain_orthogonal_to_inner_grammars() {
    let macho = decode(Scheme::MachO, "__ZN4demo5alphaEv").unwrap();
    let coff = decode(Scheme::CoffPe, "__imp_?alpha@demo@@YAXH@Z").unwrap();
    let elf = decode(Scheme::Elf, "_ZN4demo5alphaEv@@GLIBCXX_3.4").unwrap();

    assert_eq!(macho.concrete_family, Scheme::ItaniumCpp);
    assert!(macho.platform.leading_underscore);

    assert_eq!(coff.kind, SymbolKind::Import);
    assert!(coff.platform.import_prefix);
    assert_eq!(coff.platform.inner_scheme, Some(Scheme::MicrosoftCpp));

    assert_eq!(elf.platform.elf_version.as_deref(), Some("GLIBCXX_3.4"));
    assert_eq!(elf.display(), "demo::alpha()@GLIBCXX_3.4");
}

#[test]
fn heuristic_decoder_reports_confidence_for_unambiguous_prefixes() {
    let rust = heuristic_decode("__RNvCs9y1O7KqhnLf_4demo5alpha").unwrap();
    let swift = heuristic_decode("_$s4Demo5alphayyF").unwrap();
    let jni = heuristic_decode("Java_p_q_r_A_g").unwrap();

    assert_eq!(rust.scheme, Scheme::RustV0);
    assert_eq!(rust.confidence, Confidence::Certain);

    assert_eq!(swift.scheme, Scheme::Swift);
    assert_eq!(swift.confidence, Confidence::Certain);

    assert_eq!(jni.scheme, Scheme::Jni);
    assert_eq!(jni.confidence, Confidence::Certain);
}

#[test]
fn canonical_symbols_encode_without_verbatim_replay() {
    let itanium = Symbol::function(Scheme::ItaniumCpp, ["foo"], []).with_return(Type::void());
    let msvc = Symbol::function(Scheme::MicrosoftCpp, ["demo", "alpha"], [Type::int()])
        .with_calling_convention(CallingConvention::Cdecl)
        .with_return(Type::void());
    let dlang =
        Symbol::function(Scheme::Dlang, ["demo", "alpha"], [Type::int()]).with_return(Type::void());
    let jni = Symbol::function(
        Scheme::Jni,
        ["p", "q", "r", "A", "f"],
        [Type::int(), Type::named(["java", "lang", "String"])],
    )
    .with_return(Type::int());
    let objc = Symbol::special(Scheme::ObjectiveC, SpecialKind::ObjectiveCClass, ["Point"]);
    let vlang = Symbol::function(Scheme::Vlang, ["main", "main"], []);
    let elf = Symbol::function(Scheme::Elf, ["demo", "alpha"], [])
        .with_return(Type::void())
        .with_platform(PlatformDecorations::default().with_elf_version("GLIBCXX_3.4"));

    assert_eq!(encode(Scheme::ItaniumCpp, &itanium).unwrap(), "_Z3foov");
    assert_eq!(
        encode(Scheme::MicrosoftCpp, &msvc).unwrap(),
        "?alpha@demo@@YAXH@Z"
    );
    assert_eq!(encode(Scheme::Dlang, &dlang).unwrap(), "_D4demo5alphaFiZv");
    assert_eq!(
        encode(Scheme::Jni, &jni).unwrap(),
        "Java_p_q_r_A_f__ILjava_lang_String_2"
    );
    assert_eq!(
        encode(Scheme::ObjectiveC, &objc).unwrap(),
        "_OBJC_CLASS_$_Point"
    );
    assert_eq!(encode(Scheme::Vlang, &vlang).unwrap(), "main__main");
    assert_eq!(
        encode(Scheme::Elf, &elf).unwrap(),
        "_ZN4demo5alphaEv@@GLIBCXX_3.4"
    );
}

#[test]
fn expanded_canonical_encoders_cover_wrappers_and_conventions() {
    let cdecl = Symbol::function(Scheme::Cdecl, ["cdecl_fn"], [Type::int(), Type::int()]);
    let stdcall = Symbol::function(Scheme::Stdcall, ["stdcall_fn"], [Type::int(), Type::int()]);
    let fastcall = Symbol::function(
        Scheme::Fastcall,
        ["fastcall_fn"],
        [Type::int(), Type::int()],
    );
    let vectorcall = Symbol::function(
        Scheme::Vectorcall,
        ["vectorcall_fn"],
        [Type::int(), Type::int()],
    );
    let macho = Symbol::function(Scheme::MachO, ["demo", "alpha"], []).with_return(Type::void());
    let coff = Symbol::function(Scheme::CoffPe, ["stdcall_fn"], [Type::int(), Type::int()])
        .with_platform(PlatformDecorations::default().with_inner_scheme(Scheme::Stdcall));
    let fortran = Symbol::function(Scheme::FortranExternal, ["foo_bar"], []);
    let gfortran = Symbol::function(Scheme::GfortranModule, ["sample", "five"], []);
    let ada = Symbol::function(Scheme::AdaGnat, ["ada", "text_io", "put_line"], []);
    let mut elaboration = Symbol::new(Scheme::AdaGnat, SymbolKind::ModuleInit);
    elaboration.path = vec![Name::identifier("mypkg")];

    assert_eq!(encode(Scheme::Cdecl, &cdecl).unwrap(), "_cdecl_fn");
    assert_eq!(encode(Scheme::Stdcall, &stdcall).unwrap(), "_stdcall_fn@8");
    assert_eq!(
        encode(Scheme::Fastcall, &fastcall).unwrap(),
        "@fastcall_fn@8"
    );
    assert_eq!(
        encode(Scheme::Vectorcall, &vectorcall).unwrap(),
        "vectorcall_fn@@8"
    );
    assert_eq!(encode(Scheme::MachO, &macho).unwrap(), "__ZN4demo5alphaEv");
    assert_eq!(
        encode(Scheme::CoffPe, &coff).unwrap(),
        "__imp__stdcall_fn@8"
    );
    assert_eq!(
        encode(Scheme::FortranExternal, &fortran).unwrap(),
        "foo_bar__"
    );
    assert_eq!(
        encode(Scheme::GfortranModule, &gfortran).unwrap(),
        "__sample_MOD_five"
    );
    assert_eq!(
        encode(Scheme::AdaGnat, &ada).unwrap(),
        "ada__text_io__put_line"
    );
    assert_eq!(encode(Scheme::AdaGnat, &elaboration).unwrap(), "mypkg_E");
}

#[test]
fn plain_unity_and_objective_c_edge_forms_normalize_cleanly() {
    let plain = decode(Scheme::Plain, "AnimEventLoader::LoadAnimationEventDatabase").unwrap();
    let unity = decode(
        Scheme::UnityIl2Cpp,
        "Animator_GetGoalRotation_mB7B67DE4EBA3C26D713754D1D76D4F529E783DB2",
    )
    .unwrap();
    let mono = decode(Scheme::MonoManaged, "UnityEngine.UI.Text$$get_fontStyle").unwrap();
    let objc = decode(
        Scheme::ObjectiveC,
        "___51-[VUIBackgroundMediaController loadAlphaImageProxy]_block_invoke_2",
    )
    .unwrap();

    assert_eq!(plain.kind, SymbolKind::Method);
    assert_eq!(
        plain.display(),
        "AnimEventLoader::LoadAnimationEventDatabase"
    );

    assert_eq!(unity.kind, SymbolKind::Method);
    assert_eq!(unity.display(), "Animator::GetGoalRotation");

    assert_eq!(mono.kind, SymbolKind::Method);
    assert_eq!(mono.display(), "UnityEngine.UI.Text::get_fontStyle");

    assert_eq!(objc.kind, SymbolKind::Runtime);
    assert_eq!(
        objc.display(),
        "block invoke for -[VUIBackgroundMediaController loadAlphaImageProxy]"
    );
}

#[test]
fn tolerant_itanium_fallback_handles_graalish_names() {
    let symbol = decode(
        Scheme::ItaniumCpp,
        "_ZN29sun.security.rsa.RSASignature16engineInitVerifyEJvP23java.security.PublicKey",
    )
    .unwrap();

    assert_eq!(symbol.kind, SymbolKind::Method);
    assert_eq!(
        symbol.path[0],
        Name::identifier("sun.security.rsa.RSASignature")
    );
    assert_eq!(symbol.path[1], Name::identifier("engineInitVerify"));
    assert_eq!(
        symbol.display(),
        "sun.security.rsa.RSASignature::engineInitVerify"
    );
}
