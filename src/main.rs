use clap::{Args, Parser, Subcommand, ValueEnum};
use once_cell::sync::OnceCell;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::prelude::*;
use std::io::BufReader;
use std::io::ErrorKind;
use std::path::Path;
use std::path::PathBuf;
use std::process;
const CONFIG_JSON_PATH: &str = ".note/.note_config.json";
const LOCAL_CONFIG_JSON_PATH: &str = ".note_config.json";
const SETTINGS_JSON_PATH: &str = ".note/templates/.vscode/settings.json";
const JA_LATEXMKRC_PATH: &str = ".note/templates/.ja_latexmkrc";
const EN_LATEXMKRC_PATH: &str = ".note/templates/.en_latexmkrc";
const JA_TEMPLATE_PATH: &str = ".note/templates/src/ja_template.tex";

const EN_TEMPLATE_PATH: &str = ".note/templates/src/en_template.tex";
const BIB_PATH: &str = ".note/templates/bib/template.bib";
const GITIGNORE_PATH: &str = ".note/templates/.gitignore";
const README_PATH: &str = ".note/templates/README.md";

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
    #[arg(
        short,
        long = "lang",
        help = "The language setting for the new project"
    )]
    #[clap(value_enum, default_value_t=Language::Japanese)]
    language: Language,
}

#[derive(Debug, Args)]
struct ConfigArgs {
    #[arg(long, help = "Set or show the local configurations")]
    local: bool,
    #[arg(
        long = "author",
        help = "Set the default author name for newly created documents"
    )]
    author_name: Option<String>,
    #[arg(
        short,
        long = "lang",
        value_enum,
        help = "Set the default language for newly created projects"
    )]
    language: Option<Language>,
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::New(args) => {
            Config::load_config(Scope::Local);
            create_project(&args)
        }
        Commands::Config(args) => {
            let scope = if args.local {
                Scope::Local
            } else {
                Scope::Global
            };
            Config::load_config(scope);
            let config = get_new_config(args);
            config.save_to_file();
            if scope == Scope::Local && config.scope == Scope::Global {
                println!("Local config not found.")
            }
            println!("Current config: {:#?}", config);
        }
    }
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
    prepare_readme(&project);
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
        Err(_) => panic!(
            "There was a problem creating the directory {}",
            &project.path.display()
        ),
    };

    for sub_directory in SUB_DIRECTORIES {
        fs::create_dir(project.path.join(sub_directory)).unwrap();
    }
}

#[derive(Debug, Copy, Clone, ValueEnum, Serialize, Deserialize)]
enum Language {
    English,
    Japanese,
}
impl Default for Language {
    fn default() -> Self {
        Self::Japanese
    }
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
fn prepare_settings_json(project: &Project) {
    let setting = get_settings_json(project);

    //serialized
    let file_content: String = serde_json::to_string_pretty(&setting).unwrap();
    //write
    let mut file = fs::File::create(project.path.join(".vscode/settings.json")).unwrap();
    file.write_all(file_content.as_bytes()).unwrap();
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
    let args = &mut setting.latex_workshop_latex_tools[0].args;
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

    setting
}

fn prepare_latexmkrc(project: &Project) {
    let home_dir_str = env::var("HOME").unwrap();
    let home_dir = Path::new(&home_dir_str);

    let file_content = fs::read_to_string(home_dir.join(match project.language {
        Language::English => EN_LATEXMKRC_PATH,
        Language::Japanese => JA_LATEXMKRC_PATH,
    }))
    .unwrap();

    let destination_file_path = project.path.join(".latexmkrc");

    let mut file = fs::File::create(destination_file_path).unwrap();
    file.write_all(file_content.as_bytes()).unwrap();
}

fn prepare_tex_file(project: &Project) {
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
            &format!(r"\author{{{}}}", Config::get().author_name),
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
    let mut file = fs::File::create(destination_file_path).unwrap();
    file.write_all(file_content.as_bytes()).unwrap();
}
fn prepare_bib_file(project: &Project) {
    let home_dir_str = env::var("HOME").unwrap();
    let home_dir = Path::new(&home_dir_str);
    let file_content = fs::read_to_string(home_dir.join(BIB_PATH)).unwrap();

    let destination_file_path = project.path.join(format!("bib/{}.bib", project.name));

    let mut file = fs::File::create(destination_file_path).unwrap();
    file.write_all(file_content.as_bytes()).unwrap();
}
fn prepare_gitignore(project: &Project) {
    let home_dir_str = env::var("HOME").unwrap();
    let home_dir = Path::new(&home_dir_str);
    let file_content = fs::read_to_string(home_dir.join(GITIGNORE_PATH)).unwrap();

    let destination_file_path = project.path.join(".gitignore");

    let mut file = fs::File::create(destination_file_path).unwrap();
    file.write_all(file_content.as_bytes()).unwrap();
}

fn uppercase_first(data: &str) -> String {
    // Uppercase first letter.
    let mut result = String::new();
    let mut first = true;
    for value in data.chars() {
        if first {
            result.push(value.to_ascii_uppercase());
            first = false;
        } else {
            result.push(value);
        }
    }
    result
}

fn prepare_readme(project: &Project) {
    let home_dir_str = env::var("HOME").unwrap();
    let home_dir = Path::new(&home_dir_str);
    let mut file_content = fs::read_to_string(home_dir.join(README_PATH)).unwrap();
    file_content =
        file_content.replace("CAPITALIZED_PROJECT_NAME", &uppercase_first(&project.name));
    file_content = file_content.replace("PROJECT_NAME", &project.name);
    file_content = file_content.replace("SOURCE_FILE", &project.name);

    let destination_file_path = project.path.join("README.md");
    let mut file = fs::File::create(destination_file_path).unwrap();
    file.write_all(file_content.as_bytes()).unwrap();
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Config {
    scope: Scope,

    #[serde(default)]
    author_name: String,

    #[serde(default)]
    language: Language,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq)]
enum Scope {
    Global,
    Local,
}
static CONFIG: OnceCell<Config> = OnceCell::new();

impl Config {
    fn get() -> &'static Config {
        CONFIG.get().expect("Config is not initialized")
    }

    fn global_default() -> Config {
        Config {
            author_name: String::default(),
            language: Language::Japanese,
            scope: Scope::Global,
        }
    }

    fn create_global_config_file() {
        let config = Config::global_default();
        config.save_to_file();
        CONFIG
            .set(config)
            .expect("Config has already been initialized.");
    }

    fn save_to_file(&self) {
        let file_content: String = serde_json::to_string_pretty(self).unwrap();
        let config_file_path = match self.scope {
            Scope::Global => {
                let home_dir = env::var("HOME").unwrap();
                PathBuf::from(&home_dir).join(CONFIG_JSON_PATH)
            }
            Scope::Local => {
                let current_dir = env::current_dir().unwrap();
                PathBuf::from(&current_dir).join(LOCAL_CONFIG_JSON_PATH)
            }
        };
        let mut file = fs::File::create(config_file_path).expect("Failed to save the config file.");
        file.write_all(file_content.as_bytes()).unwrap();
    }

    fn load_local_config() -> Result<(), std::io::Error> {
        let current_path = env::current_dir().unwrap();
        let config_file = match fs::File::open(current_path.join(LOCAL_CONFIG_JSON_PATH)) {
            Ok(config_file) => config_file,
            Err(error) => {
                return Err(error);
            }
        };
        let reader = BufReader::new(config_file);
        let config: Config = serde_json::from_reader(reader).unwrap();
        CONFIG.set(config).unwrap();
        return Ok(());
    }

    /// scope = Scope::Localのときはlocalのconfigを読みに行き、なければglobalを読む
    fn load_config(scope: Scope) {
        if scope == Scope::Local {
            match Config::load_local_config() {
                Ok(_) => return,
                Err(error) if error.kind() == ErrorKind::NotFound => {}
                Err(error) => {
                    panic!(
                        "There was a problem opening the local config file: {:?}",
                        error
                    )
                }
            }
        }

        let home_dir = env::var("HOME").unwrap();
        let config_file_path = PathBuf::from(&home_dir).join(CONFIG_JSON_PATH);

        let config_file = match fs::File::open(config_file_path) {
            Ok(config_file) => config_file,
            Err(ref error) if error.kind() == ErrorKind::NotFound => {
                Config::create_global_config_file();
                return;
            }
            Err(error) => {
                panic!("There was a problem opening the config file: {:?}", error)
            }
        };
        let reader = BufReader::new(config_file);
        let config: Config = serde_json::from_reader(reader).unwrap();
        CONFIG.set(config).unwrap();
    }
}

/// 現在のconfigをargsに従って変更したものを返す
/// argsにscope以外に変更の指定がなければ何もしない
/// 特にscopeも変更しない
fn get_new_config(args: ConfigArgs) -> Config {
    let scope = if args.local {
        Scope::Local
    } else {
        Scope::Global
    };
    let mut config = Config::get().clone();
    if let Some(author_name) = &args.author_name {
        config = Config {
            scope,
            author_name: author_name.to_string(),
            ..config
        };
    }
    if let Some(language) = args.language {
        config = Config {
            scope,
            language,
            ..config
        };
    }
    return config;
}
