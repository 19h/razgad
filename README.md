<p align="center">
  <strong>razgad</strong><br>
  <em>A universal symbol demangler / remangler and function-name parser toolkit.</em>
</p>

<p align="center">
  <code>50 public schemes</code> &middot; <code>heuristic detection</code> &middot; <code>decode + re-encode</code> &middot; <code>signature parsing</code>
</p>

---

**razgad** is a Rust library for decoding, classifying, normalizing, and re-emitting mangled, decorated, and runtime symbol names across a wide spread of compiler, platform, and language ecosystems.

It also does something most demangling libraries do not: it exposes a reusable parser for already-demangled function names and tool-generated declaration strings. That matters in real reverse-engineering workflows, where you often need to consume both raw manglings and post-processed names from disassemblers, decompilers, crash logs, symbol servers, game engines, or metadata dumps.

The crate gives you two complementary layers:

1. A cross-scheme `Symbol` model for mangled-name decoding, heuristic detection, and re-encoding.
2. A function-name parser for extracting access modifiers, calling conventions, return types, templates, argument names, return-location annotations, and trailing qualifiers from display strings.

```rust
use razgad::{
    decode, encode, heuristic_decode, parse_function_name, PlatformDecorations, Scheme, Symbol,
    Type,
};

fn demo() -> Result<(), Box<dyn std::error::Error>> {
    let detected = heuristic_decode("__imp_?alpha@demo@@YAXH@Z")?;
    assert_eq!(detected.scheme, Scheme::CoffPe);
    assert_eq!(detected.symbol.display(), "import thunk for demo::alpha(int)");

    let itanium = decode(Scheme::ItaniumCpp, "_ZN4demo3VecIiE4pushERKi")?;
    assert_eq!(itanium.display(), "demo::Vec<int>::push(int const&)");

    let parsed = parse_function_name(
        "public: std::vector<int> __cdecl demo::Widget::run(std::map<int, std::string> const& values, char *name) const",
    )
    .unwrap();
    assert_eq!(parsed.calling_convention.as_deref(), Some("__cdecl"));
    assert_eq!(parsed.callable_name.as_deref(), Some("demo::Widget::run"));

    let fresh = Symbol::function(Scheme::Elf, ["demo", "alpha"], [])
        .with_return(Type::void())
        .with_platform(PlatformDecorations::default().with_elf_version("GLIBCXX_3.4"));

    assert_eq!(encode(Scheme::Elf, &fresh)?, "_ZN4demo5alphaEv@@GLIBCXX_3.4");
    Ok(())
}
```

---

## Why razgad exists

Most symbol tooling falls into one of three buckets:

- it understands one ABI deeply, but falls apart on mixed corpora
- it only produces a display string, throwing away structure you need for indexing or analysis
- it assumes the input is either fully mangled or fully clean, and has no answer for the messy middle

razgad exists to handle that messy middle.

It treats symbol handling as a normalization problem, not only a pretty-printing problem. Wrappers stay separate from inner grammars. Platform decorations stay orthogonal to semantic identity. Exact byte replay stays possible when the normalized model would otherwise be lossy. And already-demangled names can still be parsed into useful structure instead of being left as opaque strings.

In practice this makes the crate useful for reverse engineering, corpus analysis, binary indexing, signature databases, symbol cleanup, crash-symbol normalization, and any workflow that has to cross boundaries between compilers, languages, and tooling conventions.

---

## Public API

The public surface now has two distinct halves.

### Mangling / demangling API

| Function | Purpose |
|----------|---------|
| `decode(scheme, input)` | Decode with an explicit, caller-chosen scheme |
| `heuristic_decode(input)` | Detect likely scheme, attach confidence, then decode |
| `encode(scheme, &symbol)` | Re-emit a `Symbol` back into a scheme-specific spelling |

The reusable model is built around:

- `Scheme` - the scheme requested by the caller or selected heuristically
- `Symbol` - the normalized symbol record
- `Name`, `Type`, `Signature` - structured identity and callable type information
- `PlatformDecorations` - wrappers such as import prefixes, leading underscores, and ELF versions
- `Confidence` - certainty level for heuristic discovery

### Function-name parsing API

| Function / type | Purpose |
|-----------------|---------|
| `normalize_symbol_display()` | Normalize Rust-style escape sequences and common display artifacts |
| `parse_function_name()` | Parse C++-style scoped declarations using `::` |
| `parse_function_name_with_separator()` | Parse alternate scope conventions such as `.` |
| `parse_template_node()` | Parse a template tree from a qualified type or callable |
| `parse_template_node_with_separator()` | Same parser with custom scope separator |
| `split_scope()` / `split_scope_with_separator()` | Split qualified paths without breaking nested templates |
| `split_argument_name()` / `split_argument_name_with_separator()` | Separate type text from argument names |
| `template_depth()` | Measure nested template depth in a declaration |
| `ParsedFunctionName`, `ParsedArgument`, `TemplateNode` | Structured outputs for downstream analysis |

This parser layer is not decorative. It is now part of how the crate enriches `Plain`, dotted naming schemes, Swift displays, MSVC demangled outputs, function-pointer return styles, and receiver-like method displays.

---

## Function-name parser example

The parser is designed for already-readable declarations that still carry useful structure:

```rust
use razgad::{parse_function_name, AccessModifier};

let parsed = parse_function_name(
    "private: __int64 __usercall Foo::bar@<rax>(int a, char const *msg) const",
)
.unwrap();

assert_eq!(parsed.access, Some(AccessModifier::Private));
assert_eq!(parsed.return_type.as_deref(), Some("__int64"));
assert_eq!(parsed.calling_convention.as_deref(), Some("__usercall"));
assert_eq!(parsed.return_location.as_deref(), Some("@<rax>"));
assert_eq!(parsed.callable_path, vec!["Foo", "bar"]);
assert_eq!(parsed.arguments[0].type_text, "int");
assert_eq!(parsed.arguments[0].name.as_deref(), Some("a"));
assert_eq!(parsed.trailing_qualifiers.as_deref(), Some("const"));
```

It also supports non-C++ scope separators for ecosystems that prefer dotted names:

```rust
use razgad::parse_function_name_with_separator;

let parsed = parse_function_name_with_separator(
    "Swift.Int Demo.Widget.run(Swift.String name, Swift.Bool)",
    ".",
)
.unwrap();

assert_eq!(parsed.callable_path, vec!["Demo", "Widget", "run"]);
```

It also handles function-pointer return styles such as `void (__cdecl *demo::signal(int))(char const *)`, pointer-to-member declarator forms such as `int (demo::Widget::*demo::Factory::slot())`, and avoids mistaking Go receiver forms like `main.(*T).Method` for signatures.

---

## The normalized symbol model

The core idea is a scheme-neutral `Symbol` tree:

```text
Symbol
|- scheme
|- concrete_family
|- kind
|- path
|- signature
|- special
|- platform
`- verbatim
```

This split matters.

- `scheme` records the route the caller cares about: `MachO`, `CoffPe`, `Elf`, `IntelNativeCpp`, and so on.
- `concrete_family` records the inner grammar actually doing the work: for example a `MachO` symbol may still be an Itanium C++ symbol under the wrapper.
- `kind` separates normal functions from methods, constructors, destructors, vtables, thunks, metadata, imports, module initializers, type encodings, closures, and runtime artifacts.
- `platform` keeps transport details out of semantic identity: leading underscores, import thunk prefixes, inner scheme hints, ELF versions.
- `verbatim` preserves byte-for-byte replay safety for decoded inputs.

This gives you a model that is useful for programmatic analysis while still remaining practical for exact round-tripping.

---

## Supported schemes

`Scheme::all_public()` currently exposes **50 public schemes**.

| Group | Schemes |
|------|---------|
| Core ABIs and mainstream languages | `ItaniumCpp`, `MicrosoftCpp`, `Dlang`, `RustLegacy`, `RustV0`, `Swift`, `ObjectiveC`, `Jni` |
| Legacy and vendor C++ families | `BorlandCpp`, `WatcomCpp`, `DigitalMars`, `IbmXlCppLegacy`, `HpAccCppLegacy`, `SunStudioCppLegacy`, `CfrontCpp`, `ArmCppLegacy`, `GreenHillsCpp`, `IntelNativeCpp`, `EdgCppLegacy`, `CrayCpp`, `SgiMipsproCpp`, `MetrowerksCpp`, `Os400Cpp`, `Vms`, `CarbonCpp` |
| Calling conventions and binary wrappers | `Cdecl`, `Stdcall`, `Fastcall`, `Vectorcall`, `MachO`, `CoffPe`, `Elf` |
| Naming and runtime ecosystems | `Pascal`, `FortranExternal`, `DotNet`, `Haskell`, `AdaGnat`, `GfortranModule`, `Ocaml`, `Go`, `Zig`, `Nim`, `PascalDelphi`, `Modula`, `Crystal`, `Vlang`, `WebAssembly`, `Plain`, `UnityIl2Cpp`, `MonoManaged` |

Some important subtleties:

- `IntelNativeCpp` is treated as a target-dependent family that can resolve to MSVC or Itanium.
- `MachO`, `CoffPe`, and `Elf` are wrappers, not standalone inner grammars.
- Several historical schemes are intentionally modeled as stable naming conventions rather than full ABI-rich type systems.
- Dotted naming families such as Ada, Modula, Pascal/Delphi, Go receiver forms, and parts of Swift / managed-name handling now benefit from the shared declaration parser instead of ad hoc path splitting alone.

---

## Round-tripping philosophy

razgad is deliberately **normalized first** and **lossless by escape hatch**.

When you decode a symbol, the original text is preserved in `Symbol::verbatim`. That means `encode()` can replay the exact original bytes even when the normalized model does not fully describe every vendor-specific token.

This is a deliberate tradeoff:

- callers get a usable cross-scheme AST
- obscure vendor spellings still survive round-trips intact
- canonical fresh construction stays honest instead of faking precision it does not really have

Fresh canonical encoding is currently implemented for a focused subset, with especially solid coverage for:

- Itanium-family construction
- Windows C decoration families (`cdecl`, `stdcall`, `fastcall`, `vectorcall`)
- D, JNI, Ada GNAT, gfortran modules, Fortran externals, and V names
- platform wrappers such as Mach-O, COFF import thunks, and ELF versioned symbols
- plain, Unity IL2CPP, and Mono-style managed naming forms

The canonical encoder surface is intentionally narrower than the decoder surface. The crate is conservative about what it claims to synthesize from structured data.

---

## Detection strategy

`heuristic_decode()` runs ordered sniffers and returns both the chosen `Scheme` and a `Confidence` value.

Examples of strong signals:

- `_R` / `__R` -> Rust v0
- `_ZN...17h...E` / `__ZN...17h...E` -> Rust legacy
- `_Z...`, `__Z...`, `_ZTV...` -> Itanium-family
- `?name@@...` -> MSVC-family
- `Java_...` -> JNI
- `_OBJC_...`, `-[...]`, `+[...]`, `v@:` -> Objective-C forms
- `__imp_...` -> COFF import thunk wrapper
- `...@@GLIBCXX_...` -> ELF versioned wrapper
- Unity IL2CPP and Mono-managed forms are recognized before generic naming fallbacks

For genuinely ambiguous forms the API returns `Medium` or `Low` confidence rather than pretending certainty.

---

## Architecture at a glance

Internally the crate is organized around a few clear layers:

- `src/schemes/` contains per-family decoders plus wrapper handling for Mach-O / COFF / ELF.
- `src/heuristics.rs` handles scheme discovery and confidence assignment.
- `src/model.rs` defines the shared, scheme-neutral symbol representation.
- `src/codec.rs` handles canonical encoding and exact verbatim replay.
- `src/function_names.rs` parses already-readable declarations, templates, arguments, calling conventions, and return-location annotations.
- `src/text.rs` is the bridge layer that projects demangled or parsed text back into `Name`, `Type`, `Signature`, and `Symbol` structures.

One of the more important recent architectural shifts is that the generic function-name parser is no longer just a side utility. It now participates directly in:

- `Plain` symbol decoding
- dotted naming families in `src/schemes/naming.rs`
- Swift demangled-display enrichment
- MSVC demangled-display enrichment
- function-pointer and pointer-to-member declaration projection through the shared parser path

That keeps the crate from having four separate half-parsers for the same declaration features.

For high-confidence families, razgad leans on battle-tested ecosystem crates where that makes sense:

- `cpp_demangle` for Itanium-family parsing
- `msvc-demangler` for Microsoft C++
- `rustc-demangle` for Rust forms
- an in-tree pure-Rust Swift demangler derived from Swift's demangling sources

The important part is what happens after that: vendor-specific outputs are normalized into one common model instead of being left as unrelated display strings.

---

## Validation

The test suite is deliberately behavior-first.

- `tests/exhaustive.rs` contains **102 fixture cases** spanning every public scheme in `Scheme::all_public()`
- fixture tests assert explicit decode, heuristic detection, and decode-then-encode round-trips
- `tests/function_names.rs` exercises declaration parsing, nested templates, Rust display normalization, alternate scope separators, function-pointer and pointer-to-member declarators, Go receiver displays, and plain-scheme enrichment
- `tests/model.rs` checks that templates, wrappers, metadata, runtime artifacts, dotted naming schemes, Go receiver methods, Objective-C runtime wrappers, Swift, and MSVC all project correctly into the same normalized tree
- `cargo test` currently passes with **33 total tests** in this repository

Run the suite with:

```bash
cargo test
```

There is also a corpus utility for bulk validation against large symbol lists:

```bash
cargo run --bin corpus_check -- path/to/function_names.txt --dump-failures failures.tsv
```

That tool reports coverage, scheme distribution, sample failures, and can emit a TSV dump of undecoded symbols for follow-up work.

---

## Building and using it

### Build

```bash
cargo build
```

### Test

```bash
cargo test
```

### Use as a dependency

```toml
[dependencies]
razgad = { path = "../razgad" }
```

Then:

```rust
use razgad::{decode, heuristic_decode, parse_function_name, Scheme};

let symbol = decode(Scheme::Swift, "_$s4Demo5alphayyF")?;
assert_eq!(symbol.display(), "Demo.alpha()");

let detected = heuristic_decode("Java_p_q_r_A_f__ILjava_lang_String_2")?;
assert_eq!(detected.scheme, Scheme::Jni);

let parsed = parse_function_name("private: int __fastcall demo::Widget::run(std::string const& name)")
    .unwrap();
assert_eq!(parsed.callable_name.as_deref(), Some("demo::Widget::run"));
```

---

## Current shape of the project

Today, razgad is already good at a very specific kind of work:

- decoding a broad range of mangled and decorated symbol forms through one API
- preserving wrapper semantics instead of flattening everything into one string
- giving callers a normalized symbol representation they can inspect and transform
- parsing human-readable declaration strings into structured parts
- round-tripping decoded inputs safely
- expanding coverage through fixture-driven and corpus-driven validation

It is not pretending to be a perfect canonical encoder for every ABI on day one. The implementation is intentionally incremental: broad decode coverage first, faithful normalization second, shared declaration parsing across schemes, and canonical fresh encoding where it can be done honestly.

That bias is what makes the crate useful in real reverse-engineering and binary-analysis workflows instead of only in toy examples.
