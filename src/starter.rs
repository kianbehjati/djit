use crate::check;
use crate::errors;
use crate::manager;
use crate::parser::DjangoOptions;
use anyhow::{self, Context};
use handlebars::Handlebars;
use serde_json::json;
use std::fs::{File, create_dir, write};
use std::path::Path;
use std::path::PathBuf;
use std::process;

pub fn starter(res: DjangoOptions, description: String, path: String) -> anyhow::Result<()> {
    //checking Internet Connection
    if !(check::checker()) {
        return Err(errors::ManagerError::Network.into());
    };

    let apps: Vec<_> = res.apps.split(",").collect();
    let mut is_uv: bool = false;

    // embeding files into binary
    let sec_gen = include_str!("sec_gen.py");
    let settings_tpl = include_str!("settings.tpl");

    std::env::set_current_dir(std::path::Path::new(&path)).context(format!(
        "Can't switch path(does not exists or a permission issue) to {}",
        path
    ))?;

    //write templates in given path
    write("sec_gen.py", sec_gen)
        .context("can't write 'sec_gen.py in selected dir(check permissions)'")?;
    write("settings.tpl", settings_tpl)
        .context("can't write 'settings.tpl in selected dir(check permissions)'")?;

    // look for uv & python
    if cfg!(target_os = "windows") {
        //check uv
        let process_c = process::Command::new("powershell")
            .args(["-c", "(Get-command uv).Path"])
            .output()
            .context("powershell failed")?;
        match String::from_utf8_lossy(&process_c.stdout).len() {
            0 => {
                println!("uv not found :(");
            }
            _ => {
                println!("uv installed :>");
                is_uv = true;
            }
        }
        //check python
        let python_path = process::Command::new("powershell")
            .args(["-c", "(Get-command python).Path"])
            .output()
            .context("powershell failed")?;
        match String::from_utf8_lossy(&python_path.stdout).len() {
            0 => {
                println!("python not installed.");
                return Err(errors::ManagerError::PythonNotInstalled.into());
            }
            _ => (),
        }
    } else {
        let process_c = process::Command::new("which")
            .args(["uv"])
            .output()
            .context("bash shell failed")?;
        match String::from_utf8_lossy(&process_c.stdout).len() {
            0 => {
                println!("uv not found :(");
            }
            _ => {
                println!("uv installed :>");
                is_uv = true;
            }
        }

        let python_path = process::Command::new("which")
            .args(["python3"])
            .output()
            .context("bash shell failed")?;
        match String::from_utf8_lossy(&python_path.stdout).len() {
            0 => {
                println!("python not installed.");
                return Err(errors::ManagerError::PythonNotInstalled.into());
            }
            _ => (),
        }
    }

    // init command chain

    if is_uv {
        println!("Creating virtual env with uv...");
        process::Command::new("uv")
            .args(["venv", ".venv"])
            .output()
            .context("venv failed")?;
        println!("Installing django with uv...");
        if cfg!(target_os = "windows") {
            process::Command::new("cmd")
                .args(["/C", "uv pip install django python-dotenv"])
                .output()
                .context("venv failed")?;
        } else {
            process::Command::new("uv")
                .args(["pip", "install", "django", "python-dotenv"])
                .output()
                .context("venv failed")?;
        }
    } else {
        println!("Creating virtual env...");
        if cfg!(target_os = "windows") {
            process::Command::new("python")
                .args(["-m", "venv", ".venv"])
                .output()
                .context("venv failed")?;
            process::Command::new("cmd")
                .args(["/C", ".venv\\Scripts\\pip.exe install django python-dotenv"])
                .output()
                .context("venv failed")?;
        } else {
            process::Command::new("python3")
                .args(["-m", "venv", ".venv"])
                .output()
                .context("venv failed")?;
            println!("Installing django...");
            process::Command::new(".venv/bin/pip")
                .args(["install", "django", "python-dotenv"])
                .output()
                .context("venv failed / make sure that you have python3-venv installed !!!")?; // will get an error bc python -m venv doesn't work in linux
        }
    }

    println!("Creating Django Project...");
    if cfg!(target_os = "windows") {
        process::Command::new("cmd")
            .args([
                "/C",
                ".venv\\Scripts\\python.exe -m django startproject",
                res.name.as_str(),
                ".",
            ])
            .output()
            .context("Creating Django App Failed.")?;
    } else {
        process::Command::new(".venv/bin/python")
            .args(["-m", "django", "startproject", res.name.as_str(), "."])
            .output()
            .context("Creating Django App Failed.")?;
    }

    let mut apps_str = String::new();

    if apps.len() > 1 {
        println!("Creating Apps...");
        for app in apps {
            apps_str.push_str(&format!(", \"{}\"", app));
            if cfg!(target_os = "windows") {
                process::Command::new("cmd")
                    .args(["/C", ".venv\\Scripts\\python.exe -m django startapp", app])
                    .spawn()
                    .context("Starting App(s) Failed.")?;
            } else {
                process::Command::new(".venv/bin/python")
                    .args(["-m", "django", "startapp", app])
                    .spawn()
                    .context("Starting App(s) Failed.")?;
            }
        }
    }

    //handlebars template rendering
    let settings = File::create("settings.py").context("Failed to Create settings.py file.")?;
    let mut hb = Handlebars::new();
    hb.register_template_file("template", "./settings.tpl")
        .context("Failed to create template from settings.tpl in handlebars")?;
    hb.render_to_write(
        "template",
        &json!({"app_name" : res.name.as_str(),"apps" : apps_str}),
        &settings,
    )
    .context("Handlebars failed to render to write template")?;

    let settings_path: String;
    let env_path: String;
    let settings_backup: String;

    if cfg!(target_os = "windows") {
        settings_path = res.name.clone() + "\\" + "settings.py";
        env_path = res.name.clone() + "\\" + ".env";
        settings_backup = res.name.clone() + "\\" + "settings.bkp.py";
    } else {
        settings_path = res.name.clone() + "/" + "settings.py";
        env_path = res.name.clone() + "/" + ".env";
        settings_backup = res.name.clone() + "/" + "settings.bkp.py";
    }

    if cfg!(target_os = "windows") {
        process::Command::new("cmd")
            .args([
                "/C",
                "copy",
                settings_path.as_str(),
                settings_backup.as_str(),
            ])
            .output()
            .context("Failed to copy backup of original settings.")?;
        process::Command::new("cmd")
            .args(["/C", "move", "settings.py", settings_path.as_str()])
            .output()
            .context("Failed to replace settings.")?;
        process::Command::new("cmd")
            .args(["/C", ".venv\\Scripts\\python.exe sec_gen.py"])
            .output()
            .context("Failed to run .env script.")?;
        process::Command::new("cmd")
            .args(["/C", "move", ".env", env_path.as_str()])
            .output()
            .context("Failed to move .env into main app.")?;
    } else {
        process::Command::new("cp")
            .args([settings_path.as_str(), settings_backup.as_str()])
            .output()
            .context("Failed to copy backup of original settings.")?;
        process::Command::new("mv")
            .args(["settings.py", settings_path.as_str()])
            .output()
            .context("Failed to replace settings.")?;
        process::Command::new(".venv/bin/python")
            .args(["sec_gen.py"])
            .output()
            .context("Failed to run .env script.")?;
        process::Command::new("mv")
            .args([".env", env_path.as_str()])
            .output()
            .context("Failed to move .env into main app.")?;
    }
    if cfg!(target_os = "windows") {
        process::Command::new("cmd")
            .args(["/C", "del", "/Q", "settings.tpl", "sec_gen.py"])
            .spawn()
            .context("Failed to remove junk!")?;
    } else {
        process::Command::new("rm")
            .args(["-f", "settings.tpl", "sec_gen.py"])
            .spawn()
            .context("Failed to remove junk!")?;
    }
    //templates
    create_dir("templates").context("Failed to create 'templates'.")?;

    //save to projects.json
    match manager::list() {
        Ok(mut p) => {
            match manager::save(res, description, PathBuf::from(path), &mut p) {
                Ok(()) => {}
                Err(e) => errors::error_printer(e),
            };
        }
        Err(e) => {
            errors::error_printer(e);
        }
    }
    return Ok(());
}
