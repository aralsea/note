#![allow(unused)]
use std::path::Path;
use std::env;
use std::fs;
use std::io::BufReader;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::prelude::*;
fn main() {
    let args:Vec<String> = env::args().collect();

    if args.len()>2 && args[1] == "new" {
        let project_name = &args[2];
        create_project(project_name, Language::Japanese);
    }else{
        eprintln!("unknown command!");
    }
}

fn create_project(project_name:&str, language:Language){
    match prepare_directories(project_name){
        Ok(_) => println!("succeeded!"),
        Err(e) => panic!("{e}"),
    }

    let current_path = env::current_dir().unwrap();
    let project_path = current_path.join(project_name);
    let setting = get_settings_json(project_path.as_path(), language);
    write_file(project_path.as_path(), setting);
 }


fn prepare_directories(project_name:&str)->Result<(), std::io::Error>{
    match fs::create_dir(project_name){
        Ok(_) => println!("ok"),
        Err(e) => panic!("{e}"),
    }
    
    for sub_directory in ["/.vscode", "src", "out", "bib"]{
        fs::create_dir_all(format!("{project_name}/{sub_directory}")).unwrap_or_else(|why| {
            println!("! {:?}", why.kind());
        });
    }

    return Ok(());
}
enum Language{
    English,
    Japanese
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
struct LatexTool{
    name: String,
    command: String,
    args: Vec<String>,
    env: HashMap<String, String>,
}

fn get_settings_json(project_path: &Path, language:Language)->VscodeSetting{
    let file_name = match language{
        Language::English => panic!("English configuration is not implemented."),
        Language::Japanese => "templates/ja/ja_project/.vscode/settings.json",
    };
    
    let file = fs::File::open(file_name).unwrap();
    let reader = BufReader::new(file);
    let mut setting: VscodeSetting = serde_json::from_reader(reader).unwrap();

  

    //出力ディレクトリを設定
    setting.latex_workshop_latex_outdir = project_path.join("out").into_os_string().into_string().unwrap();
    
    //.latexmkrcの場所を設定
    let mut args = &mut setting.latex_workshop_latex_tools[0].args;
    let mut r_option_found = false;
    for i in 0..(args.len()-1){
        if args[i] == "-r"{
            r_option_found = true;
            args[i+1] = project_path.join(".latexmkrc").into_os_string().into_string().unwrap();
            break;
        }
    }
    if !r_option_found{
        panic!("\"-r\" option not found in {file_name}.")
    }

    return setting;
}
fn write_file(project_path:&Path, setting: VscodeSetting) -> std::io::Result<()> {
    //serialized
    let serialized: String = serde_json::to_string(&setting).unwrap();
    println!("{serialized}");
    //write
    let mut file = fs::File::create(project_path.join(".vscode/settings.json"))?;
    file.write_all(serialized.as_bytes())?;
    Ok(())
}