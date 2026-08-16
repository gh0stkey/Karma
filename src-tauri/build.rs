fn main() {
    #[cfg(target_os = "macos")]
    {
        let dist = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("vendor/mlx-dist/lib");
        let dist = dist.canonicalize().expect("vendor/mlx-dist/lib not found");
        println!("cargo:rustc-link-arg=-Wl,-rpath,{}", dist.display());
        println!("cargo:rustc-link-arg=-Wl,-rpath,@executable_path/../Resources");
    }
    tauri_build::build()
}
