#[derive(Debug)]
struct IgnoreMacros(std::collections::HashSet<&'static str>);

impl bindgen::callbacks::ParseCallbacks for IgnoreMacros {
    fn will_parse_macro(&self, name: &str) -> bindgen::callbacks::MacroParsingBehavior {
        if self.0.contains(name) {
            bindgen::callbacks::MacroParsingBehavior::Ignore
        } else {
            bindgen::callbacks::MacroParsingBehavior::Default
        }
    }
}


#[cfg(feature = "docs-rs")]
fn main() {}

#[cfg(not(feature = "docs-rs"))]
fn main() {
    let pkg = pkg_config::probe_library("vips").expect("Could not find libvips");

    let out_path = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("bindings.rs");

    let ignored_macros = IgnoreMacros(
        vec!["FP_INFINITE", "FP_NAN", "FP_NORMAL", "FP_SUBNORMAL", "FP_ZERO"].into_iter().collect(),
    );

    let mut builder = bindgen::Builder::default();

    for path in pkg.include_paths.iter().filter_map(|p| p.to_str()) {
        builder = builder.clang_arg("-I");
        builder = builder.clang_arg(path);
    }

    let bindings = builder
        .derive_debug(true)
        .impl_debug(true)
        .default_enum_style(bindgen::EnumVariation::Rust { non_exhaustive: false })
        .header_contents("wrapper.h", "#include \"vips/vips.h\"")
        .parse_callbacks(Box::new(ignored_macros))
        .generate()
        .unwrap();

    bindings.write_to_file(out_path).unwrap();
}
