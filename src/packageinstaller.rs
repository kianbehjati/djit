// install packages here 
// to_do : install packages need for db 
use anyhow::{self, Context};
use std::process;
pub fn install(packages: Vec<&str>, is_uv:bool) -> anyhow::Result<()>{
    let mut packages_str = String::new();
    
    for package in packages {
        packages_str.push_str(&format!(" {}",package));
    }
    if is_uv{
        if cfg!(target_os = "windows"){
            let install_out = process::Command::new("cmd")
                .args(["/C",format!("uv pip install{}",&packages_str.as_str()).as_str()])
                .output()
                .context("failed to install packages")?;
        }else {
            let install_out = process::Command::new("sh")
                .args(["-c",format!("uv pip install{}",&packages_str.as_str()).as_str()])
                .output()
                .context("failed to install packages")?;
        }
    }
    else {
        if cfg!(target_os = "windows"){
            let install_out = process::Command::new("cmd")
                .args(["/C",format!(".venv\\Scripts\\pip.exe install{}",&packages_str.as_str()).as_str()])
                .output()
                .context("failed to install packages")?;
        }else {
            let install_out = process::Command::new("sh")
                .args(["-c",format!("./venv/bin/pip install{}",&packages_str.as_str()).as_str()])
                .output()
                .context("failed to install packages")?;
        }
    }

    return Ok(());
}

pub fn requirements(is_uv: bool) -> anyhow::Result<()> {
    if is_uv {
        if cfg!(target_os = "windows"){
            process::Command::new("cmd")
                .args(["/C","uv pip freeze > requirements.txt"])
                .output()
                .context("failed to create requirements.txt with uv")
                ?;
        } else {
            process::Command::new("sh")
                .args(["-c","uv pip freeze > requirements.txt"])
                .output()
                .context("failed to create requirements.txt with uv")?;
        }
    } else {
        if cfg!(target_os = "windwos"){
            process::Command::new("cmd")
                .args([".venv\\Scripts\\pip.exe","freeze > requirements.txt"])
                .output()
                .context("failed to create requirements.txt")?;
        } else {
            process::Command::new("sh")
                .args(["-c",".venv/bin/pip freeze > requirements.txt"])
                .output()
                .context("failed to create requirements.txt with uv")?;
        }
    }
    return Ok(());
}