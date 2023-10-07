#![allow(unused)]
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path;

const JA_SETTINGS_JSON: &str = include_str!("../templates/ja/ja-project/.vscode/settings.json");
const JA_LATEXMKRC: &str = include_str!("../templates/ja/ja-project/.latexmkrc");
fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 2 && args[1] == "new" {
        let project_name = &args[2];
        create_project(project_name, Language::Japanese);
    } else {
        eprintln!("unknown command!");
    }
}

fn create_project(project_name: &str, language: Language) {
    match prepare_directories(project_name) {
        Ok(_) => println!("succeeded!"),
        Err(e) => panic!("{e}"),
    }

    let current_path = env::current_dir().unwrap();
    let project_path = current_path.join(project_name);
    prepare_settings_json(&project_path, &language);
    prepare_latexmkrc(&project_path, &language);
}

fn prepare_directories(project_name: &str) -> Result<(), std::io::Error> {
    match fs::create_dir(project_name) {
        Ok(_) => println!("ok"),
        Err(e) => panic!("{e}"),
    }

    for sub_directory in ["/.vscode", "src", "out", "bib"] {
        fs::create_dir_all(format!("{project_name}/{sub_directory}")).unwrap_or_else(|why| {
            println!("! {:?}", why.kind());
        });
    }

    return Ok(());
}
enum Language {
    English,
    Japanese,
}
#[derive(Debug, Serialize, Deserialize)]
struct VscodeSetting {
    #[serde(rename = "latex-workshop.latex.recipe.default")]
    latex_workshop_latex_recipe_default: String,

    #[serde(rename = "latex-workshop.latex.tools")]
    latex_workshop_latex_tools: Vec<LatexTool>,

    #[serde(rename = "latex-workshop.latex.outDir")]
    latex_workshop_latex_outdir: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct LatexTool {
    name: String,
    command: String,
    args: Vec<String>,
    env: HashMap<String, String>,
}
fn prepare_settings_json(project_path: &Path, language: &Language) -> Result<(), std::io::Error> {
    let setting = get_settings_json(project_path, language);

    //serialized
    let file_content: String = serde_json::to_string(&setting).unwrap();
    //write
    let mut file = fs::File::create(project_path.join(".vscode/settings.json"))?;
    file.write_all(file_content.as_bytes())?;
    Ok(())
}
fn get_settings_json(project_path: &Path, language: &Language) -> VscodeSetting {
    let original_file_content = match language {
        Language::English => panic!("English configuration is not implemented."),
        Language::Japanese => JA_SETTINGS_JSON,
    };

    let mut setting: VscodeSetting = serde_json::from_str(original_file_content).unwrap();

    //出力ディレクトリを設定
    setting.latex_workshop_latex_outdir = project_path
        .join("out")
        .into_os_string()
        .into_string()
        .unwrap();

    //.latexmkrcの場所を設定
    let mut args = &mut setting.latex_workshop_latex_tools[0].args;
    let mut r_option_found = false;
    for i in 0..(args.len() - 1) {
        if args[i] == "-r" {
            r_option_found = true;
            args[i + 1] = project_path
                .join(".latexmkrc")
                .into_os_string()
                .into_string()
                .unwrap();
            break;
        }
    }
    if !r_option_found {
        let file_name = match (language) {
            Language::English => panic!("English configuration is not implemented."),
            Language::Japanese => "templates/ja/ja-project/.vscode/settings.json",
        };
        panic!("\"-r\" option not found in {file_name}.")
    }

    return setting;
}

fn prepare_latexmkrc(project_path: &Path, language: &Language) -> Result<(), std::io::Error> {
    let file_content = match language {
        Language::English => panic!("English configuration is not implemented."),
        Language::Japanese => JA_LATEXMKRC,
    };

    let destination_file_path = project_path.join(".latexmkrc");

    let mut file = fs::File::create(destination_file_path)?;
    file.write_all(file_content.as_bytes());
    Ok(())
}
