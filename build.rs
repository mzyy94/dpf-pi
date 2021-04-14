extern crate bindgen;
extern crate cc;
use std::env;
use std::path::PathBuf;

fn main() {
    build_binding();
}

fn build_binding() {
    cc::Build::new()
        .warnings(true)
        .flag("-Wall")
        .flag("-Wextra")
        .flag("-Wno-psabi")
        .flag("-pipe")
        .flag("-fPIC")
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
        .file("/opt/vc/src/hello_pi/libs/ilclient/ilclient.c")
        .file("/opt/vc/src/hello_pi/libs/ilclient/ilcore.c")
        .file("./wrapper.c")
        .include("/opt/vc/src/hello_pi/libs/ilclient/")
        .include("/opt/vc/include/")
        .include("/opt/vc/include/interface/vcos/pthreads")
        .include("/opt/vc/include/interface/vmcs_host/linux")
        .compile("libilclient.a");

    println!("cargo:rustc-link-search=native=/opt/vc/lib");
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

    println!("cargo:warnings=running-if-changed-bindgen");
    let bindings = bindgen::Builder::default()
        .clang_arg("-I/opt/vc/include/")
        .clang_arg("-I/opt/vc/include/interface/vcos/pthreads")
        .clang_arg("-I/opt/vc/include/interface/vmcs_host/linux")
        .clang_arg("-I/opt/vc/src/hello_pi/libs/ilclient")
        .clang_arg("-I/opt/vc/src/hello_pi/libs/vgfont")
        .clang_arg("-I/opt/vc/src/hello_pi/libs/revision")
        .clang_arg("-I/usr/lib/gcc/arm-linux-gnueabihf/8/include/")
        .clang_arg("-I/usr/include/")
        .header("/opt/vc/include/bcm_host.h")
        .header("/opt/vc/src/hello_pi/libs/ilclient/ilclient.h")
        .header("./wrapper.h")
        .generate()
        .expect("Unable to generate bindings!");

    println!("cargo:rerun-if-changed=/opt/vc/src/hello_pi/libs/ilclient/ilclient.c");
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
