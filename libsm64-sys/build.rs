use std::fs;
use std::path::PathBuf;
use std::process::Command;

const SRC_DIRS: &[&str] = &[
    "libsm64/src",
    "libsm64/src/decomp",
    "libsm64/src/decomp/engine",
    "libsm64/src/decomp/include/PR",
    "libsm64/src/decomp/game",
    "libsm64/src/decomp/pc",
    "libsm64/src/decomp/mario",
    "libsm64/src/decomp/tools",
];

const MARIO_GEO: &str = "libsm64/src/decomp/mario/geo.inc.c";

fn main() {
    if !PathBuf::from(MARIO_GEO).exists() {
        Command::new("python3")
            .arg("import-mario-geo.py")
            .current_dir("libsm64")
            .output()
            .expect("Unable to download mario geometry");
    }

    let mut c_files: Vec<String> = Vec::new();
    // ugly code but it works
    for dir in SRC_DIRS {
        let mut files: Vec<String> = fs::read_dir(dir)
            .expect(dir)
            .filter_map(|entry| {
                let path = entry.ok()?.path();
                if !path.is_file() {
                    return None;
                }
                if path.extension().unwrap().to_str() != Some("c") {
                    return None;
                }
                path.to_str().map(|s| s.to_owned())
            })
            .collect();
        c_files.append(&mut files);
    }

    cc::Build::new()
        .files(c_files)
        .warnings(false)
        .compiler("emcc")
        .flag("-fno-strict-aliasing")
        .flag("-fPIC")
        .flag("-g")
        .define("SM64_LIB_EXPORT", None)
        .define("GBI_FLOATS", None)
        .define("VERSION_US", None)
        .include("libsm64/src/decomp/include")
        .target("wasm32-unknown-emscripten")
        .compile("sm64");

    let bindings = bindgen::builder()
        .header("libsm64/src/libsm64.h")
        .clang_arg("-fvisibility=default")
        .generate_inline_functions(true)
        .generate()
        .expect("Unable to generate libsm64 bindings");

    let out_dir = std::env::var("OUT_DIR").unwrap();
    let out_path = PathBuf::from(&out_dir);
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Could not write C bindings");
}
