extern crate cmake;

use bindgen::RustTarget;
use cmake::Config;
use std::{env, path::PathBuf};

fn mlx_dist_dir() -> PathBuf {
    if let Ok(dir) = env::var("MLX_DIST_DIR") {
        return PathBuf::from(dir);
    }
    PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("../mlx-dist")
}

fn build_and_link_mlx_c() {
    let dist = mlx_dist_dir();
    let dist = dist
        .canonicalize()
        .unwrap_or_else(|e| panic!("MLX dist dir {} not found: {e}", dist.display()));

    let mut config = Config::new("src/mlx-c");
    config.very_verbose(true);
    config.define("CMAKE_INSTALL_PREFIX", ".");

    #[cfg(debug_assertions)]
    {
        config.define("CMAKE_BUILD_TYPE", "Debug");
    }

    #[cfg(not(debug_assertions))]
    {
        config.define("CMAKE_BUILD_TYPE", "Release");
    }

    config.define("MLX_C_USE_SYSTEM_MLX", "ON");
    config.define("MLX_C_BUILD_EXAMPLES", "OFF");
    config.define("MLX_DIR", dist.join("share/cmake/MLX"));

    let dst = config.build();

    println!("cargo:rustc-link-search=native={}/build/lib", dst.display());
    println!("cargo:rustc-link-search=native={}/lib", dist.display());
    println!("cargo:rustc-link-lib=static=mlxc");
    println!("cargo:rustc-link-lib=dylib=mlx");

    println!("cargo:rustc-link-lib=c++");
    println!("cargo:rustc-link-lib=dylib=objc");
    println!("cargo:rustc-link-lib=framework=Foundation");

    #[cfg(feature = "metal")]
    {
        println!("cargo:rustc-link-lib=framework=Metal");
        println!("cargo:rustc-link-lib=framework=QuartzCore");
    }

    #[cfg(feature = "accelerate")]
    {
        println!("cargo:rustc-link-lib=framework=Accelerate");
    }

}

fn main() {
    println!("cargo:rerun-if-env-changed=MLX_DIST_DIR");

    build_and_link_mlx_c();

    let bindings = bindgen::Builder::default()
        .rust_target(RustTarget::Stable_1_73)
        .header("src/mlx-c/mlx/c/mlx.h")
        .header("src/mlx-c/mlx/c/linalg.h")
        .header("src/mlx-c/mlx/c/error.h")
        .header("src/mlx-c/mlx/c/transforms_impl.h")
        .clang_arg("-Isrc/mlx-c")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
