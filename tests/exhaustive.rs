use symdem::{decode, encode, heuristic_decode, Confidence, Scheme, SymbolKind};

#[derive(Clone, Copy)]
struct Case {
    scheme: Scheme,
    mangled: &'static str,
    display: &'static str,
    kind: SymbolKind,
    heuristic: Option<(Scheme, Confidence)>,
}

fn cases() -> Vec<Case> {
    vec![
        Case {
            scheme: Scheme::ItaniumCpp,
            mangled: "_Z3foov",
            display: "foo()",
            kind: SymbolKind::Function,
            heuristic: Some((Scheme::ItaniumCpp, Confidence::Certain)),
        },
        Case {
            scheme: Scheme::ItaniumCpp,
            mangled: "_ZN4demo3VecIiE4pushERKi",
            display: "demo::Vec<int>::push(int const&)",
            kind: SymbolKind::Method,
            heuristic: Some((Scheme::ItaniumCpp, Confidence::Certain)),
        },
        Case {
            scheme: Scheme::ItaniumCpp,
            mangled: "_ZTVN4demo6WidgetE",
            display: "vtable for demo::Widget",
            kind: SymbolKind::VTable,
            heuristic: Some((Scheme::ItaniumCpp, Confidence::Certain)),
        },
        Case {
            scheme: Scheme::ItaniumCpp,
            mangled: "_ZThn8_N4demo7Derived1gEv",
            display: "non-virtual thunk to demo::Derived::g()",
            kind: SymbolKind::Thunk,
            heuristic: Some((Scheme::ItaniumCpp, Confidence::Certain)),
        },
        Case {
            scheme: Scheme::MicrosoftCpp,
            mangled: "?alpha@demo@@YAXH@Z",
            display: "demo::alpha(int)",
            kind: SymbolKind::Function,
            heuristic: Some((Scheme::MicrosoftCpp, Confidence::Certain)),
        },
        Case {
            scheme: Scheme::MicrosoftCpp,
            mangled: "?run@Widget@demo@@QEAAXH@Z",
            display: "demo::Widget::run(int)",
            kind: SymbolKind::Method,
            heuristic: Some((Scheme::MicrosoftCpp, Confidence::Certain)),
        },
        Case {
            scheme: Scheme::MicrosoftCpp,
            mangled: "??0Widget@demo@@QEAA@XZ",
            display: "demo::Widget::Widget()",
            kind: SymbolKind::Constructor,
            heuristic: Some((Scheme::MicrosoftCpp, Confidence::Certain)),
        },
        Case {
            scheme: Scheme::MicrosoftCpp,
            mangled: "??_7Widget@demo@@6B@",
            display: "vftable for demo::Widget",
            kind: SymbolKind::VTable,
            heuristic: Some((Scheme::MicrosoftCpp, Confidence::Certain)),
        },
        Case {
            scheme: Scheme::MicrosoftCpp,
            mangled: "??_R0?AUWidget@demo@@@8",
            display: "RTTI Type Descriptor for demo::Widget",
            kind: SymbolKind::Metadata,
            heuristic: Some((Scheme::MicrosoftCpp, Confidence::Certain)),
        },
        Case {
            scheme: Scheme::BorlandCpp,
            mangled: "@h$qv",
            display: "h()",
            kind: SymbolKind::Function,
            heuristic: Some((Scheme::BorlandCpp, Confidence::High)),
        },
        Case {
            scheme: Scheme::BorlandCpp,
            mangled: "@h$qi",
            display: "h(int)",
            kind: SymbolKind::Function,
            heuristic: Some((Scheme::BorlandCpp, Confidence::High)),
        },
        Case {
            scheme: Scheme::WatcomCpp,
            mangled: "W?h$n()v",
            display: "h()",
            kind: SymbolKind::Function,
            heuristic: Some((Scheme::WatcomCpp, Confidence::Certain)),
        },
        Case {
            scheme: Scheme::WatcomCpp,
            mangled: "W?h$n(i)v",
            display: "h(int)",
            kind: SymbolKind::Function,
            heuristic: Some((Scheme::WatcomCpp, Confidence::Certain)),
        },
        Case {
            scheme: Scheme::DigitalMars,
            mangled: "_f",
            display: "f",
            kind: SymbolKind::Function,
            heuristic: Some((Scheme::Cdecl, Confidence::Medium)),
        },
        Case {
            scheme: Scheme::DigitalMars,
            mangled: "_g@4",
            display: "g",
            kind: SymbolKind::Function,
            heuristic: Some((Scheme::Stdcall, Confidence::High)),
        },
        Case {
            scheme: Scheme::IbmXlCppLegacy,
            mangled: "foo__Fi",
            display: "foo(int)",
            kind: SymbolKind::Function,
            heuristic: Some((Scheme::IbmXlCppLegacy, Confidence::Medium)),
        },
        Case {
            scheme: Scheme::IbmXlCppLegacy,
            mangled: "__ct__3FooFv",
            display: "Foo::Foo()",
            kind: SymbolKind::Constructor,
            heuristic: Some((Scheme::CfrontCpp, Confidence::Medium)),
        },
        Case {
            scheme: Scheme::HpAccCppLegacy,
            mangled: "h__Fv",
            display: "h()",
            kind: SymbolKind::Function,
            heuristic: Some((Scheme::HpAccCppLegacy, Confidence::Medium)),
        },
        Case {
            scheme: Scheme::HpAccCppLegacy,
            mangled: "h__Fic",
            display: "h(int, char)",
            kind: SymbolKind::Function,
            heuristic: Some((Scheme::HpAccCppLegacy, Confidence::Medium)),
        },
        Case {
            scheme: Scheme::SunStudioCppLegacy,
            mangled: "__1cBh6Fi_v_",
            display: "h(int)",
            kind: SymbolKind::Function,
            heuristic: Some((Scheme::SunStudioCppLegacy, Confidence::Certain)),
        },
        Case {
            scheme: Scheme::SunStudioCppLegacy,
            mangled: "__1cGstrcmp6Fpkc1_i_",
            display: "strcmp(char const*, char const*)",
            kind: SymbolKind::Function,
            heuristic: Some((Scheme::SunStudioCppLegacy, Confidence::Certain)),
        },
        Case {
            scheme: Scheme::CfrontCpp,
            mangled: "f__Fi",
            display: "f(int)",
            kind: SymbolKind::Function,
            heuristic: Some((Scheme::CfrontCpp, Confidence::Medium)),
        },
        Case {
            scheme: Scheme::CfrontCpp,
            mangled: "__dt__3FooFv",
            display: "Foo::~Foo()",
            kind: SymbolKind::Destructor,
            heuristic: Some((Scheme::CfrontCpp, Confidence::Medium)),
        },
        Case {
            scheme: Scheme::ArmCppLegacy,
            mangled: "__ct__3FooFv",
            display: "Foo::Foo()",
            kind: SymbolKind::Constructor,
            heuristic: Some((Scheme::CfrontCpp, Confidence::Medium)),
        },
        Case {
            scheme: Scheme::GreenHillsCpp,
            mangled: "bar__3FooFi",
            display: "Foo::bar(int)",
            kind: SymbolKind::Method,
            heuristic: Some((Scheme::GreenHillsCpp, Confidence::Low)),
        },
        Case {
            scheme: Scheme::IntelNativeCpp,
            mangled: "?alpha@demo@@YAXH@Z",
            display: "demo::alpha(int)",
            kind: SymbolKind::Function,
            heuristic: Some((Scheme::MicrosoftCpp, Confidence::Certain)),
        },
        Case {
            scheme: Scheme::IntelNativeCpp,
            mangled: "_ZN4demo5alphaEv",
            display: "demo::alpha()",
            kind: SymbolKind::Function,
            heuristic: Some((Scheme::ItaniumCpp, Confidence::Certain)),
        },
        Case {
            scheme: Scheme::EdgCppLegacy,
            mangled: "f__Fi",
            display: "f(int)",
            kind: SymbolKind::Function,
            heuristic: Some((Scheme::CfrontCpp, Confidence::Medium)),
        },
        Case {
            scheme: Scheme::CrayCpp,
            mangled: "_ZN4demo5alphaEv",
            display: "demo::alpha()",
            kind: SymbolKind::Function,
            heuristic: Some((Scheme::ItaniumCpp, Confidence::Certain)),
        },
        Case {
            scheme: Scheme::SgiMipsproCpp,
            mangled: "bar__Q23ns3FooFi",
            display: "ns::Foo::bar(int)",
            kind: SymbolKind::Method,
            heuristic: Some((Scheme::SgiMipsproCpp, Confidence::Medium)),
        },
        Case {
            scheme: Scheme::MetrowerksCpp,
            mangled: "__ct__Q23foo3BarFv",
            display: "foo::Bar::Bar()",
            kind: SymbolKind::Constructor,
            heuristic: Some((Scheme::MetrowerksCpp, Confidence::Medium)),
        },
        Case {
            scheme: Scheme::Cdecl,
            mangled: "_cdecl_fn",
            display: "cdecl_fn",
            kind: SymbolKind::Function,
            heuristic: Some((Scheme::Cdecl, Confidence::High)),
        },
        Case {
            scheme: Scheme::Stdcall,
            mangled: "_stdcall_fn@8",
            display: "stdcall_fn",
            kind: SymbolKind::Function,
            heuristic: Some((Scheme::Stdcall, Confidence::Certain)),
        },
        Case {
            scheme: Scheme::Fastcall,
            mangled: "@fastcall_fn@8",
            display: "fastcall_fn",
            kind: SymbolKind::Function,
            heuristic: Some((Scheme::Fastcall, Confidence::Certain)),
        },
        Case {
            scheme: Scheme::Vectorcall,
            mangled: "vectorcall_fn@@8",
            display: "vectorcall_fn",
            kind: SymbolKind::Function,
            heuristic: Some((Scheme::Vectorcall, Confidence::Certain)),
        },
        Case {
            scheme: Scheme::Pascal,
            mangled: "FOO",
            display: "FOO",
            kind: SymbolKind::Function,
            heuristic: None,
        },
        Case {
            scheme: Scheme::FortranExternal,
            mangled: "foo_",
            display: "foo",
            kind: SymbolKind::Function,
            heuristic: Some((Scheme::FortranExternal, Confidence::High)),
        },
        Case {
            scheme: Scheme::FortranExternal,
            mangled: "foo_bar__",
            display: "foo_bar",
            kind: SymbolKind::Function,
            heuristic: Some((Scheme::FortranExternal, Confidence::High)),
        },
        Case {
            scheme: Scheme::Dlang,
            mangled: "_D4demo5alphaFiZv",
            display: "demo::alpha(int)",
            kind: SymbolKind::Function,
            heuristic: Some((Scheme::Dlang, Confidence::Certain)),
        },
        Case {
            scheme: Scheme::Dlang,
            mangled: "_D4demo4beta5gammaFiZi",
            display: "demo::beta::gamma(int)",
            kind: SymbolKind::Function,
            heuristic: Some((Scheme::Dlang, Confidence::Certain)),
        },
        Case {
            scheme: Scheme::RustLegacy,
            mangled: "__ZN4demo5alpha17h1ac0358795dd9244E",
            display: "demo::alpha",
            kind: SymbolKind::Function,
            heuristic: Some((Scheme::RustLegacy, Confidence::Certain)),
        },
        Case {
            scheme: Scheme::RustLegacy,
            mangled: "__ZN4demo4beta6Widget3run17h549f425ea90ecbe9E",
            display: "demo::beta::Widget::run",
            kind: SymbolKind::Method,
            heuristic: Some((Scheme::RustLegacy, Confidence::Certain)),
        },
        Case {
            scheme: Scheme::RustV0,
            mangled: "__RNvCs9y1O7KqhnLf_4demo5alpha",
            display: "demo::alpha",
            kind: SymbolKind::Function,
            heuristic: Some((Scheme::RustV0, Confidence::Certain)),
        },
        Case {
            scheme: Scheme::RustV0,
            mangled: "__RNvMNtCs9y1O7KqhnLf_4demo4betaNtB2_6Widget3run",
            display: "demo::beta::Widget::run",
            kind: SymbolKind::Method,
            heuristic: Some((Scheme::RustV0, Confidence::Certain)),
        },
        Case {
            scheme: Scheme::Swift,
            mangled: "_$s4Demo5alphayyF",
            display: "Demo.alpha()",
            kind: SymbolKind::Function,
            heuristic: Some((Scheme::Swift, Confidence::Certain)),
        },
        Case {
            scheme: Scheme::Swift,
            mangled: "_$s4Demo6WidgetV3runyS2iF",
            display: "Demo.Widget.run(Swift.Int)",
            kind: SymbolKind::Method,
            heuristic: Some((Scheme::Swift, Confidence::Certain)),
        },
        Case {
            scheme: Scheme::Swift,
            mangled: "_$s4Demo6WidgetVACycfC",
            display: "Demo.Widget.init()",
            kind: SymbolKind::Constructor,
            heuristic: Some((Scheme::Swift, Confidence::Certain)),
        },
        Case {
            scheme: Scheme::ObjectiveC,
            mangled: "-[Point value]",
            display: "-[Point value]",
            kind: SymbolKind::Method,
            heuristic: Some((Scheme::ObjectiveC, Confidence::Certain)),
        },
        Case {
            scheme: Scheme::ObjectiveC,
            mangled: "+[Point origin]",
            display: "+[Point origin]",
            kind: SymbolKind::Method,
            heuristic: Some((Scheme::ObjectiveC, Confidence::Certain)),
        },
        Case {
            scheme: Scheme::ObjectiveC,
            mangled: "_OBJC_CLASS_$_Point",
            display: "Objective-C class Point",
            kind: SymbolKind::Metadata,
            heuristic: Some((Scheme::ObjectiveC, Confidence::Certain)),
        },
        Case {
            scheme: Scheme::ObjectiveC,
            mangled: "v@:",
            display: "void self selector",
            kind: SymbolKind::TypeEncoding,
            heuristic: Some((Scheme::ObjectiveC, Confidence::Medium)),
        },
        Case {
            scheme: Scheme::Jni,
            mangled: "Java_p_q_r_A_g",
            display: "p.q.r.A.g",
            kind: SymbolKind::Function,
            heuristic: Some((Scheme::Jni, Confidence::Certain)),
        },
        Case {
            scheme: Scheme::Jni,
            mangled: "Java_p_q_r_A_f__ILjava_lang_String_2",
            display: "p.q.r.A.f(int, java.lang.String)",
            kind: SymbolKind::Function,
            heuristic: Some((Scheme::Jni, Confidence::Certain)),
        },
        Case {
            scheme: Scheme::DotNet,
            mangled: "List`1",
            display: "List<T0>",
            kind: SymbolKind::Type,
            heuristic: Some((Scheme::DotNet, Confidence::High)),
        },
        Case {
            scheme: Scheme::DotNet,
            mangled: "Dictionary`2",
            display: "Dictionary<T0, T1>",
            kind: SymbolKind::Type,
            heuristic: Some((Scheme::DotNet, Confidence::High)),
        },
        Case {
            scheme: Scheme::Haskell,
            mangled: "ghczmprim_GHCziTypes_ZMZN_closure",
            display: "GHC.Types.[] closure",
            kind: SymbolKind::Closure,
            heuristic: Some((Scheme::Haskell, Confidence::High)),
        },
        Case {
            scheme: Scheme::Haskell,
            mangled: "base_GHCziBase_zpzp_closure",
            display: "GHC.Base.++ closure",
            kind: SymbolKind::Closure,
            heuristic: Some((Scheme::Haskell, Confidence::High)),
        },
        Case {
            scheme: Scheme::AdaGnat,
            mangled: "ada__text_io__put_line",
            display: "ada.text_io.put_line",
            kind: SymbolKind::Function,
            heuristic: Some((Scheme::AdaGnat, Confidence::High)),
        },
        Case {
            scheme: Scheme::AdaGnat,
            mangled: "mypkg_E",
            display: "mypkg elaboration",
            kind: SymbolKind::ModuleInit,
            heuristic: Some((Scheme::AdaGnat, Confidence::High)),
        },
        Case {
            scheme: Scheme::GfortranModule,
            mangled: "__sample_MOD_five",
            display: "sample::five",
            kind: SymbolKind::Function,
            heuristic: Some((Scheme::GfortranModule, Confidence::Certain)),
        },
        Case {
            scheme: Scheme::Ocaml,
            mangled: "camlFoo__bar_123",
            display: "Foo.bar",
            kind: SymbolKind::Function,
            heuristic: Some((Scheme::Ocaml, Confidence::Certain)),
        },
        Case {
            scheme: Scheme::Ocaml,
            mangled: "camlFoo__entry",
            display: "Foo.entry",
            kind: SymbolKind::ModuleInit,
            heuristic: Some((Scheme::Ocaml, Confidence::Certain)),
        },
        Case {
            scheme: Scheme::Go,
            mangled: "main.main",
            display: "main.main",
            kind: SymbolKind::Function,
            heuristic: Some((Scheme::Go, Confidence::Medium)),
        },
        Case {
            scheme: Scheme::Go,
            mangled: "fmt.Println",
            display: "fmt.Println",
            kind: SymbolKind::Function,
            heuristic: Some((Scheme::Go, Confidence::Medium)),
        },
        Case {
            scheme: Scheme::Go,
            mangled: "main.(*T).Method",
            display: "main.(*T).Method",
            kind: SymbolKind::Method,
            heuristic: Some((Scheme::Go, Confidence::Medium)),
        },
        Case {
            scheme: Scheme::Zig,
            mangled: "demo.math.add__anon_42",
            display: "demo.math.add",
            kind: SymbolKind::Function,
            heuristic: None,
        },
        Case {
            scheme: Scheme::Nim,
            mangled: "NimMain",
            display: "NimMain",
            kind: SymbolKind::ModuleInit,
            heuristic: Some((Scheme::Nim, Confidence::Medium)),
        },
        Case {
            scheme: Scheme::Nim,
            mangled: "NimDestroyGlobals",
            display: "NimDestroyGlobals",
            kind: SymbolKind::Runtime,
            heuristic: Some((Scheme::Nim, Confidence::Medium)),
        },
        Case {
            scheme: Scheme::PascalDelphi,
            mangled: "@Unit1@Foo$qqri",
            display: "Unit1.Foo(int)",
            kind: SymbolKind::Function,
            heuristic: Some((Scheme::PascalDelphi, Confidence::High)),
        },
        Case {
            scheme: Scheme::PascalDelphi,
            mangled: "P$UNIT1_$$_FOO$LONGINT$$LONGINT",
            display: "Unit1.Foo(longint)",
            kind: SymbolKind::Function,
            heuristic: Some((Scheme::PascalDelphi, Confidence::High)),
        },
        Case {
            scheme: Scheme::Modula,
            mangled: "Storage_open",
            display: "Storage.open",
            kind: SymbolKind::Function,
            heuristic: None,
        },
        Case {
            scheme: Scheme::Crystal,
            mangled: "*puts",
            display: "puts",
            kind: SymbolKind::Function,
            heuristic: Some((Scheme::Crystal, Confidence::Medium)),
        },
        Case {
            scheme: Scheme::Vlang,
            mangled: "main__main",
            display: "main.main",
            kind: SymbolKind::Function,
            heuristic: Some((Scheme::Vlang, Confidence::High)),
        },
        Case {
            scheme: Scheme::Vlang,
            mangled: "strings__Builder_str",
            display: "strings.Builder.str",
            kind: SymbolKind::Method,
            heuristic: Some((Scheme::Vlang, Confidence::High)),
        },
        Case {
            scheme: Scheme::CarbonCpp,
            mangled: "_ZN4demo5alphaEv",
            display: "demo::alpha()",
            kind: SymbolKind::Function,
            heuristic: Some((Scheme::ItaniumCpp, Confidence::Certain)),
        },
        Case {
            scheme: Scheme::WebAssembly,
            mangled: "env::puts",
            display: "env::puts",
            kind: SymbolKind::Import,
            heuristic: Some((Scheme::WebAssembly, Confidence::Medium)),
        },
        Case {
            scheme: Scheme::MachO,
            mangled: "__ZN4demo5alphaEv",
            display: "demo::alpha()",
            kind: SymbolKind::Function,
            heuristic: Some((Scheme::MachO, Confidence::Certain)),
        },
        Case {
            scheme: Scheme::MachO,
            mangled: "__ZTVN4demo6WidgetE",
            display: "vtable for demo::Widget",
            kind: SymbolKind::VTable,
            heuristic: Some((Scheme::MachO, Confidence::Certain)),
        },
        Case {
            scheme: Scheme::CoffPe,
            mangled: "__imp_?alpha@demo@@YAXH@Z",
            display: "import thunk for demo::alpha(int)",
            kind: SymbolKind::Import,
            heuristic: Some((Scheme::CoffPe, Confidence::Certain)),
        },
        Case {
            scheme: Scheme::CoffPe,
            mangled: "__imp__stdcall_fn@8",
            display: "import thunk for stdcall_fn",
            kind: SymbolKind::Import,
            heuristic: Some((Scheme::CoffPe, Confidence::Certain)),
        },
        Case {
            scheme: Scheme::Elf,
            mangled: "_ZN4demo5alphaEv@@GLIBCXX_3.4",
            display: "demo::alpha()@GLIBCXX_3.4",
            kind: SymbolKind::Function,
            heuristic: Some((Scheme::Elf, Confidence::Certain)),
        },
        Case {
            scheme: Scheme::Elf,
            mangled: "foo@@GLIBC_2.2.5",
            display: "foo@GLIBC_2.2.5",
            kind: SymbolKind::Function,
            heuristic: Some((Scheme::Elf, Confidence::Certain)),
        },
        Case {
            scheme: Scheme::Os400Cpp,
            mangled: "__ct__Q23foo3BarFv",
            display: "foo::Bar::Bar()",
            kind: SymbolKind::Constructor,
            heuristic: None,
        },
        Case {
            scheme: Scheme::Vms,
            mangled: "H__XI",
            display: "h(int)",
            kind: SymbolKind::Function,
            heuristic: Some((Scheme::Vms, Confidence::Medium)),
        },
        Case {
            scheme: Scheme::Vms,
            mangled: "CXX$_Z1HV0BCA19V",
            display: "h()",
            kind: SymbolKind::Function,
            heuristic: Some((Scheme::Vms, Confidence::Medium)),
        },
        Case {
            scheme: Scheme::Plain,
            mangled: "AnimEventLoader::LoadAnimationEventDatabase",
            display: "AnimEventLoader::LoadAnimationEventDatabase",
            kind: SymbolKind::Method,
            heuristic: Some((Scheme::Plain, Confidence::High)),
        },
        Case {
            scheme: Scheme::Plain,
            mangled: "tls1_new",
            display: "tls1_new",
            kind: SymbolKind::Function,
            heuristic: Some((Scheme::Plain, Confidence::Medium)),
        },
        Case {
            scheme: Scheme::UnityIl2Cpp,
            mangled: "Animator_GetGoalRotation_mB7B67DE4EBA3C26D713754D1D76D4F529E783DB2",
            display: "Animator::GetGoalRotation",
            kind: SymbolKind::Method,
            heuristic: Some((Scheme::UnityIl2Cpp, Confidence::High)),
        },
        Case {
            scheme: Scheme::UnityIl2Cpp,
            mangled: "BurstString_ConvertDoubleToString_m5B4644F134166CA236077075A11108590892EDD0",
            display: "BurstString::ConvertDoubleToString",
            kind: SymbolKind::Method,
            heuristic: Some((Scheme::UnityIl2Cpp, Confidence::High)),
        },
        Case {
            scheme: Scheme::MonoManaged,
            mangled: "UnityEngine.UI.Text$$get_fontStyle",
            display: "UnityEngine.UI.Text::get_fontStyle",
            kind: SymbolKind::Method,
            heuristic: Some((Scheme::MonoManaged, Confidence::Certain)),
        },
        Case {
            scheme: Scheme::MonoManaged,
            mangled: "MS.Internal.Xml.XPath.XPathParser$$PassToken",
            display: "MS.Internal.Xml.XPath.XPathParser::PassToken",
            kind: SymbolKind::Method,
            heuristic: Some((Scheme::MonoManaged, Confidence::Certain)),
        },
        Case {
            scheme: Scheme::ItaniumCpp,
            mangled: "_ZN29sun.security.rsa.RSASignature16engineInitVerifyEJvP23java.security.PublicKey",
            display: "sun.security.rsa.RSASignature::engineInitVerify",
            kind: SymbolKind::Method,
            heuristic: Some((Scheme::ItaniumCpp, Confidence::Certain)),
        },
        Case {
            scheme: Scheme::ObjectiveC,
            mangled: "___51-[VUIBackgroundMediaController loadAlphaImageProxy]_block_invoke_2",
            display: "block invoke for -[VUIBackgroundMediaController loadAlphaImageProxy]",
            kind: SymbolKind::Runtime,
            heuristic: Some((Scheme::ObjectiveC, Confidence::High)),
        },
        Case {
            scheme: Scheme::ObjectiveC,
            mangled: "-[NSViewServiceMarshal _bootstrap:replyData:completion:].cold.3",
            display: "cold clone of -[NSViewServiceMarshal _bootstrap:replyData:completion:]",
            kind: SymbolKind::Runtime,
            heuristic: Some((Scheme::ObjectiveC, Confidence::High)),
        },
        Case {
            scheme: Scheme::ItaniumCpp,
            mangled: "_ZN44com.oracle.svm.core.code.FactoryMethodHolder49QTESLAPrivateKeyParameters_9WuwyiRv36EGKEGTTQcfPAEJP68org.bouncycastle.pqc.legacy.crypto.qtesla.QTESLAPrivateKeyParametersiP6byte[]",
            display: "com.oracle.svm.core.code.FactoryMethodHolder::QTESLAPrivateKeyParameters_9WuwyiRv36EGKEGTTQcfPA",
            kind: SymbolKind::Method,
            heuristic: Some((Scheme::ItaniumCpp, Confidence::Certain)),
        },
        Case {
            scheme: Scheme::ItaniumCpp,
            mangled: "_Z3foov.isra.0",
            display: "foo() [clone .isra.0]",
            kind: SymbolKind::Function,
            heuristic: Some((Scheme::ItaniumCpp, Confidence::Certain)),
        },
        Case {
            scheme: Scheme::MicrosoftCpp,
            mangled: "??R_lambda_1_@?0???$__insert_range_unique@PEBEPEBE@?$__tree@EU?$less@E@__Cr@std@@V?$allocator@E@23@@__Cr@std@@QEAAXPEBE0@Z@QEBA?A?_auto_@@AEBE1@Z",
            display: "__insert_range_unique::lambda_1_::operator()",
            kind: SymbolKind::Method,
            heuristic: Some((Scheme::MicrosoftCpp, Confidence::Certain)),
        },
        Case {
            scheme: Scheme::MicrosoftCpp,
            mangled: "?__invoke@_lambda_1_@?0???$RegisterWebUIControllerInterfaceBinder@VLensPageHandlerFactory@mojom@lens@@VLensOverlayUntrustedUI@3@@content@@YAXPEAV?$BinderMapWithContext@PEAVRenderFrameHost@content@@@mojo@@@Z@CA?A?_auto_@@PEAVRenderFrameHost@2@V?$PendingReceiver@VLensPageHandlerFactory@mojom@lens@@@4@@Z",
            display: "content::LensOverlayUntrustedUI::lens::mojom::LensPageHandlerFactory::RegisterWebUIControllerInterfaceBinder::_lambda_1_::__invoke",
            kind: SymbolKind::Method,
            heuristic: Some((Scheme::MicrosoftCpp, Confidence::Certain)),
        },
        Case {
            scheme: Scheme::MachO,
            mangled: "__ZNK6webrtc23RtpTransceiverInterface8receiverEv_vfpthunk_",
            display: "webrtc::RtpTransceiverInterface::receiver() const",
            kind: SymbolKind::Method,
            heuristic: Some((Scheme::MachO, Confidence::Certain)),
        },
        Case {
            scheme: Scheme::MachO,
            mangled: "__ZNKSt3__110__function6__funcIZ32-[VKMapView initWithDescriptor:]E3$_2NS_9allocatorIS2_EEFvvEE7__cloneEv",
            display: "macho wrapper for -[VKMapView initWithDescriptor:]",
            kind: SymbolKind::Runtime,
            heuristic: Some((Scheme::MachO, Confidence::Certain)),
        },
        Case {
            scheme: Scheme::CfrontCpp,
            mangled: ".getloc__Q23std8ios_baseCFv",
            display: "std::ios_base::getloc()",
            kind: SymbolKind::Method,
            heuristic: Some((Scheme::CfrontCpp, Confidence::Medium)),
        },
    ]
}

#[test]
fn fixture_catalog_covers_every_public_scheme() {
    let mut seen = std::collections::BTreeSet::new();
    for case in cases() {
        seen.insert(case.scheme);
    }

    let expected = Scheme::all_public();
    let actual: Vec<_> = seen.into_iter().collect();
    assert_eq!(
        actual, expected,
        "fixture catalog must cover every public scheme"
    );
}

#[test]
fn explicit_decode_matches_fixture_catalog() {
    for case in cases() {
        let symbol = decode(case.scheme, case.mangled).unwrap_or_else(|err| {
            panic!(
                "failed to decode {} as {:?}: {err}",
                case.mangled, case.scheme
            )
        });
        assert_eq!(symbol.kind, case.kind, "wrong kind for {}", case.mangled);
        assert_eq!(
            symbol.display(),
            case.display,
            "wrong display for {}",
            case.mangled
        );
    }
}

#[test]
fn decode_then_encode_round_trips_every_fixture() {
    for case in cases() {
        let symbol = decode(case.scheme, case.mangled).unwrap_or_else(|err| {
            panic!(
                "failed to decode {} as {:?}: {err}",
                case.mangled, case.scheme
            )
        });
        let reencoded = encode(case.scheme, &symbol).unwrap_or_else(|err| {
            panic!(
                "failed to encode {} as {:?}: {err}",
                case.mangled, case.scheme
            )
        });
        assert_eq!(
            reencoded, case.mangled,
            "round trip mismatch for {}",
            case.mangled
        );
    }
}

#[test]
fn heuristic_decode_finds_expected_scheme_and_confidence() {
    for case in cases() {
        let Some((expected_scheme, minimum_confidence)) = case.heuristic else {
            continue;
        };
        let detected = heuristic_decode(case.mangled)
            .unwrap_or_else(|err| panic!("heuristic decode failed for {}: {err}", case.mangled));
        assert_eq!(
            detected.scheme, expected_scheme,
            "wrong heuristic scheme for {}",
            case.mangled
        );
        assert!(
            detected.confidence >= minimum_confidence,
            "heuristic confidence too low for {}",
            case.mangled
        );
        assert_eq!(
            detected.symbol.display(),
            case.display,
            "heuristic display mismatch for {}",
            case.mangled
        );
    }
}
