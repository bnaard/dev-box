use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn collect_yaml_files(dir: &Path, files: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).expect("read addon catalog directory") {
        let path = entry.expect("read addon catalog entry").path();
        if path.is_dir() {
            collect_yaml_files(&path, files);
        } else if matches!(
            path.extension().and_then(|ext| ext.to_str()),
            Some("yaml" | "yml")
        ) {
            files.push(path);
        }
    }
}

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let addons_dir = manifest_dir.join("../addons");
    let mut files = Vec::new();
    collect_yaml_files(&addons_dir, &mut files);
    files.sort();

    println!("cargo:rerun-if-changed={}", addons_dir.display());

    let mut generated =
        String::from("pub const EMBEDDED_ADDON_YAMLS: &[(&str, &str, &str)] = &[\n");
    for path in files {
        println!("cargo:rerun-if-changed={}", path.display());
        let category = path
            .parent()
            .and_then(Path::file_name)
            .and_then(|name| name.to_str())
            .expect("addon category directory");
        let source = path
            .strip_prefix(&addons_dir)
            .expect("addon path below catalog")
            .to_string_lossy();
        let content = fs::read_to_string(&path).expect("read addon YAML");
        generated.push_str(&format!("    ({category:?}, {source:?}, {content:?}),\n"));
    }
    generated.push_str("];\n");

    let out = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR")).join("embedded_addons.rs");
    fs::write(out, generated).expect("write embedded addon catalog");
}
