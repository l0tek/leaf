use std::{env, fs, path::PathBuf, process::Command};

fn main() {
    println!("cargo:rerun-if-changed=assets/leaf-icon.ico");
    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("fehlendes Manifest"));
    let icon = manifest_dir.join("assets/leaf-icon.ico");
    let output = PathBuf::from(env::var("OUT_DIR").expect("fehlendes Ausgabeverzeichnis"));
    let source = output.join("leaf-icon.rc");
    let resource = output.join("leaf-icon.res");
    fs::write(&source, format!("32512 ICON \"{}\"\n", icon.display()))
        .expect("Windows-Ressourcendatei konnte nicht geschrieben werden");

    let compiler = env::var_os("LLVM_RC").unwrap_or_else(|| "llvm-rc".into());
    let status = Command::new(compiler)
        .arg("/fo")
        .arg(&resource)
        .arg(&source)
        .status()
        .expect("llvm-rc konnte nicht gestartet werden; LLVM_RC auf llvm-rc setzen");
    assert!(
        status.success(),
        "Windows-App-Icon konnte nicht kompiliert werden"
    );
    println!("cargo:rustc-link-arg={}", resource.display());
}
