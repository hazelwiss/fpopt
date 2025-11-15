use std::path::PathBuf;

use bindgen::EnumVariation;

fn main() {
    #[cfg(target_arch = "loongarch64")]
    let binding_path = PathBuf::from("binding/binding_loongarch64.rs");
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    let binding_path = PathBuf::from("binding/binding_x86.rs");
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
}
