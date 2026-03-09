use crate::{
    CallingConvention, Error, Name, Scheme, Signature, SpecialKind, Symbol, SymbolKind, Type,
};

pub fn encode_symbol(scheme: Scheme, symbol: &Symbol) -> Result<String, Error> {
    if let Some(verbatim) = &symbol.verbatim {
        return Ok(verbatim.clone());
    }

    match scheme {
        Scheme::ItaniumCpp => encode_itanium(symbol),
        Scheme::MicrosoftCpp => encode_microsoft(symbol),
        Scheme::Cdecl => encode_cdecl(symbol),
        Scheme::Stdcall => encode_stdcall(symbol),
        Scheme::Fastcall => encode_fastcall(symbol),
        Scheme::Vectorcall => encode_vectorcall(symbol),
        Scheme::Dlang => encode_dlang(symbol),
        Scheme::Jni => encode_jni(symbol),
        Scheme::ObjectiveC => encode_objective_c(symbol),
        Scheme::Vlang => encode_vlang(symbol),
        Scheme::FortranExternal => encode_fortran_external(symbol),
        Scheme::GfortranModule => encode_gfortran_module(symbol),
        Scheme::AdaGnat => encode_ada(symbol),
        Scheme::MachO => encode_macho(symbol),
        Scheme::CoffPe => encode_coff(symbol),
        Scheme::Elf => encode_elf(symbol),
        Scheme::Plain => encode_plain(symbol),
        Scheme::UnityIl2Cpp => encode_unity_il2cpp(symbol),
        Scheme::MonoManaged => encode_mono_managed(symbol),
        Scheme::IntelNativeCpp => match symbol.concrete_family {
            Scheme::MicrosoftCpp => encode_microsoft(symbol),
            _ => encode_itanium(symbol),
        },
        Scheme::CarbonCpp | Scheme::CrayCpp => encode_itanium(symbol),
        _ => Err(Error::new(format!(
            "canonical encoding is not implemented for {scheme:?}"
        ))),
    }
}

fn encode_itanium(symbol: &Symbol) -> Result<String, Error> {
    let signature = symbol
        .signature
        .as_ref()
        .ok_or_else(|| Error::new("Itanium symbol requires a signature"))?;
    let names = encode_itanium_path(&symbol.path)?;
    let params = encode_itanium_params(signature)?;
    Ok(format!("_Z{names}{params}"))
}

fn encode_itanium_path(path: &[Name]) -> Result<String, Error> {
    if path.is_empty() {
        return Err(Error::new("Itanium symbol requires at least one name"));
    }
    let encoded = path
        .iter()
        .map(encode_simple_name)
        .collect::<Result<Vec<_>, _>>()?
        .join("");
    if path.len() == 1 {
        Ok(encoded)
    } else {
        Ok(format!("N{encoded}E"))
    }
}

fn encode_itanium_params(signature: &Signature) -> Result<String, Error> {
    if signature.parameters.is_empty() {
        return Ok("v".to_string());
    }
    let mut out = String::new();
    for param in &signature.parameters {
        out.push_str(&encode_itanium_type(param)?);
    }
    Ok(out)
}

fn encode_itanium_type(ty: &Type) -> Result<String, Error> {
    match ty {
        Type::Void => Ok("v".to_string()),
        Type::Int => Ok("i".to_string()),
        Type::ConstRef(inner) => Ok(format!("RK{}", encode_itanium_type(inner)?)),
        _ => Err(Error::new("unsupported Itanium type")),
    }
}

fn encode_microsoft(symbol: &Symbol) -> Result<String, Error> {
    if symbol.kind != SymbolKind::Function {
        return Err(Error::new(
            "only free MSVC functions are supported canonically",
        ));
    }
    let signature = symbol
        .signature
        .as_ref()
        .ok_or_else(|| Error::new("MSVC symbol requires a signature"))?;
    if signature.calling_convention != Some(CallingConvention::Cdecl)
        || signature.return_type != Some(Type::void())
    {
        return Err(Error::new("unsupported MSVC signature"));
    }
    if symbol.path.is_empty() {
        return Err(Error::new("MSVC symbol requires a path"));
    }
    let function = simple_identifier(symbol.path.last().unwrap())?;
    let mut out = String::new();
    out.push('?');
    out.push_str(function);
    out.push('@');
    for scope in symbol.path[..symbol.path.len() - 1].iter().rev() {
        out.push_str(simple_identifier(scope)?);
        out.push('@');
    }
    out.push_str("@YAX");
    if signature.parameters.is_empty() {
        out.push('X');
    } else {
        for param in &signature.parameters {
            out.push_str(&encode_msvc_type(param)?);
        }
        out.push('@');
    }
    out.push('Z');
    Ok(out)
}

fn encode_msvc_type(ty: &Type) -> Result<String, Error> {
    match ty {
        Type::Int => Ok("H".to_string()),
        _ => Err(Error::new("unsupported MSVC type")),
    }
}

fn encode_dlang(symbol: &Symbol) -> Result<String, Error> {
    let signature = symbol
        .signature
        .as_ref()
        .ok_or_else(|| Error::new("D symbol requires a signature"))?;
    let mut out = String::from("_D");
    for part in &symbol.path {
        let name = simple_identifier(part)?;
        out.push_str(&name.len().to_string());
        out.push_str(name);
    }
    out.push('F');
    for param in &signature.parameters {
        out.push_str(&encode_d_type(param)?);
    }
    out.push('Z');
    out.push_str(&encode_d_type(
        signature
            .return_type
            .as_ref()
            .ok_or_else(|| Error::new("D symbol requires a return type"))?,
    )?);
    Ok(out)
}

fn encode_cdecl(symbol: &Symbol) -> Result<String, Error> {
    Ok(format!("_{}", last_identifier(symbol)?))
}

fn encode_stdcall(symbol: &Symbol) -> Result<String, Error> {
    Ok(format!(
        "_{}@{}",
        last_identifier(symbol)?,
        stack_bytes(symbol)?
    ))
}

fn encode_fastcall(symbol: &Symbol) -> Result<String, Error> {
    Ok(format!(
        "@{}@{}",
        last_identifier(symbol)?,
        stack_bytes(symbol)?
    ))
}

fn encode_vectorcall(symbol: &Symbol) -> Result<String, Error> {
    Ok(format!(
        "{}@@{}",
        last_identifier(symbol)?,
        stack_bytes(symbol)?
    ))
}

fn encode_d_type(ty: &Type) -> Result<String, Error> {
    match ty {
        Type::Void => Ok("v".to_string()),
        Type::Int => Ok("i".to_string()),
        _ => Err(Error::new("unsupported D type")),
    }
}

fn encode_jni(symbol: &Symbol) -> Result<String, Error> {
    if symbol.path.is_empty() {
        return Err(Error::new("JNI symbol requires a path"));
    }
    let mut out = String::from("Java_");
    let parts = symbol
        .path
        .iter()
        .map(simple_identifier)
        .collect::<Result<Vec<_>, _>>()?;
    out.push_str(&parts.join("_"));
    let signature = symbol.signature.as_ref();
    if let Some(signature) = signature {
        if !signature.parameters.is_empty() {
            out.push_str("__");
            for param in &signature.parameters {
                out.push_str(&encode_jni_type(param)?);
            }
        }
    }
    Ok(out)
}

fn encode_jni_type(ty: &Type) -> Result<String, Error> {
    match ty {
        Type::Int => Ok("I".to_string()),
        Type::Named(parts) => Ok(format!("L{}_2", parts.join("_"))),
        _ => Err(Error::new("unsupported JNI type")),
    }
}

fn encode_objective_c(symbol: &Symbol) -> Result<String, Error> {
    match symbol.special {
        Some(SpecialKind::ObjectiveCClass) => {
            let name = simple_identifier(
                symbol
                    .path
                    .first()
                    .ok_or_else(|| Error::new("Objective-C class symbol requires a name"))?,
            )?;
            Ok(format!("_OBJC_CLASS_$_{name}"))
        }
        _ => Err(Error::new(
            "only Objective-C class metadata is supported canonically",
        )),
    }
}

fn encode_vlang(symbol: &Symbol) -> Result<String, Error> {
    if symbol.path.len() < 2 {
        return Err(Error::new("V symbol requires at least two path components"));
    }
    let parts = symbol
        .path
        .iter()
        .map(simple_identifier)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(format!("{}__{}", parts[0], parts[1..].join("_")))
}

fn encode_fortran_external(symbol: &Symbol) -> Result<String, Error> {
    let name = last_identifier(symbol)?.to_ascii_lowercase();
    if name.contains('_') {
        Ok(format!("{name}__"))
    } else {
        Ok(format!("{name}_"))
    }
}

fn encode_gfortran_module(symbol: &Symbol) -> Result<String, Error> {
    if symbol.path.len() < 2 {
        return Err(Error::new(
            "gfortran module symbol requires module and name",
        ));
    }
    let parts = symbol
        .path
        .iter()
        .map(simple_identifier)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(format!(
        "__{}_MOD_{}",
        parts[..parts.len() - 1].join("_"),
        parts.last().unwrap()
    ))
}

fn encode_ada(symbol: &Symbol) -> Result<String, Error> {
    if symbol.kind == SymbolKind::ModuleInit {
        return Ok(format!(
            "{}_E",
            last_identifier(symbol)?.to_ascii_lowercase()
        ));
    }
    let parts = symbol
        .path
        .iter()
        .map(simple_identifier)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(parts.join("__").to_ascii_lowercase())
}

fn encode_macho(symbol: &Symbol) -> Result<String, Error> {
    let mut inner = symbol.clone();
    inner.scheme = Scheme::ItaniumCpp;
    inner.concrete_family = Scheme::ItaniumCpp;
    inner.verbatim = None;
    Ok(format!("_{}", encode_itanium(&inner)?))
}

fn encode_coff(symbol: &Symbol) -> Result<String, Error> {
    let inner_scheme = symbol
        .platform
        .inner_scheme
        .ok_or_else(|| Error::new("COFF symbol requires inner scheme"))?;
    let mut inner = symbol.clone();
    inner.scheme = inner_scheme;
    inner.concrete_family = inner_scheme;
    inner.verbatim = None;
    Ok(format!("__imp_{}", encode_symbol(inner_scheme, &inner)?))
}

fn encode_plain(symbol: &Symbol) -> Result<String, Error> {
    if !symbol.path.is_empty() {
        return Ok(symbol
            .path
            .iter()
            .map(simple_identifier)
            .collect::<Result<Vec<_>, _>>()?
            .join("::"));
    }
    Ok(symbol.display())
}

fn encode_unity_il2cpp(symbol: &Symbol) -> Result<String, Error> {
    if symbol.path.len() >= 2 {
        let owner = simple_identifier(&symbol.path[symbol.path.len() - 2])?;
        let method = simple_identifier(symbol.path.last().unwrap())?;
        return Ok(format!("{owner}_{method}"));
    }
    Ok(symbol.display())
}

fn encode_mono_managed(symbol: &Symbol) -> Result<String, Error> {
    if symbol.path.len() >= 2 {
        let owner = symbol.path[..symbol.path.len() - 1]
            .iter()
            .map(simple_identifier)
            .collect::<Result<Vec<_>, _>>()?
            .join(".");
        let method = simple_identifier(symbol.path.last().unwrap())?;
        return Ok(format!("{owner}$${method}"));
    }
    Ok(symbol.display())
}

fn encode_elf(symbol: &Symbol) -> Result<String, Error> {
    let version = symbol
        .platform
        .elf_version
        .as_deref()
        .ok_or_else(|| Error::new("ELF symbol requires a version"))?;
    let mut inner = symbol.clone();
    inner.scheme = Scheme::ItaniumCpp;
    inner.concrete_family = Scheme::ItaniumCpp;
    inner.verbatim = None;
    Ok(format!("{}@@{version}", encode_itanium(&inner)?))
}

fn encode_simple_name(name: &Name) -> Result<String, Error> {
    match name {
        Name::Identifier(name) => Ok(format!("{}{}", name.len(), name)),
        _ => Err(Error::new("unsupported non-identifier name")),
    }
}

fn last_identifier(symbol: &Symbol) -> Result<&str, Error> {
    simple_identifier(
        symbol
            .path
            .last()
            .ok_or_else(|| Error::new("symbol requires at least one path component"))?,
    )
}

fn stack_bytes(symbol: &Symbol) -> Result<usize, Error> {
    let signature = symbol
        .signature
        .as_ref()
        .ok_or_else(|| Error::new("symbol requires a signature"))?;
    Ok(signature.parameters.iter().map(type_stack_bytes).sum())
}

fn type_stack_bytes(ty: &Type) -> usize {
    match ty {
        Type::Void => 0,
        Type::Int => 4,
        Type::ConstRef(_) | Type::Named(_) | Type::Other(_) => 4,
    }
}

fn simple_identifier(name: &Name) -> Result<&str, Error> {
    match name {
        Name::Identifier(name) => Ok(name.as_str()),
        _ => Err(Error::new("unsupported non-identifier name")),
    }
}
