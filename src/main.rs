mod parser;
use std::process;
use crate::parser::parser;
use handlebars::Handlebars;
use serde_json::json;
use std::fs::File;

fn main() {
    // arg parsing
    let res = parser();
    let apps: Vec<_> = res.apps.split(",").collect();
    let mut is_uv: bool = false;
    // end

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

    // end
  
    // init command chain
    
    
    if is_uv{
        println!("Creating virtual env with uv...");
        process::Command::new("uv").args(["venv",".venv"]).output().expect("venv failed");
        process::Command::new("cmd").args(["/C","uv pip install django"]).output().expect("venv failed");
        println!("Installing django with uv...");
    }else {
        println!("Creating virtual env...");
        process::Command::new("python").args(["-m","venv",".venv"]).output().expect("venv failed");
        process::Command::new("cmd").args(["/C","uv pip install django"]).output().expect("venv failed");
        println!("Installing django...");
    }
    
    println!("Creating Django App...");
    process::Command::new("cmd").args(["/C",".venv\\Scripts\\python.exe -m django startproject",res.name.as_str(),"."]).output().expect("Django App Failed.");

    
    if apps.len() > 1{
        println!("Creating Apps...");
        for app in apps {
            process::Command::new("cmd").args(["/C",".venv\\Scripts\\python.exe -m django startapp",app]).spawn().expect("Starting App Failed.");
        }
    }

    let settings = File::create("settings.py").unwrap();
    let mut h = Handlebars::new();
    h.register_template_file("template", "./settings.tpl").unwrap();
    h.render_to_write("template", &json!({"name" : "Kian","job" : "Software Dev"}),&settings).unwrap();

}

