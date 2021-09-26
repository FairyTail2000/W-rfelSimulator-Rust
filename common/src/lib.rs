use directories::ProjectDirs;
use std::fs::create_dir_all;
use std::path::{PathBuf, MAIN_SEPARATOR};
use std::process::exit;

pub fn settings_path(file: &str) -> PathBuf {
    match ProjectDirs::from("", "", "würfeln") {
        None => PathBuf::from(format!(".{}{}", MAIN_SEPARATOR, file)),
        Some(dirs) => {
            if !dirs.data_dir().exists() {
                match create_dir_all(dirs.data_dir()) {
                    Ok(_) => {}
                    Err(e) => {
                        eprintln!("{}", e);
                        exit(-1);
                    }
                }
            }
            PathBuf::from(format!(
                "{}{}{}",
                dirs.data_dir().to_str().unwrap(),
                MAIN_SEPARATOR,
                file
            ))
        }
    }
}