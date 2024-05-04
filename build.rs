use std::path::PathBuf;

use bindgen::EnumVariation;

fn main() {
    let binding_path = PathBuf::from("binding/binding.rs");
    if !binding_path.exists() {
        let bindings = bindgen::builder()
            .header("fpops.h")
            .default_enum_style(EnumVariation::ModuleConsts)
            .use_core()
            .ignore_methods()
            .merge_extern_blocks(true)
            .generate()
            .expect("failed to generate rust bindings");
        bindings
            .write_to_file(&binding_path)
            .expect("failed to write binding to file");
    }
    println!("cargo:rustc-link-lib=binding");
}
