extern crate bindgen;
extern crate cc;
use std::env;
use std::path::PathBuf;

fn main() {
    let vc_root = env::var("VC_ROOT").unwrap_or("/opt/vc".to_string());
    println!("cargo:rerun-if-changed=build.rs");
    println!(
        "cargo:rerun-if-changed={}/src/hello_pi/libs/ilclient/ilclient.c",
        vc_root
    );
    println!("cargo:rerun-if-changed=wrapper.h");
    println!("cargo:rerun-if-changed=wrapper.c");
    build_binding();
}

fn build_binding() {
    let vc_root = env::var("VC_ROOT").unwrap_or("/opt/vc".to_string());
    let flag_sysroot = env::var("SYSROOT").map_or("-invalid-argument".to_string(), |sysroot| {
        format!("--sysroot={}", sysroot)
    });

    cc::Build::new()
        .warnings(false)
        .flag("-Wall")
        .flag("-Wextra")
        .flag("-Wno-psabi")
        .flag("-Wno-unused-parameter")
        .flag("-Wno-sign-compare")
        .flag("-pipe")
        .flag("-fPIC")
        .flag_if_supported(flag_sysroot.as_str())
        .define("STANDALONE", None)
        .define("__STDC_CONSTANT_MACROS", None)
        .define("__STDC_LIMIT_MACROS", None)
        .define("TARGET_POSIX", None)
        .define("_LINUX ", None)
        .define("PIC", None)
        .define("_REENTRANT", None)
        .define("_LARGEFILE64_SOURCE", None)
        .define("_FILE_OFFSET_BITS", "64")
        .define("HAVE_LIBOPENMAX", "2")
        .define("OMX", None)
        .define("OMX_SKIP64BIT", None)
        .define("USE_EXTERNAL_OMX", None)
        .define("HAVE_LIBBCM_HOST", None)
        .define("USE_EXTERNAL_LIBBCM_HOST", None)
        .define("USE_VCHIQ_ARM", None)
        .file(format!("{}/src/hello_pi/libs/ilclient/ilclient.c", vc_root))
        .file(format!("{}/src/hello_pi/libs/ilclient/ilcore.c", vc_root))
        .file("./wrapper.c")
        .include(format!("{}/src/hello_pi/libs/ilclient/", vc_root))
        .include(format!("{}/include/", vc_root))
        .include(format!("{}/include/interface/vcos/pthreads", vc_root))
        .include(format!("{}/include/interface/vmcs_host/linux", vc_root))
        .compile("libilclient.a");

    println!("cargo:rustc-link-search=native={}/lib", vc_root);
    println!(
        "cargo:rustc-link-search=native={}",
        env::var("OUT_DIR").unwrap()
    );

    println!("cargo:rustc-cdylib-link-arg=-Wl,--copy-dt-needed-entries");
    println!("cargo:rustc-link-lib=brcmGLESv2");
    println!("cargo:rustc-link-lib=brcmEGL");
    println!("cargo:rustc-link-lib=vcos");
    println!("cargo:rustc-link-lib=bcm_host");
    println!("cargo:rustc-link-lib=openmaxil");
    println!("cargo:rustc-link-lib=vchiq_arm");
    println!("cargo:rustc-link-lib=pthread");
    println!("cargo:rustc-link-lib=rt");
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    let mut builder = bindgen::Builder::default()
        .clang_arg(format!("-I{}/include/", vc_root))
        .clang_arg(format!("-I{}/include/interface/vcos/pthreads", vc_root))
        .clang_arg(format!("-I{}/include/interface/vmcs_host/linux", vc_root))
        .clang_arg(format!("-I{}/src/hello_pi/libs/ilclient", vc_root))
        .clang_arg(format!("-I{}/src/hello_pi/libs/vgfont", vc_root))
        .clang_arg(format!("-I{}/src/hello_pi/libs/revision", vc_root))
        .header(format!("{}/include/bcm_host.h", vc_root))
        .header(format!("{}/src/hello_pi/libs/ilclient/ilclient.h", vc_root))
        .header("./wrapper.h");

    if let Ok(sysroot) = env::var("SYSROOT") {
        builder = builder.clang_arg(format!("--sysroot={}", sysroot));
    }

    let bindings = builder.generate().expect("Unable to generate bindings!");

    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
