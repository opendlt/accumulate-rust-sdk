// Embeds an `asInvoker` application manifest into this crate's example and binary
// targets on Windows (MSVC).
//
// Without it, Windows' Installer Detection heuristic forces UAC elevation on any
// executable whose name contains "update"/"setup"/"install" — e.g.
// `cargo run --example example_10_threshold_updates` produces
// example_10_threshold_updates.exe, which then fails to launch unelevated
// (os error 740, "The requested operation requires elevation").
//
// `embed-manifest` only targets `bins`, so we emit the linker args directly for
// both `examples` and `bins`. These `rustc-link-arg-{examples,bins}` directives are
// package-local: when this crate is consumed as a library dependency, its examples
// are not built, so dependents (e.g. the codegen test harness) are unaffected.

use std::{env, fs, path::PathBuf};

const ASINVOKER_MANIFEST: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
  <trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
    <security>
      <requestedPrivileges>
        <requestedExecutionLevel level="asInvoker" uiAccess="false"/>
      </requestedPrivileges>
    </security>
  </trustInfo>
</assembly>
"#;

// `[lints.clippy] expect_used = "deny"` in Cargo.toml applies to every target,
// including this build script. Returning a Result and using `?` satisfies it
// without suppressing the lint: a failure here aborts the build with the error
// message, which is exactly the desired behaviour for a build script.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=build.rs");

    // Only Windows MSVC needs (and supports, via link.exe) this manifest embedding.
    let is_windows = env::var_os("CARGO_CFG_WINDOWS").is_some();
    let is_msvc = env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default() == "msvc";
    if !is_windows || !is_msvc {
        return Ok(());
    }

    let out_dir = env::var("OUT_DIR")?;
    let manifest_path = PathBuf::from(&out_dir).join("accumulate-asinvoker.manifest");
    fs::write(&manifest_path, ASINVOKER_MANIFEST)?;
    let input = manifest_path.display();

    // Plain `rustc-link-arg` applies to all binary outputs (examples, bins, tests,
    // benches) of this package, and is package-local (does not affect dependents).
    println!("cargo:rustc-link-arg=/MANIFEST:EMBED");
    println!("cargo:rustc-link-arg=/MANIFESTINPUT:{input}");
    println!("cargo:rustc-link-arg=/MANIFESTUAC:NO");

    Ok(())
}
