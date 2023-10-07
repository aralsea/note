use std::env;
use std::fs;
use std::io::ErrorKind;

fn main() {
    let home_dir = env::var("HOME").unwrap();
    let config_dir = format!("{home_dir}/.note_config");
    match fs::create_dir(&config_dir) {
        Ok(_) => {}
        Err(ref error) if error.kind() == ErrorKind::AlreadyExists => {}
        Err(error) => panic!(
            "There was a problem creating the config directory {}: {:?}",
            config_dir, error
        ),
    };
}
