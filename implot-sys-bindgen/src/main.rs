use bindgen::{callbacks::EnumVariantValue, Builder};
use std::{env, io::Write, path::PathBuf};

// All this crate does is run bindgen on cimplot and store the result
// in the src folder of the implot-sys crate. We add those bindings
// to git so people don't have to install clang just to use implot-rs.

#[derive(Debug)]
struct Callbacks;

fn snake_case(name: &str) -> String {
    // Take care of exceptions
    let name = name.replace("NaN", "Nan");

    // SNAKE_CASE
    let mut s = String::new();
    for (i, c) in name.chars().enumerate() {
        if c.is_uppercase() {
            // characters without capitalization are considered lowercase
            if i != 0 {
                s.push('_');
            }
            s.push(c);
        } else {
            s.push(c.to_ascii_uppercase());
        }
    }

    s
}

impl bindgen::callbacks::ParseCallbacks for Callbacks {
    fn enum_variant_name(
        &self,
        enum_name: Option<&str>,
        original_variant_name: &str,
        _variant_value: EnumVariantValue,
    ) -> Option<String> {
        let enum_name = enum_name?;
        if enum_name.starts_with("ImPlot") || enum_name == "ImAxis_" {
            let name = original_variant_name.split('_').last().unwrap();

            if enum_name.ends_with("Flags_") {
                // Assume bitfield
                Some(snake_case(name))
            } else {
                // Assume regular enum
                Some(name.to_string())
            }
        } else {
            None
        }
    }
}

fn main() {
    let cwd = env::current_dir().expect("Could not read current directory");
    let sys_crate_path = cwd
        .join("..")
        .join("implot-sys")
        .canonicalize()
        .expect("Could not find sys crate directory");

    let cimgui_include_path = PathBuf::from(
        env::var_os("DEP_IMGUI_THIRD_PARTY").expect("DEP_IMGUI_THIRD_PARTY not defined"),
    );

    let bindings = Builder::default()
        .clang_arg("-DCIMGUI_DEFINE_ENUMS_AND_STRUCTS=1")
        .clang_arg(format!("-I{}", cimgui_include_path.display()))
        .header(
            sys_crate_path
                .join("third-party")
                .join("cimplot")
                .join("cimplot.h")
                .to_str()
                .expect("Could not turn cimplot.h path into string"),
        )
        .parse_callbacks(Box::new(Callbacks))
        // Reuse the imgui types that implot requires from imgui_sys so we don't define
        // our own new types.
        .raw_line("pub use imgui_sys::*;")
        .allowlist_recursively(false)
        .allowlist_function("ImPlot.*")
        .allowlist_type("ImPlot.*")
        // We do want to create bindings for the scalar typedefs
        .allowlist_type("Im[U|S][0-9]{1,2}")
        .allowlist_type("ImVector_.*")
        .allowlist_type("ImAxis.*")
        .allowlist_type("ImPool_.*")
        // Remove some functions that would take a variable-argument list
        .blocklist_function("ImPlot_AnnotateVVec4")
        .blocklist_function("ImPlot_AnnotateVStr")
        .blocklist_function("ImPlot_AnnotateClampedVVec4")
        .blocklist_function("ImPlot_AnnotateClampedVStr")
        .blocklist_function("ImPlot_AnnotationV")
        .blocklist_function("ImPlot_TagXV")
        .blocklist_function("ImPlot_TagYV")
        .blocklist_function("ImPlotAnnotationCollection_AppendV")
        .blocklist_function("ImPlotTagCollection_AppendV")
        .bitfield_enum("ImPlot([a-zA-Z]*)Flags_")
        .default_enum_style(bindgen::EnumVariation::Rust {
            non_exhaustive: false,
        })
        .rustified_enum("ImPlotCol_")
        // See https://github.com/rust-lang/rust-bindgen/issues/1188
        .blocklist_type("time_t")
        .raw_line("pub type time_t = libc::time_t;")
        .raw_line("pub type tm = libc::tm;")
        .generate()
        .expect("Unable to generate bindings");

    // The above type re-export shenanigans make bindgen unable to derive Copy, Clone and Debug on
    // some types, but they would work - we hence manually re-add them here.
    let mut bindings_string = bindings.to_string();
    ["ImPlotInputMap", "ImPlotStyle"].iter().for_each(|name| {
        bindings_string = bindings_string.replace(
            &format!("pub struct {}", name),
            &format!("#[derive(Clone, Copy, Debug)]\npub struct {}", name),
        );
    });

    // Finally we write the bindings to a file.
    let out_path = sys_crate_path.join("src");
    let mut out_file =
        std::fs::File::create(out_path.join("bindings.rs")).expect("Could not open bindings file");
    out_file
        .write_all(&bindings_string.into_bytes()[..])
        .expect("Couldn't write bindings");
}
