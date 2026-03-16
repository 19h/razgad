use std::ffi::CStr;
use std::os::raw::c_char;

unsafe extern "C" {
    fn razgad_swift_demangle(mangled_name: *const c_char, mangled_len: usize) -> *const c_char;
    fn razgad_swift_demangle_free(ptr: *const c_char);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Error {
    InvalidSymbol,
}

pub(crate) fn demangle(input: &str) -> Result<String, Error> {
    let ptr = unsafe { razgad_swift_demangle(input.as_ptr() as *const c_char, input.len()) };
    if ptr.is_null() {
        return Err(Error::InvalidSymbol);
    }
    let result = unsafe { CStr::from_ptr(ptr) }
        .to_string_lossy()
        .into_owned();
    unsafe { razgad_swift_demangle_free(ptr) };
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_new_mangling() {
        let result = demangle("$s4main5helloyyF").unwrap();
        assert_eq!(result, "main.hello() -> ()");
    }

    #[test]
    fn basic_new_mangling_2() {
        let result = demangle("$s4main3FooCACycfc").unwrap();
        assert_eq!(result, "main.Foo.init() -> main.Foo");
    }

    #[test]
    fn invalid_symbol() {
        assert!(demangle("not_a_swift_symbol").is_err());
    }

    #[test]
    fn old_mangling() {
        // _T prefix — old mangling, should work since we compiled with
        // SWIFT_SUPPORT_OLD_MANGLING=1
        let result = demangle("_TFC4main3Foo3barfS0_FT_Si");
        assert!(result.is_ok(), "Old mangling should succeed: {:?}", result);
    }

    #[test]
    fn generic_function() {
        let result = demangle("$s4main3fooyxxlF").unwrap();
        assert!(result.contains("main.foo"), "Got: {result}");
    }
}
