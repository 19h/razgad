// ffi.cpp — Thin C wrapper around Swift's C++ demangler for Rust FFI.
//
// This file provides a simple C interface that Rust can call via FFI.
// It uses the upstream Swift demangler's Context API directly.

#include "swift/Demangling/Demangle.h"
#include "swift/Demangling/Demangler.h"

#include <cstdlib>
#include <cstring>

extern "C" {

/// Demangle a Swift symbol and return the result as a newly allocated
/// null-terminated C string. The caller must free the returned pointer
/// with `razgad_swift_demangle_free`. Returns NULL if the input is not
/// a valid Swift mangled symbol.
const char *razgad_swift_demangle(const char *mangled_name, size_t mangled_len) {
    if (!mangled_name || mangled_len == 0)
        return nullptr;

    llvm::StringRef input(mangled_name, mangled_len);

    if (!swift::Demangle::isSwiftSymbol(input))
        return nullptr;

    swift::Demangle::DemangleOptions opts;
    opts.SynthesizeSugarOnTypes = true;

    std::string result = swift::Demangle::demangleSymbolAsString(input, opts);

    // If the demangler returned the input unchanged, it failed.
    if (result == std::string(mangled_name, mangled_len))
        return nullptr;

    char *out = static_cast<char *>(malloc(result.size() + 1));
    if (!out)
        return nullptr;
    memcpy(out, result.c_str(), result.size() + 1);
    return out;
}

/// Free a string returned by razgad_swift_demangle.
void razgad_swift_demangle_free(const char *ptr) {
    free(const_cast<char *>(ptr));
}

} // extern "C"
