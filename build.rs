use std::env;
use std::fs;
use std::io::ErrorKind;
use std::path::Path;

use fs_extra::dir;
fn main() {
    let home_dir_str = env::var("HOME").unwrap();
    let home_dir = Path::new(&home_dir_str);

    // ~/.note_configを作成
    let config_dir = home_dir.join(".note_config");
    match fs::create_dir(&config_dir) {
        Ok(_) => {}
        Err(ref error) if error.kind() == ErrorKind::AlreadyExists => {}
        Err(error) => panic!("{error}"),
    };

    // templatesを.note_config/にコピー
    let mut options = dir::CopyOptions::new();
    options = options.skip_exist(true);
    dir::copy("templates", config_dir, &options).unwrap();
}
