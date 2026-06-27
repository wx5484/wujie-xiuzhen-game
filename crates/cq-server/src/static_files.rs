use std::path::PathBuf;

use tower_http::services::{ServeDir, ServeFile};

pub fn service(public_dir: &str) -> ServeDir<ServeFile> {
    service_from_root(PathBuf::from(public_dir))
}

pub fn subdir_service(public_dir: &str, subdir: &str) -> ServeDir<ServeFile> {
    let subdir = subdir.trim_matches('/');
    service_from_root(PathBuf::from(public_dir).join(subdir))
}

fn service_from_root(root: PathBuf) -> ServeDir<ServeFile> {
    let index = root.join("index.html");
    ServeDir::new(root).fallback(ServeFile::new(index))
}
