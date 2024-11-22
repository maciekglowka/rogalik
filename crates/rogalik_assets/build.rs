use std::{fs::File, io::Write, path::Path};

const ASSET_FILE_NAME: &str = "included_assets.rs";

fn main() {
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR env var is not set!");
    let dest_path = Path::new(&out_dir).join(ASSET_FILE_NAME);
    let Ok(asset_dir_var) = std::env::var("ROGALIK_ASSETS") else {
        return;
    };
    let asset_dir = Path::new(&asset_dir_var);
    println!("cargo::rerun-if-changed={}", asset_dir.to_string_lossy());

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
        let absolute = Path::new(asset_dir).join(path);
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
