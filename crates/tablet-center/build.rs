fn main() {
    // The UI is embedded at build time; a directory path makes Cargo rebuild
    // when any file in it changes or is added, not only the ones it knew.
    println!("cargo:rerun-if-changed=../../ui");
    tauri_build::build()
}
