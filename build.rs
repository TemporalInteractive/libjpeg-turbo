use anyhow::{anyhow, Context, Result};
use std::{collections::HashMap, env, path::PathBuf, process::Command};

#[derive(Debug)]
struct Library {
    include_paths: Vec<PathBuf>,
    defines: HashMap<String, Option<String>>,
}

/// Check if nasm is installed on the users system.
fn check_nasm() {
    if !Command::new("nasm")
        .arg("-v")
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
    {
        println!("cargo:warning=NASM does not seem to be installed, so turbojpeg will be compiled without \
            SIMD extensions.");
    }
}

fn compile() -> Result<Library> {
    // Check nasm when using simd
    if !cfg!(feature = "simd") {
        check_nasm();
    }

    // Use gcc compiler
    std::env::set_var("CC", "C:\\mingw64\\bin\\gcc");

    let source_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?)
        .join("ffi")
        .join("libjpeg-turbo");

    let mut cmake = cmake::Config::new(source_path);
    cmake.configure_arg("-DENABLE_SHARED=1");
    cmake.configure_arg("-DENABLE_STATIC=0");
    cmake.define("CMAKE_INSTALL_DEFAULT_LIBDIR", "lib");
    if cfg!(feature = "simd") {
        cmake.configure_arg("-DREQUIRE_SIMD=ON");
    }

    let dst_path = cmake.build();

    let lib_path = dst_path.join("lib");
    let include_path = dst_path.join("include");

    println!("cargo:rustc-link-search=native={}", lib_path.display());
    println!("cargo:rustc-link-lib=dylib=turbojpeg");

    Ok(Library {
        include_paths: vec![include_path],
        defines: HashMap::new(),
    })
}

fn generate_bindings(lib: &Library) -> Result<()> {
    let target = env::var("TARGET").unwrap();
    let mut builder = bindgen::Builder::default()
        .header("ffi/wrapper.h")
        .use_core()
        .ctypes_prefix("libc")
        .clang_args(&["-target", &target]);

    for path in lib.include_paths.iter() {
        let path = path.to_str().unwrap();
        builder = builder.clang_arg(format!("-I{}", path));
        println!("cargo:rerun-if-changed={}", path);
    }

    for (name, value) in lib.defines.iter() {
        if let Some(value) = value {
            builder = builder.clang_arg(format!("-D{}={}", name, value));
        } else {
            builder = builder.clang_arg(format!("-D{}", name));
        }
    }

    let bindings = builder
        .generate()
        .map_err(|_| anyhow!("could not generate bindings"))?;

    let out_file = PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("bindings.rs");
    bindings
        .write_to_file(&out_file)
        .context("could not write bindings to OUT_DIR")?;
    println!("Generated bindings are stored in {}", out_file.display());

    Ok(())
}

fn main() -> Result<()> {
    println!("cargo:rerun-if-changed=build.rs");

    let lib = compile()?;
    generate_bindings(&lib)
}
