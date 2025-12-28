mod parser;
mod check;
use std::process;
use crate::parser::{DjangoOptions, parser};
use handlebars::Handlebars;
use serde_json::json;
use std::fs::{File, write,create_dir};

fn main() {

    //checking Internet Connection
    if !(check::checker()){
        return;
    };

    // arg parsing
    let django_option = parser();
    let res:DjangoOptions; 
    match django_option {
        Some(d) => {
            res = d;
        }
        None => {
            println!("Usage: \n \tdjit project_name apps(e.g : firstapp,secondapp,...)\n \t#Note : no space between app , seperate with ','.\n \tdefault django app name is core");
            return;
        }
    }
    
    let apps: Vec<_> = res.apps.split(",").collect();
    let mut is_uv: bool = false;

    // embeding files into binary
    let sec_gen = include_str!("sec_gen.py");
    let settings_tpl = include_str!("settings.tpl");
    write("sec_gen.py", sec_gen).expect("Failed to write python script!");
    write("settings.tpl", settings_tpl).expect("Failed to write settings template!");

    // look for uv & python
    if cfg!(target_os = "windows"){
        let process_c = process::Command::new("powershell").args(["-c","(Get-command uv).Path"]).output().expect("powershell failed");
        match String::from_utf8_lossy(&process_c.stdout).len() {
            0 => {
                println!("uv not found :(");
                let python_path = process::Command::new("powershell").args(["-c","(Get-command python).Path"]).output().expect("powershell failed");
                match String::from_utf8_lossy(&python_path.stdout).len() {
                    0 => {
                        println!("python not installed.");
                        return;
                    }
                    _ => ()
                }
            },
            _ => {
                println!("you're a uv guy :>");
                is_uv = true;
            }
        }
    }
    else {
        let process_c = process::Command::new("which").args(["uv"]).output().expect("bash failed");
        match String::from_utf8_lossy(&process_c.stdout).len() {
            0 => {
                println!("uv not found :(");
                let python_path = process::Command::new("which").args(["python3"]).output().expect("bash shell failed");
                match String::from_utf8_lossy(&python_path.stdout).len() {
                    0 => {
                        println!("python not installed.");
                        return;
                    }
                    _ => ()
                }
            },
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
        println!("Installing django with uv...");
        if cfg!(target_os = "windows"){
            process::Command::new("cmd").args(["/C","uv pip install django python-dotenv"]).output().expect("venv failed");
        }
        else {
            process::Command::new("uv").args(["pip" ,"install" ,"django" ,"python-dotenv"]).output().expect("venv failed");
        }
        
    }else {
        println!("Creating virtual env...");
        if cfg!(target_os = "windows"){
            process::Command::new("python").args(["-m","venv",".venv"]).output().expect("venv failed");
            process::Command::new("cmd").args(["/C",".venv\\Scripts\\pip.exe install django python-dotenv"]).output().expect("venv failed");
        }
        else {
            process::Command::new("sudo").args(["apt","install","python3-venv"]).output().expect("installing python venv failed");
            process::Command::new("python3").args(["-m","venv",".venv"]).output().expect("venv failed");
            println!("Installing django...");
            process::Command::new(".venv/bin/pip").args(["install" ,"django" ,"python-dotenv"]).output().expect("venv failed"); // will get an error bc python -m venv doesn't work in linux
        }
        
    }
    
    println!("Creating Django Project...");
    if cfg!(target_os = "windows"){
        process::Command::new("cmd").args(["/C",".venv\\Scripts\\python.exe -m django startproject",res.name.as_str(),"."]).output().expect("Django App Failed.");
    }
    else {
        process::Command::new(".venv/bin/python").args(["-m" ,"django" ,"startproject",res.name.as_str(),"."]).output().expect("Django App Failed.");
    }

    let mut apps_str = String::new();

    if apps.len() > 1{
        println!("Creating Apps...");
        for app in apps {
            apps_str.push_str(&format!(", \"{}\"",app));
            if cfg!(target_os = "windows"){
                process::Command::new("cmd").args(["/C",".venv\\Scripts\\python.exe -m django startapp",app]).spawn().expect("Starting App Failed.");
            }
            else {
                process::Command::new(".venv/bin/python").args(["-m" ,"django" ,"startapp",app]).spawn().expect("Starting App Failed.");
            }
        }
    }

    //handlebars template rendering 
    let settings = File::create("settings.py").expect("Failed to Create settings.py file.");
    let mut hb = Handlebars::new();
    hb.register_template_file("template", "./settings.tpl").expect("Failed to create template from settings.tpl in handlebars");
    hb.render_to_write("template", &json!({"app_name" : res.name.as_str(),"apps" : apps_str}),&settings).unwrap();

    let settings_path:String;
    let env_path:String;
    let settings_backup:String;

    if cfg!(target_os = "windows"){
        settings_path = res.name.clone()+"\\"+"settings.py";
        env_path = res.name.clone()+"\\"+".env";
        settings_backup = res.name.clone()+"\\"+"settings.bkp.py";
    }
    else {
        settings_path = res.name.clone()+"/"+"settings.py";
        env_path = res.name.clone()+"/"+".env";
        settings_backup = res.name.clone()+"/"+"settings.bkp.py";
    }
    

    if cfg!(target_os = "windows"){
        process::Command::new("cmd").args(["/C","copy",settings_path.as_str(),settings_backup.as_str()]).output().expect("Failed to copy backup original settings. ");
        process::Command::new("cmd").args(["/C","move","settings.py",settings_path.as_str()]).output().expect("Failed to replace settings. ");
        process::Command::new("cmd").args(["/C",".venv\\Scripts\\python.exe sec_gen.py"]).output().expect("Failed to run .env script. ");
        process::Command::new("cmd").args(["/C","move",".env",env_path.as_str()]).output().expect("Failed to move .env into main app. ");
    }
    else {
        process::Command::new("cp").args([settings_path.as_str(),settings_backup.as_str()]).output().expect("Failed to copy backup original settings. ");
        process::Command::new("mv").args(["settings.py",settings_path.as_str()]).output().expect("Failed to replace settinga. ");
        process::Command::new(".venv/bin/python").args(["sec_gen.py"]).output().expect("Failed to run .env script. ");
        process::Command::new("mv").args([".env",env_path.as_str()]).output().expect("Failed to move .env into main app. ");
    }
    if cfg!(target_os = "windows"){
        process::Command::new("cmd").args(["/C","del","/Q","settings.tpl","sec_gen.py"]).spawn().expect("Failed to remove junk!");
    }
    else {
        process::Command::new("rm").args(["-f","settings.tpl","sec_gen.py"]).spawn().expect("Failed to remove junk!");
    }
    //templates
    create_dir("templates").expect("Failed to create 'templates'. ");
}

