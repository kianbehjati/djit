mod parser;
use std::process;
use crate::parser::parser;
use handlebars::Handlebars;
use serde_json::json;
use std::fs::{File, write,create_dir};

fn main() {
    // arg parsing
    let res = parser();
    let apps: Vec<_> = res.apps.split(",").collect();
    let mut is_uv: bool = false;

    // embeding files into binary
    let sec_gen = include_str!("sec_gen.py");
    let settings_tpl = include_str!("settings.tpl");
    write("sec_gen.py", sec_gen).expect("Failed to write python script!");
    write("settings.tpl", settings_tpl).expect("Failed to write settings template!");

    // look for uv
    if cfg!(target_os = "windows"){
        let process_c = process::Command::new("powershell").args(["-c","(Get-command uv).Path"]).output().expect("powershell failed");
        match String::from_utf8_lossy(&process_c.stdout).len() {
            0 => println!("no uv found :("),
            _ => {
                println!("you're a uv guy :>");
                is_uv = true;
            }
        }
    }
    else {
        let process_c = process::Command::new("sh").args(["which","uv"]).output().expect("sh failed");
        match String::from_utf8_lossy(&process_c.stdout).len() {
            0 => println!("no uv found :("),
            _ => {
                println!("you're a uv guy :>");
                is_uv = true;
            }
        }
    }


  
    // init command chain
    
    
    if is_uv{
        println!("Creating virtual env with uv...");
        process::Command::new("uv").args(["venv",".venv"]).output().expect("venv failed");
        process::Command::new("cmd").args(["/C","uv pip install django python-dotenv"]).output().expect("venv failed");
        println!("Installing django with uv...");
    }else {
        println!("Creating virtual env...");
        process::Command::new("python").args(["-m","venv",".venv"]).output().expect("venv failed");
        process::Command::new("cmd").args(["/C","uv pip install django python-dotenv"]).output().expect("venv failed");
        println!("Installing django...");
    }
    
    println!("Creating Django App...");
    process::Command::new("cmd").args(["/C",".venv\\Scripts\\python.exe -m django startproject",res.name.as_str(),"."]).output().expect("Django App Failed.");

    let mut apps_str = String::new();

    if apps.len() > 1{
        println!("Creating Apps...");
        for app in apps {
            apps_str.push_str(&format!(", \"{}\"",app));
            process::Command::new("cmd").args(["/C",".venv\\Scripts\\python.exe -m django startapp",app]).spawn().expect("Starting App Failed.");
        }
    }

    //handlebars template rendering 
    let settings = File::create("settings.py").expect("Failed to Create settings.py file.");
    let mut hb = Handlebars::new();
    hb.register_template_file("template", "./settings.tpl").expect("Failed to create template from settings.tpl in handlebars");
    hb.render_to_write("template", &json!({"app_name" : res.name.as_str(),"apps" : apps_str}),&settings).unwrap();
    let settings_path = res.name.clone()+"\\"+"settings.py";
    let env_path = res.name.clone()+"\\"+".env";
    let settings_backup = res.name.clone()+"\\"+"settings.bkp.py";

    process::Command::new("cmd").args(["/C","copy",settings_path.as_str(),settings_backup.as_str()]).output().expect("Failed to copy backup original settings. ");
    process::Command::new("cmd").args(["/C","move","settings.py",settings_path.as_str()]).output().expect("Failed to replace settings. ");
    process::Command::new("cmd").args(["/C",".venv\\Scripts\\python.exe sec_gen.py"]).output().expect("Failed to run .env script. ");
    process::Command::new("cmd").args(["/C","move",".env",env_path.as_str()]).output().expect("Failed to move .env into main app. ");

    //clean up
    process::Command::new("cmd").args(["/C","del","/Q","settings.tpl","sec_gen.py"]).spawn().expect("Failed to remove junk!");

    //templates
    create_dir("templates").expect("Failed to create 'templates'. ");
}

