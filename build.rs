use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn find_blueprint_compiler() -> Option<PathBuf> {
    // 1. Check if blueprint-compiler is in PATH
    if let Ok(output) = Command::new("blueprint-compiler").arg("--version").output() {
        if output.status.success() {
            return Some(PathBuf::from("blueprint-compiler"));
        }
    }

    // 2. Check ~/.local/bin/blueprint-compiler
    if let Ok(home) = env::var("HOME") {
        let local_bin = PathBuf::from(&home).join(".local/bin/blueprint-compiler");
        if local_bin.exists() {
            return Some(local_bin);
        }

        // 3. Check python fallback with cloned repository if present
        let local_src =
            PathBuf::from(&home).join(".local/src/blueprint-compiler/blueprint-compiler.py");
        if local_src.exists() {
            return Some(local_src);
        }
    }

    None
}

fn compile_blueprints(ui_dir: &Path) {
    let compiler_opt = find_blueprint_compiler();

    let entries = fs::read_dir(ui_dir).expect("Failed to read UI directory for blueprint files");
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) == Some("blp") {
            let output_ui = path.with_extension("ui");
            println!("cargo:rerun-if-changed={}", path.display());

            if let Some(ref compiler) = compiler_opt {
                let mut cmd = if compiler.extension().and_then(|e| e.to_str()) == Some("py") {
                    let mut c = Command::new("python3");
                    c.arg(compiler);
                    c
                } else {
                    Command::new(compiler)
                };

                cmd.arg("compile")
                    .arg(&path)
                    .arg("--output")
                    .arg(&output_ui);

                let status = cmd.status().unwrap_or_else(|e| {
                    panic!("Failed to execute blueprint compiler on {:?}: {}", path, e)
                });

                if !status.success() {
                    panic!("blueprint-compiler failed on file: {:?}", path);
                }
            } else if !output_ui.exists() {
                panic!(
                    "Could not find `blueprint-compiler` and precompiled UI file {:?} does not exist. Please install blueprint-compiler.",
                    output_ui
                );
            } else {
                println!(
                    "cargo:warning=blueprint-compiler not found, using existing precompiled UI: {:?}",
                    output_ui
                );
            }
        }
    }
}

fn compile_schemas(data_dir: &Path) {
    let schema_file = data_dir.join("io.gitlab.lewisHeart.GnomePaths.gschema.xml");
    if schema_file.exists() {
        println!("cargo:rerun-if-changed={}", schema_file.display());
        if let Ok(status) = Command::new("glib-compile-schemas").arg(data_dir).status() {
            if !status.success() {
                println!("cargo:warning=glib-compile-schemas returned non-zero exit code");
            }
        }
    }
}

fn main() {
    let data_dir = Path::new("data");
    compile_schemas(data_dir);

    let resources_dir = Path::new("data/resources");
    let ui_dir = resources_dir.join("ui");

    if ui_dir.exists() {
        compile_blueprints(&ui_dir);
    }

    println!("cargo:rerun-if-changed=data/resources/resources.gresource.xml");
    println!("cargo:rerun-if-changed=data/resources/style.css");
    println!("cargo:rerun-if-changed=data/resources/icons");

    glib_build_tools::compile_resources(
        &[resources_dir],
        "data/resources/resources.gresource.xml",
        "gnome_paths.gresource",
    );
}
