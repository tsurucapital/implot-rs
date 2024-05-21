use bindgen::{Builder, CargoCallbacks};
use std::{env, io::Write, path::PathBuf};

// All this crate does is run bindgen on cimplot and store the result
// in the src folder of the implot-sys crate. We add those bindings
// to git so people don't have to install clang just to use implot-rs.

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
        .parse_callbacks(Box::new(CargoCallbacks))
        // Reuse the imgui types that implot requires from imgui_sys so we don't define
        // our own new types.
        .raw_line("pub use imgui_sys::*;")
        .whitelist_recursively(false)
        .whitelist_function("ImPlot.*")
        .whitelist_type("ImPlot.*")
        // We do want to create bindings for the scalar typedefs
        .whitelist_type("Im[U|S][0-9]{1,2}")
        .whitelist_type("ImVector_.*")
        .whitelist_type("ImAxis")
        .whitelist_type("ImPool_.*")
        // Remove some functions that would take a variable-argument list
        .blacklist_function("ImPlot_AnnotateVVec4")
        .blacklist_function("ImPlot_AnnotateVStr")
        .blacklist_function("ImPlot_AnnotateClampedVVec4")
        .blacklist_function("ImPlot_AnnotateClampedVStr")
        .blacklist_function("ImPlot_AnnotationV")
        .blacklist_function("ImPlot_TagXV")
        .blacklist_function("ImPlot_TagYV")
        .blacklist_function("ImPlotAnnotationCollection_AppendV")
        .blacklist_function("ImPlotTagCollection_AppendV")
        // See https://github.com/rust-lang/rust-bindgen/issues/1188
        .blacklist_type("time_t")
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
        std::fs::File::create(&out_path.join("bindings.rs")).expect("Could not open bindings file");
    out_file
        .write_all(&bindings_string.into_bytes()[..])
        .expect("Couldn't write bindings");
}
