fn main() {
    let mut build = cc::Build::new();

    build
        .cpp(true)
        .std("c++17")
        .include("vendor/swift-demangle/include")
        .define("SWIFT_SUPPORT_OLD_MANGLING", "1")
        .define("SWIFT_STDLIB_HAS_TYPE_PRINTING", "1")
        // Suppress warnings from upstream code — we don't control it.
        .warnings(false);

    // All upstream Demangling source files.
    let sources = [
        "vendor/swift-demangle/src/Context.cpp",
        "vendor/swift-demangle/src/CrashReporter.cpp",
        "vendor/swift-demangle/src/Demangler.cpp",
        "vendor/swift-demangle/src/DemanglingErrorHandling.cpp",
        "vendor/swift-demangle/src/ManglingUtils.cpp",
        "vendor/swift-demangle/src/NodeDumper.cpp",
        "vendor/swift-demangle/src/NodePrinter.cpp",
        "vendor/swift-demangle/src/OldDemangler.cpp",
        "vendor/swift-demangle/src/OldRemangler.cpp",
        "vendor/swift-demangle/src/Punycode.cpp",
        "vendor/swift-demangle/src/Remangler.cpp",
        // Our thin FFI shim.
        "vendor/swift-demangle/src/ffi.cpp",
    ];

    for src in &sources {
        build.file(src);
    }

    build.compile("swift_demangle");

    // Re-run if any source or header changes.
    println!("cargo:rerun-if-changed=vendor/swift-demangle/");
}
