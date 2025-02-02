use std::fs;
use wat::parse_file;

// This script is used to generate .wasm files from .wat files for benchmarks/tests and to build
// the plugins into .wasm files.
//
// It will write all generated .wasm files into the `./target/wasm` directory.
fn main() {
    const WAT_DIR: &str = "wat";
    const PLUGIN_DIR: &str = "plugins/";
    const TARGET_DIR: &str = "target/wasm/";

    // Re-run if any file in the `wat` or `plugins` directory changes
    println!("cargo:rerun-if-changed={}", WAT_DIR);
    println!("cargo:rerun-if-changed={}", PLUGIN_DIR);

    // Create output directory if it doesn't exist
    fs::create_dir_all(TARGET_DIR).unwrap_or_else(|_| panic!("Create {} dir", TARGET_DIR));

    // Scan `wat` directory for .wat files and build corresponding .wasm files
    for wat_file in fs::read_dir(WAT_DIR).unwrap_or_else(|_| panic!("Read {}", WAT_DIR)) {
        let wat_file = wat_file.unwrap();
        let wasm =
            parse_file(wat_file.path()).unwrap_or_else(|_| panic!("Parsing {:?}", wat_file.path()));
        // Change extension to .wasm
        let wasm_filename = wat_file.path().with_extension("wasm");
        // Get only the filename part of the `Path`
        let wasm_filename = wasm_filename.file_name().unwrap().to_str().unwrap();
        let wasm_file = format!("{}{}", TARGET_DIR, wasm_filename);
        fs::write(&wasm_file, wasm).unwrap_or_else(|_| panic!("Writing {}", wasm_file));
    }
}
