use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    let profile = env::var("PROFILE").unwrap();
    let toolchain = env::var("TARGET").unwrap();

    let files = match target_os.as_ref() {
        "windows" => {
            let lib_folder = Path::new("..").join("lib").join(&toolchain);
            if !lib_folder.exists() {
                panic!("Could not find the path '{}'", lib_folder.display());
            }

            fs::read_dir(lib_folder).unwrap()
                .filter(|x| x.is_ok())
                .map(|x| x.unwrap())
                .map(|x| x.path())
                .filter(|x| !x.is_dir())
                .collect::<Vec<PathBuf>>()
        }

        x => panic!("Unknown target os '{}'", x),
    };

    let target_dir = Path::new("..").join("target").join(profile);
    for path in files {
        let target_file = target_dir.join(path.file_name().unwrap());
        println!("Copying '{}' into '{}'", path.display(), target_file.display());
        fs::copy(path.into_os_string(), target_file.into_os_string()).unwrap();
    }
}