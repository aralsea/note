#![allow(unused)]
use clap::{Args, Parser, Subcommand, ValueEnum};
use once_cell::sync::OnceCell;
use regex::Regex;
use serde::de;
use serde::{Deserialize, Serialize};

use std::alloc::GlobalAlloc;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::prelude::*;
use std::io::BufReader;
use std::io::ErrorKind;
use std::path::Path;
use std::path::PathBuf;
use std::process;

const NOTE_PATH: &str = ".note";
const CONFIG_JSON_PATH: &str = ".note/config.json";
const SETTINGS_JSON_PATH: &str = ".note/templates/.vscode/settings.json";
const JA_LATEXMKRC_PATH: &str = ".note/templates/.ja_latexmkrc";
const EN_LATEXMKRC_PATH: &str = ".note/templates/.en_latexmkrc";
const JA_TEMPLATE_PATH: &str = ".note/templates/src/ja_template.tex";

const EN_TEMPLATE_PATH: &str = ".note/templates/src/en_template.tex";
const BIB_PATH: &str = ".note/templates/bib/template.bib";
const GITIGNORE_PATH: &str = ".note/templates/.gitignore";

const SUB_DIRECTORIES: [&str; 4] = [".vscode", "src", "out", "bib"];
#[derive(Debug, Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    #[command(about = "Create a new latex project in the current directory")]
    New(NewArgs),
    #[command(about = "View and edit the current configurations")]
    Config(ConfigArgs),
}
#[derive(Debug, Args)]
struct NewArgs {
    #[arg(help = "The name of the new LaTeX project")]
    project_name: String,
    #[arg(short, long, help = "The language setting for the new project")]
    #[clap(value_enum, default_value_t=Language::Japanese)]
    language: Language,
}

#[derive(Debug, Args)]
struct ConfigArgs {
    #[arg(long, help = "Set the default author name for newly created documents")]
    author_name: Option<String>,
}

fn main() {
    Config::load_config();
    let cli = Cli::parse();
    match cli.command {
        Commands::New(args) => create_project(&args),
        Commands::Config(args) => {
            if let Some(author_name) = &args.author_name {
                set_author_name(author_name);
            }
            show_config();
        }
    }
    return;
}

struct Project {
    name: String,
    path: PathBuf,
    language: Language,
}
fn create_project(args: &NewArgs) {
    let current_path = env::current_dir().unwrap();
    let project = Project {
        name: args.project_name.clone(),
        path: current_path.join(&args.project_name),
        language: args.language,
    };
    prepare_directories(&project);

    prepare_settings_json(&project);
    prepare_latexmkrc(&project);
    prepare_tex_file(&project);
    prepare_bib_file(&project);
    prepare_gitignore(&project);
    print!(
        "Success! 🎉\n\n\
    A LaTeX-ready directory has been created for you.\n\
    Path: {}\n\
    \n\
    You can now navigate to the directory and start your LaTeX project. Happy writing!\n",
        project.path.display()
    )
}

fn prepare_directories(project: &Project) {
    match fs::create_dir(&project.path) {
        Ok(_) => {}
        Err(ref error) if error.kind() == ErrorKind::AlreadyExists => {
            println!("The directory `{}` already exists.", &project.name);
            process::exit(0);
        }
        Err(error) => panic!(
            "There was a problem creating the directory {}",
            &project.path.display()
        ),
    };

    for sub_directory in SUB_DIRECTORIES {
        fs::create_dir(project.path.join(sub_directory)).unwrap();
    }
}

#[derive(Debug, Copy, Clone, ValueEnum)]
enum Language {
    English,
    Japanese,
}
#[derive(Debug, Serialize, Deserialize)]
struct VscodeSetting {
    #[serde(rename = "latex-workshop.latex.tools")]
    latex_workshop_latex_tools: Vec<LatexTool>,

    #[serde(rename = "latex-workshop.latex.outDir")]
    latex_workshop_latex_out_dir: String,

    #[serde(flatten)]
    other_fields: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
struct LatexTool {
    name: String,
    command: String,
    args: Vec<String>,
    env: HashMap<String, String>,
}
fn prepare_settings_json(project: &Project) -> std::io::Result<()> {
    let setting = get_settings_json(project);

    //serialized
    let file_content: String = serde_json::to_string_pretty(&setting).unwrap();
    //write
    let mut file = fs::File::create(project.path.join(".vscode/settings.json"))?;
    file.write_all(file_content.as_bytes())?;
    return Ok(());
}
fn get_settings_json(project: &Project) -> VscodeSetting {
    let home_dir_str = env::var("HOME").unwrap();
    let home_dir = Path::new(&home_dir_str);
    let file = fs::File::open(home_dir.join(SETTINGS_JSON_PATH)).unwrap();
    let reader = BufReader::new(file);
    let mut setting: VscodeSetting = serde_json::from_reader(reader).unwrap();

    //出力ディレクトリを設定
    setting.latex_workshop_latex_out_dir = project
        .path
        .join("out")
        .into_os_string()
        .into_string()
        .unwrap();

    //参照する.latexmkrcの場所を設定
    let mut args = &mut setting.latex_workshop_latex_tools[0].args;
    let mut r_option_found = false;
    for i in 0..(args.len() - 1) {
        if args[i] == "-r" {
            r_option_found = true;
            args[i + 1] = project
                .path
                .join(".latexmkrc")
                .into_os_string()
                .into_string()
                .unwrap();
            break;
        }
    }
    if !r_option_found {
        let file_name = "templates/.vscode/settings.json";
        panic!("\"-r\" option not found in {file_name}.")
    }

    return setting;
}

fn prepare_latexmkrc(project: &Project) -> std::io::Result<()> {
    let home_dir_str = env::var("HOME").unwrap();
    let home_dir = Path::new(&home_dir_str);

    let file_content = fs::read_to_string(home_dir.join(match project.language {
        Language::English => EN_LATEXMKRC_PATH,
        Language::Japanese => JA_LATEXMKRC_PATH,
    }))
    .unwrap();

    let destination_file_path = project.path.join(".latexmkrc");

    let mut file = fs::File::create(destination_file_path)?;
    file.write_all(file_content.as_bytes());
    return Ok(());
}

fn prepare_tex_file(project: &Project) -> std::io::Result<()> {
    let home_dir_str = env::var("HOME").unwrap();
    let home_dir = Path::new(&home_dir_str);
    let mut file_content = fs::read_to_string(home_dir.join(match project.language {
        Language::English => EN_TEMPLATE_PATH,
        Language::Japanese => JA_TEMPLATE_PATH,
    }))
    .unwrap();

    // authorを書き換える
    let re = Regex::new(r"\\author\{[^}]*\}").unwrap(); // \author{.*}にマッチするregex
    file_content = re
        .replace_all(
            &file_content,
            &format!(r"\author{{{}}}", Config::global().author_name),
        )
        .to_string();

    // addbibresourceを書き換える
    let re = Regex::new(r"\\addbibresource\{[^}]*\}").unwrap(); // \addbibresource{.*}にマッチするregex
    file_content = re
        .replace_all(
            &file_content,
            &format!(r"\addbibresource{{../bib/{}.bib}}", project.name),
        )
        .to_string();

    let destination_file_path = project.path.join(format!("src/{}.tex", project.name));
    let mut file = fs::File::create(destination_file_path)?;
    file.write_all(file_content.as_bytes());
    return Ok(());
}
fn prepare_bib_file(project: &Project) -> std::io::Result<()> {
    let home_dir_str = env::var("HOME").unwrap();
    let home_dir = Path::new(&home_dir_str);
    let file_content = fs::read_to_string(home_dir.join(BIB_PATH)).unwrap();

    let destination_file_path = project.path.join(format!("bib/{}.bib", project.name));

    let mut file = fs::File::create(destination_file_path)?;
    file.write_all(file_content.as_bytes());
    return Ok(());
}
fn prepare_gitignore(project: &Project) -> std::io::Result<()> {
    let home_dir_str = env::var("HOME").unwrap();
    let home_dir = Path::new(&home_dir_str);
    let file_content = fs::read_to_string(home_dir.join(GITIGNORE_PATH)).unwrap();

    let destination_file_path = project.path.join(".gitignore");

    let mut file = fs::File::create(destination_file_path)?;
    file.write_all(file_content.as_bytes());
    return Ok(());
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Config {
    #[serde(default)]
    author_name: String,
}
static CONFIG: OnceCell<Config> = OnceCell::new();

impl Config {
    fn global() -> &'static Config {
        CONFIG.get().expect("Config is not initialized")
    }

    fn default() -> Config {
        return Config {
            author_name: String::default(),
        };
    }
    fn path() -> PathBuf {
        let home_dir = env::var("HOME").unwrap();
        let config_dir = PathBuf::from(&home_dir).join(CONFIG_JSON_PATH);
        return config_dir;
    }
    fn create_config_file() {
        // CONFIG_PATHを新規作成する
        let config = Config::default();
        CONFIG
            .set(Config::default())
            .expect("Config has already been initialized.");
        config.save_to_file();
    }
    fn save_to_file(&self) {
        let file_content: String = serde_json::to_string_pretty(self).unwrap();
        let destination_file_path = Config::path();

        let mut file =
            fs::File::create(destination_file_path).expect("Failed to save the config file.");
        file.write_all(file_content.as_bytes());
    }

    fn load_config() {
        let config_file = match fs::File::open(Config::path()) {
            Ok(config_file) => config_file,
            Err(ref error) if error.kind() == ErrorKind::NotFound => {
                Config::create_config_file();
                return;
            }
            Err(error) => {
                panic!("There was a problem opening the config file: {:?}", error)
            }
        };
        let reader = BufReader::new(config_file);
        let config: Config = serde_json::from_reader(reader).unwrap();
        CONFIG.set(config);
        return;
    }
}

fn show_config() {
    println!("{:#?}", Config::global());
}
fn set_author_name(author_name: &str) {
    let new_config = Config {
        author_name: author_name.to_string(),
        ..Config::global().clone()
    };
    new_config.save_to_file();
}
