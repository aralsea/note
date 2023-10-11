use fs_extra::dir;
use regex::Regex;
use std::env;
use std::fs;
use std::io::prelude::*;
use std::io::ErrorKind;
use std::path::Path;

fn main() {
    let home_dir_str = env::var("HOME").unwrap();
    let home_dir = Path::new(&home_dir_str);

    // ~/.note_configを作成
    let config_dir = home_dir.join(".note");
    match fs::create_dir(&config_dir) {
        Ok(_) => {}
        Err(ref error) if error.kind() == ErrorKind::AlreadyExists => {}
        Err(error) => panic!("{error}"),
    };

    // templatesを.note_configの配下にコピー
    let mut options = dir::CopyOptions::new();
    options = options.overwrite(true);
    dir::copy("templates", &config_dir, &options).unwrap();

    // .indentconfig.yamlの中身のパスを書き換える
    let destination_file_path = &config_dir.join("templates/.indentconfig.yaml");
    let mut file_content = fs::read_to_string(destination_file_path).unwrap();

    let re = Regex::new(r"replace me").unwrap();
    file_content = re
        .replace_all(
            &file_content,
            &config_dir
                .join("templates/user-settings.yaml")
                .to_string_lossy(),
        )
        .to_string();
    let mut file = fs::File::create(destination_file_path).unwrap();
    file.write_all(file_content.as_bytes()).unwrap();

    // .indentconfig.yamlをホームディレクトリにコピー
    fs::copy(
        &config_dir.join("templates/.indentconfig.yaml"),
        &home_dir.join(".indentconfig.yaml"),
    )
    .unwrap();
}
