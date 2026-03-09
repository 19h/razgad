# symdem architecture

This crate is designed around three constraints:

1. The input domain mixes true grammars (Itanium, MSVC, Rust v0, D) with naming conventions and wrappers (Mach-O, COFF imports, ELF symbol versions, Objective-C runtime metadata).
2. Some vendor schemes are fully specified, while others are historical and drift across compiler versions.
3. A production demangler still needs deterministic round-tripping even when the normalized model cannot perfectly reconstruct every vendor-specific token.

The resulting design is intentionally *normalized first* and *lossless by escape hatch*.

## Public API shape

The crate exposes three primary entry points:

- `decode(scheme, input)` for explicit per-scheme decoding.
- `encode(scheme, symbol)` for explicit per-scheme encoding.
- `heuristic_decode(input)` for scheme discovery plus decoding when the caller does not know the ABI.

Every scheme also has a dedicated module-level codec facade, but the generic API remains the canonical surface.

## Universal symbol model

The core representation is a scheme-neutral `Symbol` tree.

```text
Symbol
|- scheme: Scheme
|- concrete_family: SchemeFamily
|- language: Language
|- kind: SymbolKind
|- path: Vec<Name>
|- signature: Option<Signature>
|- special: Option<SpecialKind>
|- platform: PlatformDecorations
|- vendor: VendorMetadata
|- verbatim: Option<String>
```

### Why this shape

- `scheme` records the public API route requested by the caller, including aliases and wrappers such as `IntelNativeCpp`, `MachO`, `CoffPe`, and `Elf`.
- `concrete_family` records the actual mangling grammar used after wrappers are peeled. For example, `MachO` may decode to the `ItaniumCpp` concrete family, while `IntelNativeCpp` may resolve to `ItaniumCpp` or `MicrosoftCpp`.
- `kind` separates functions, data, metadata, thunks, tables, constructors, destructors, module initializers, elaboration symbols, and opaque runtime artifacts.
- `path` captures qualified identity in a reusable way across C++, Rust, D, Swift, OCaml, Ada, Go, JNI, and several legacy schemes.
- `signature` carries calling convention, parameters, return type, qualifiers, variadic state, and method-ness.
- `platform` stores orthogonal wrappers such as leading underscores, import thunk prefixes, stdcall byte counts, and ELF versions.
- `vendor` stores details that do not fit the common tree cleanly, such as MSVC access bits, Swift metadata flavors, or legacy descriptor payloads.
- `verbatim` guarantees round-trip safety for any decoded input even if the normalized model does not capture every byte semantically.

## Names and types

`Name` is expressive enough for both regular identifiers and compiler-generated path pieces:

- plain identifiers
- template / generic applications
- operators
- special compiler names
- anonymous / synthetic names

`Type` is intentionally broad rather than ABI-specific:

- primitives
- named types by path
- pointers, references, rvalue references
- arrays and function types
- tuples / generic placeholders
- Objective-C runtime pseudo-types (`id`, `SEL`)
- opaque vendor payloads when a scheme embeds something not worth normalizing further

This is sufficient for tested coverage across Itanium templates, MSVC methods, Rust crates, D signatures, JNI signatures, Objective-C encodings, and legacy Cfront-style schemes.

## Alias and wrapper handling

Several user-listed schemes are aliases, wrappers, or target-dependent families rather than standalone grammars.

- `IntelNativeCpp` delegates to Itanium or MSVC based on prefix or target hint.
- `CarbonCpp` delegates to Itanium.
- `CrayCpp` delegates to Itanium for modern cases and preserves legacy payloads via `verbatim`.
- `EdgCppLegacy` uses the shared Cfront-style decoder for old-style manglings.
- `MachO`, `CoffPe`, and `Elf` wrap an inner grammar and decorate the decoded symbol via `PlatformDecorations`.
- `WebAssembly` is modeled as a name transport convention rather than a type-rich mangling grammar.

This decomposition keeps the data model honest: wrappers stay wrappers, and true mangling grammars remain distinct.

## Decoding strategy

Decoding proceeds in layers:

1. wrapper peel (`MachO`, `CoffPe`, `Elf`, VMS Itanium wrapper)
2. family-specific parser (Itanium, MSVC, Cfront-style, Rust, D, Swift, etc.)
3. normalization into `Symbol`
4. preservation of the original raw input in `verbatim`

The parser is deliberately split into high-confidence families and pattern-oriented legacy families.

- High-confidence families: Itanium, MSVC, D, Rust legacy, Rust v0, Swift, JNI, C decorations, Objective-C runtime forms.
- Pattern legacy families: Borland, Watcom, xlC legacy, HP aCC legacy, SunPro legacy, Cfront-derived families, OCaml, GNAT, Haskell Z-encoding, Pascal/Delphi, V, Nim runtime symbols, and other historically stable naming conventions.

## Encoding strategy

Encoding uses two modes:

1. canonical encoding from normalized fields when the scheme encoder supports it.
2. exact replay from `verbatim` when a decoded symbol is being re-emitted and the normalized tree does not contain enough scheme-specific detail to guarantee byte-for-byte reconstruction.

This is a deliberate design choice. It gives callers a usable normalized AST while still supporting lossless round-trips for obscure vendor grammars.

## Heuristic decoding

`heuristic_decode` applies ordered sniffers with confidence scores.

Examples:

- `_R` or `__R` -> Rust v0
- `_ZN...17h...E` / `__ZN...17h...E` -> Rust legacy
- `_Z`, `__Z`, `_ZTV`, `_ZTI`, `_ZTS`, `_ZGV`, `_ZTT`, `_ZTC` -> Itanium family
- `?` / `??_` -> MSVC family unless the string matches a known Digital Mars pattern
- `@name@N`, `_name@N`, `name@@N` -> Windows C decoration family
- `$s`, `_$s`, `$S`, `_T` -> Swift
- `_D` -> D
- `Java_` -> JNI
- `_OBJC_`, `OBJC_`, `-[`, `+[`, `v@:` -> Objective-C forms
- `caml` -> OCaml
- `__mod_MOD_` -> gfortran module naming
- `ada__`, trailing `_E` -> GNAT conventions

For ambiguous cases the API returns `Confidence::Medium` or `Confidence::Low` rather than pretending certainty.

## Testing plan

The test suite is intentionally front-loaded.

- Fixture-based decode and round-trip tests cover every public scheme.
- Model tests assert that the normalized `Symbol` tree is expressive enough for templates, metadata, wrappers, and runtime conventions.
- Heuristic tests validate both detected scheme and confidence.
- Canonical encode tests use symbols built from the normalized AST rather than only replaying `verbatim` fixtures.

The implementation is written only after these expectations are pinned down.
