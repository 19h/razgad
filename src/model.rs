#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Scheme {
    ItaniumCpp,
    MicrosoftCpp,
    BorlandCpp,
    WatcomCpp,
    DigitalMars,
    IbmXlCppLegacy,
    HpAccCppLegacy,
    SunStudioCppLegacy,
    CfrontCpp,
    ArmCppLegacy,
    GreenHillsCpp,
    IntelNativeCpp,
    EdgCppLegacy,
    CrayCpp,
    SgiMipsproCpp,
    MetrowerksCpp,
    Cdecl,
    Stdcall,
    Fastcall,
    Vectorcall,
    Pascal,
    FortranExternal,
    Dlang,
    RustLegacy,
    RustV0,
    Swift,
    ObjectiveC,
    Jni,
    DotNet,
    Haskell,
    AdaGnat,
    GfortranModule,
    Ocaml,
    Go,
    Zig,
    Nim,
    PascalDelphi,
    Modula,
    Crystal,
    Vlang,
    CarbonCpp,
    WebAssembly,
    MachO,
    CoffPe,
    Elf,
    Os400Cpp,
    Vms,
    Plain,
    UnityIl2Cpp,
    MonoManaged,
}

impl Scheme {
    pub fn all_public() -> Vec<Self> {
        vec![
            Self::ItaniumCpp,
            Self::MicrosoftCpp,
            Self::BorlandCpp,
            Self::WatcomCpp,
            Self::DigitalMars,
            Self::IbmXlCppLegacy,
            Self::HpAccCppLegacy,
            Self::SunStudioCppLegacy,
            Self::CfrontCpp,
            Self::ArmCppLegacy,
            Self::GreenHillsCpp,
            Self::IntelNativeCpp,
            Self::EdgCppLegacy,
            Self::CrayCpp,
            Self::SgiMipsproCpp,
            Self::MetrowerksCpp,
            Self::Cdecl,
            Self::Stdcall,
            Self::Fastcall,
            Self::Vectorcall,
            Self::Pascal,
            Self::FortranExternal,
            Self::Dlang,
            Self::RustLegacy,
            Self::RustV0,
            Self::Swift,
            Self::ObjectiveC,
            Self::Jni,
            Self::DotNet,
            Self::Haskell,
            Self::AdaGnat,
            Self::GfortranModule,
            Self::Ocaml,
            Self::Go,
            Self::Zig,
            Self::Nim,
            Self::PascalDelphi,
            Self::Modula,
            Self::Crystal,
            Self::Vlang,
            Self::CarbonCpp,
            Self::WebAssembly,
            Self::MachO,
            Self::CoffPe,
            Self::Elf,
            Self::Os400Cpp,
            Self::Vms,
            Self::Plain,
            Self::UnityIl2Cpp,
            Self::MonoManaged,
        ]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Confidence {
    Low,
    Medium,
    High,
    Certain,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SymbolKind {
    Function,
    Method,
    Constructor,
    Destructor,
    VTable,
    Thunk,
    Metadata,
    TypeEncoding,
    Type,
    Closure,
    ModuleInit,
    Runtime,
    Import,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpecialKind {
    Vftable,
    RttiTypeDescriptor,
    ObjectiveCClass,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CallingConvention {
    Cdecl,
    Stdcall,
    Fastcall,
    Vectorcall,
    Thiscall,
    Swiftcall,
    Golang,
    Usercall,
    Userpurge,
    D,
    C,
    Cpp,
    ObjectiveC,
    Pascal,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Type {
    Void,
    Int,
    Named(Vec<String>),
    ConstRef(Box<Type>),
    Other(String),
}

impl Type {
    pub fn void() -> Self {
        Self::Void
    }

    pub fn int() -> Self {
        Self::Int
    }

    pub fn named<I, S>(parts: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        Self::Named(
            parts
                .into_iter()
                .map(|part| part.as_ref().to_string())
                .collect(),
        )
    }

    pub fn const_ref(inner: Self) -> Self {
        Self::ConstRef(Box::new(inner))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Name {
    Identifier(String),
    Template { name: String, args: Vec<Type> },
}

impl Name {
    pub fn identifier(name: impl Into<String>) -> Self {
        Self::Identifier(name.into())
    }

    pub fn template(name: impl Into<String>, args: Vec<Type>) -> Self {
        Self::Template {
            name: name.into(),
            args,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PlatformDecorations {
    pub leading_underscore: bool,
    pub import_prefix: bool,
    pub inner_scheme: Option<Scheme>,
    pub elf_version: Option<String>,
}

impl PlatformDecorations {
    pub fn with_elf_version(mut self, version: impl Into<String>) -> Self {
        self.elf_version = Some(version.into());
        self
    }

    pub fn with_inner_scheme(mut self, scheme: Scheme) -> Self {
        self.inner_scheme = Some(scheme);
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Signature {
    pub calling_convention: Option<CallingConvention>,
    pub parameters: Vec<Type>,
    pub return_type: Option<Type>,
}

impl Signature {
    pub fn new(parameters: Vec<Type>) -> Self {
        Self {
            calling_convention: None,
            parameters,
            return_type: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Symbol {
    pub scheme: Scheme,
    pub concrete_family: Scheme,
    pub kind: SymbolKind,
    pub path: Vec<Name>,
    pub signature: Option<Signature>,
    pub special: Option<SpecialKind>,
    pub platform: PlatformDecorations,
    pub verbatim: Option<String>,
    pub(crate) display: Option<String>,
}

impl Symbol {
    pub fn new(scheme: Scheme, kind: SymbolKind) -> Self {
        Self {
            scheme,
            concrete_family: scheme,
            kind,
            path: Vec::new(),
            signature: None,
            special: None,
            platform: PlatformDecorations::default(),
            verbatim: None,
            display: None,
        }
    }

    pub fn function<I, S, J>(scheme: Scheme, path: I, parameters: J) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
        J: IntoIterator<Item = Type>,
    {
        let path = path
            .into_iter()
            .map(|part| Name::identifier(part.as_ref()))
            .collect::<Vec<_>>();
        let mut symbol = Self::new(scheme, SymbolKind::Function);
        symbol.path = path;
        symbol.signature = Some(Signature::new(parameters.into_iter().collect()));
        symbol
    }

    pub fn special<I, S>(scheme: Scheme, special: SpecialKind, path: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut symbol = Self::new(
            scheme,
            match special {
                SpecialKind::ObjectiveCClass | SpecialKind::RttiTypeDescriptor => {
                    SymbolKind::Metadata
                }
                SpecialKind::Vftable => SymbolKind::VTable,
            },
        );
        symbol.special = Some(special);
        symbol.path = path
            .into_iter()
            .map(|part| Name::identifier(part.as_ref()))
            .collect();
        symbol
    }

    pub fn with_return(mut self, return_type: Type) -> Self {
        let signature = self
            .signature
            .get_or_insert_with(|| Signature::new(Vec::new()));
        signature.return_type = Some(return_type);
        self
    }

    pub fn with_calling_convention(mut self, convention: CallingConvention) -> Self {
        let signature = self
            .signature
            .get_or_insert_with(|| Signature::new(Vec::new()));
        signature.calling_convention = Some(convention);
        self
    }

    pub fn with_platform(mut self, platform: PlatformDecorations) -> Self {
        self.platform = platform;
        self
    }

    pub fn with_display(mut self, display: impl Into<String>) -> Self {
        self.display = Some(display.into());
        self
    }

    pub fn with_verbatim(mut self, verbatim: impl Into<String>) -> Self {
        self.verbatim = Some(verbatim.into());
        self
    }

    pub fn display(&self) -> String {
        match &self.display {
            Some(display) => display.clone(),
            None => render_symbol(self),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DetectedSymbol {
    pub scheme: Scheme,
    pub confidence: Confidence,
    pub symbol: Symbol,
}

fn render_symbol(symbol: &Symbol) -> String {
    let mut path = symbol
        .path
        .iter()
        .map(render_name)
        .collect::<Vec<_>>()
        .join("::");

    if let Some(signature) = &symbol.signature {
        let params = signature
            .parameters
            .iter()
            .map(render_type)
            .collect::<Vec<_>>()
            .join(", ");
        path.push('(');
        path.push_str(&params);
        path.push(')');
    }

    path
}

fn render_name(name: &Name) -> String {
    match name {
        Name::Identifier(name) => name.clone(),
        Name::Template { name, args } => {
            let args = args.iter().map(render_type).collect::<Vec<_>>().join(", ");
            format!("{name}<{args}>")
        }
    }
}

fn render_type(ty: &Type) -> String {
    match ty {
        Type::Void => "void".to_string(),
        Type::Int => "int".to_string(),
        Type::Named(parts) => parts.join("::"),
        Type::ConstRef(inner) => format!("{} const&", render_type(inner)),
        Type::Other(name) => name.clone(),
    }
}
