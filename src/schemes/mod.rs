mod dlang;
mod itanium;
mod jni;
mod legacy;
mod msvc;
mod naming;
mod objc;
mod plain;
mod rust;
mod swift;
mod unity;
mod windows_c;
mod wrappers;

use crate::{Error, Scheme, Symbol};

pub fn decode(scheme: Scheme, input: &str) -> Result<Symbol, Error> {
    let result = match scheme {
        Scheme::ItaniumCpp => itanium::decode(scheme, input),
        Scheme::MicrosoftCpp => msvc::decode(scheme, input),
        Scheme::BorlandCpp
        | Scheme::WatcomCpp
        | Scheme::DigitalMars
        | Scheme::IbmXlCppLegacy
        | Scheme::HpAccCppLegacy
        | Scheme::SunStudioCppLegacy
        | Scheme::CfrontCpp
        | Scheme::ArmCppLegacy
        | Scheme::GreenHillsCpp
        | Scheme::EdgCppLegacy
        | Scheme::SgiMipsproCpp
        | Scheme::MetrowerksCpp
        | Scheme::Os400Cpp
        | Scheme::Vms => legacy::decode(scheme, input),
        Scheme::IntelNativeCpp => {
            msvc::decode(scheme, input).or_else(|| itanium::decode(scheme, input))
        }
        Scheme::CrayCpp | Scheme::CarbonCpp => itanium::decode(scheme, input),
        Scheme::Cdecl | Scheme::Stdcall | Scheme::Fastcall | Scheme::Vectorcall => {
            windows_c::decode(scheme, input)
        }
        Scheme::Dlang => dlang::decode(scheme, input),
        Scheme::RustLegacy | Scheme::RustV0 => rust::decode(scheme, input),
        Scheme::Swift => swift::decode(scheme, input),
        Scheme::ObjectiveC => objc::decode(scheme, input),
        Scheme::Jni => jni::decode(scheme, input),
        Scheme::MachO | Scheme::CoffPe | Scheme::Elf => wrappers::decode(scheme, input),
        Scheme::Plain => plain::decode(scheme, input),
        Scheme::UnityIl2Cpp | Scheme::MonoManaged => unity::decode(scheme, input),
        Scheme::Pascal
        | Scheme::FortranExternal
        | Scheme::DotNet
        | Scheme::Haskell
        | Scheme::AdaGnat
        | Scheme::GfortranModule
        | Scheme::Ocaml
        | Scheme::Go
        | Scheme::Zig
        | Scheme::Nim
        | Scheme::PascalDelphi
        | Scheme::Modula
        | Scheme::Crystal
        | Scheme::Vlang
        | Scheme::WebAssembly => naming::decode(scheme, input),
    };

    result.ok_or_else(|| Error::new(format!("decode not implemented for {scheme:?}: {input}")))
}

pub fn detect(input: &str) -> Option<(Scheme, crate::Confidence)> {
    rust::detect(input)
        .or_else(|| swift::detect(input))
        .or_else(|| jni::detect(input))
        .or_else(|| objc::detect(input))
        .or_else(|| dlang::detect(input))
        .or_else(|| msvc::detect(input))
        .or_else(|| windows_c::detect(input))
        .or_else(|| wrappers::detect(input))
        .or_else(|| itanium::detect(input))
        .or_else(|| unity::detect(input))
        .or_else(|| legacy::detect(input))
        .or_else(|| naming::detect(input))
        .or_else(|| plain::detect(input))
}
