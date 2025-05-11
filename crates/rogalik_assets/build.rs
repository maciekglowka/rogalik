use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

#[allow(dead_code)]
const ASSET_FILE_NAME: &str = "included_assets.rs";

fn main() {
    #[cfg(not(debug_assertions))]
    embedded();
    #[cfg(debug_assertions)]
    root_dir_only();
}

fn root_dir_only() {
    let (asset_dir, dest_path) = get_dirs();
    println!("cargo:rerun-if-changed={}", asset_dir.to_string_lossy());
    println!("cargo:rerun-if-env-changed=ROGALIK_ASSETS");

    let mut asset_file =
        File::create(&dest_path).expect(&format!("Can't create the asset_file at {:?}", dest_path));

    asset_file
        .write_all(
            format!(
                "
           const ASSET_ROOT: &str = \"{}\";\n\n
        ",
                asset_dir.as_path().to_str().unwrap()
            )
            .as_ref(),
        )
        .unwrap();
}

#[allow(dead_code)]
fn embedded() {
    let (asset_dir, dest_path) = get_dirs();
    println!("cargo:rerun-if-changed={}", asset_dir.to_string_lossy());
    println!("cargo:rerun-if-env-changed=ROGALIK_ASSETS");

    let mut asset_file =
        File::create(&dest_path).expect(&format!("Can't create the asset_file at {:?}", dest_path));

    asset_file
        .write_all(
            "fn get_embedded() -> HashMap<&'static str, &'static [u8]> {
                let mut assets = HashMap::new();
            \n"
            .as_ref(),
        )
        .unwrap();

    let paths = find_paths(&asset_dir, String::new());
    for path in paths.iter() {
        let absolute = asset_dir.join(path);
        asset_file
            .write_all(
                format!(
                    "
                assets.insert({:?}, include_bytes!({:?}).as_slice());
            ",
                    path,
                    absolute.to_string_lossy(),
                )
                .as_ref(),
            )
            .unwrap();
    }

    asset_file
        .write_all(
            "
        assets
        }\n\n"
                .as_ref(),
        )
        .unwrap();
}

fn get_dirs() -> (PathBuf, PathBuf) {
    let out_dir_var = std::env::var("OUT_DIR").expect("OUT_DIR env var is not set!");
    let dest_path = Path::new(&out_dir_var).join(ASSET_FILE_NAME);
    if let Ok(asset_dir_var) = std::env::var("ROGALIK_ASSETS") {
        (Path::new(&asset_dir_var).into(), dest_path)
    } else {
        (
            find_asset_root(&out_dir_var).expect("Default asset dir not found!"),
            dest_path,
        )
    }
}

fn find_paths(root: &Path, root_str: String) -> Vec<String> {
    if !root.is_dir() {
        return vec![root_str];
    }

    let mut paths = Vec::new();
    for e in std::fs::read_dir(root).unwrap() {
        let entry = e.unwrap();
        let path = entry.path();
        let file_name = path.file_name().unwrap();
        let relative = Path::new(&root_str).join(file_name);
        paths.extend(find_paths(&path, relative.to_string_lossy().into_owned()));
    }
    paths
}

fn find_asset_root(out_dir: &str) -> Option<PathBuf> {
    let mut cur = Path::new(out_dir);
    while let Some(parent) = cur.parent() {
        if parent.file_name().map(|a| a.to_str().unwrap()) == Some("target") {
            let project_root = parent.parent()?;
            let assets_root = project_root.join("assets");
            return Some(assets_root);
        }
        cur = parent;
    }

    None
}
