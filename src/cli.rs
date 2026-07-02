use std::io::{self, Write};
use std::path::{Path, PathBuf};
use clap::{Parser, Subcommand};
use owo_colors::OwoColorize;
use owo_colors::colors::css::Black;
use owo_colors::colors::xterm::FernGreen;
use owo_colors::colors::*;
use anyhow::{Context};
use crate::{docker, errors};
use crate::docker::{DB_Options, DB_Type, Tag};
use rfd;
use crate::starter;
use crate::manager;
#[derive(Parser)]
#[command(name = "djit")]
#[command(about = "Djit automates Django project setup, so you can start building instead of configuring.", long_about = None)]
struct CLI {
    /// get more detailed output of process
    #[arg(short, long)]
    verbose: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a Django Project
    Start,
    /// Manage your Django Projects(Created With Djit)
    Manage
}
pub fn cli() {
    /*
    todo
        - follow the user flow ✅
        - before each action check requirements(e.g docker running, existence of projects.json, etc.) ✅
        - askfileopen for path in starter ✅
        - define stacks(minimal, worker, api) 
        - make cli modern and beautiful(indicatif) 
    */
    let cli_obj = CLI::parse();
    match cli_obj.command {
        Some(command) => {
            match command {
                Commands::Manage => {manager();}
                Commands::Start => {
                    match starter() {
                        Ok(()) => {}
                        Err(e) => {errors::error_printer(e);}
                        };}
            }
        }
        None => {panic!("Please select a <Command> first (e.g djit.exe start)")}
    }
}

fn manager() -> anyhow::Result<()> {
    loop {
        let mut input = String::new();
        print!("Select an Option to start[({})ist, ({})elete, ({})uit]: ","L".fg::<BrightBlue>().bold().underline(),"D".fg::<Red>().bold() ,"Q".fg::<BrightMagenta>().bold());
        io::stdout().flush()?;
        io::stdin().read_line(&mut input)?;
        match input.trim().to_lowercase().as_str() {
            "l" => {
                match manager::list() {
                    Ok(projects) => {
                        println!("Projects:");
                        for project in projects["projects"].as_array().ok_or(errors::ManagerError::Value("As Array Failed".into()))? {
                            let project_data = projects[project
                                .as_str()
                                .ok_or(errors::ManagerError::Value("as_str() failed".to_string()))?]
                                .clone();
                            println!("Name: {}", project
                                .as_str()
                                .ok_or(errors::ManagerError::Value("as_str() failed".to_string()))?
                                .to_string().fg::<Blue>());
                            println!("Apps: {}", project_data["apps"]
                                .as_str()
                                .ok_or(errors::ManagerError::Value("as_str() failed".to_string()))?
                                .to_string().fg::<Cyan>());
                            println!("description: {}",project_data["description"]
                                .as_str()
                                .ok_or(errors::ManagerError::Value("as_str() failed".to_string()))?
                                .to_string().fg::<Green>());
                            println!("Path: {}", project_data["path"]
                                .as_str()
                                .ok_or(errors::ManagerError::Value("as_str() failed".to_string()))?
                                .to_string().fg::<Yellow>());
                            println!("Date: {}", project_data["date"]
                                .as_str()
                                .ok_or(errors::ManagerError::Value("as_str() failed".to_string()))?
                                .to_string().fg::<Magenta>());
                            println!("---------------------------");
                        }
                    }
                    Err(e) => {errors::error_printer(e);}
                }
            },
            "d" => {
                let mut project_name = String::new();
                let mut project_path = String::new();
                let mut _permanent = String::new();
                let delete_permanent: bool;
                print!("{} of the project you want to {}: ","Name".fg::<Blue>().underline().italic(),"Delete".fg::<Red>().bold().underline());
                io::stdout().flush()?;
                io::stdin().read_line(&mut project_name)?;

                print!("{} of the project you want to {}: ","Path".fg::<Yellow>().underline().italic(),"Delete".fg::<Red>().bold().underline());
                io::stdout().flush()?;
                io::stdin().read_line(&mut project_path)?;

                print!("Do you want to delete the project permanently? ({}/{}): ","y".fg::<Red>().bold().underline(),"n".fg::<Green>());
                io::stdout().flush()?;
                io::stdin().read_line(&mut _permanent)?;
                match _permanent
                    .trim()
                    .to_lowercase()
                    .chars()
                    .next()
                    .unwrap_or('d') {
                        'y' => {delete_permanent = true;}
                        'n' => {delete_permanent = false;}
                        _ => {panic!("Invalid input for delete permanently, type y or n.")}
                    } 
                match manager::list() {
                    Ok(mut projects) => {
                        match manager::delete(project_name.trim().into(),PathBuf::from(project_path.trim()),&mut projects,delete_permanent) {
                            Ok(()) => {println!("Project deleted successfully.");}
                            Err(e) => {errors::error_printer(e);}
                        }
                    }
                    Err(e) => {errors::error_printer(e);}
                }
            }
            "q" => {
                println!("{}","Exiting...".fg::<BrightMagenta>().bold().underline()); 
                break;
            }
            _ => {println!("Invalid option. Please select (L), (D), or (Q).");}
        }
    }
    return Ok(());
}
fn starter() -> anyhow::Result<()>{
    let mut project_name: String = String::new();
    let mut apps: String = String::new();
    let mut description = String::new();
    let mut db: Option<DB_Type>;
    let mut use_docker: bool = false;
    let mut path: PathBuf = PathBuf::new();
    let mut db_option = DB_Options {db_name: String::new(), db_user: String::new(), db_password: String::new()};
    let mut db_host: String = String::new();
    let mut python_tag: Tag = Tag { version: String::new(), tag_status: String::new() };
    let mut db_tag: Tag = Tag { version: String::new(), tag_status: String::new() };
    // check for projects.json
    let config_path = dirs::home_dir()
        .ok_or(errors::ManagerError::HomePathNotFound)?
        .join(".djit")
        .join("projects.json");
    if !(config_path.exists()) {
        std::fs::create_dir_all(config_path.parent().unwrap())
            .context("Failed to create .djit directory in home dir")?;
        std::fs::File::create(&config_path)
            .context("Failed to create projects.json file in .djit directory")?;
        std::fs::write(&config_path, "{\"projects\": []}")
            .context("Failed to write initial content to projects.json file")?;
    };
    //project name 
    print!("Project name[{}]: ","a-z".fg::<BrightBlue>());
    io::stdout().flush()?;
    io::stdin().read_line(&mut project_name)?;

    if !(is_valid_name(&project_name.trim())) {
        panic!("{} is not a valid django project/app name",project_name.trim().fg::<Red>());
    }

    //decription 
    print!("Project Description[{}]: ","Optional".fg::<BrightMagenta>());
    io::stdout().flush()?;
    io::stdin().read_line(&mut description)?;

    //apps
    print!("Apps[{},{},...]: ","users".fg::<Green>().dimmed(),"shop".fg::<Cyan>().dimmed());
    io::stdout().flush()?;
    io::stdin().read_line(&mut apps)?;
    for app in apps.trim().split(",").collect::<Vec<&str>>() {
        if app.len() == 0 {
                continue;
        };
        if !(is_valid_name(app)) {
            panic!("{} is not a valid django project name",app.fg::<Red>());
        }
    }

    //db
    print!("Which DB [{} ,{} ,{}]: ","Posgres(p)".fg::<Green>().bold(),"Mysql(m)".fg::<Blue>().italic(),"Sqlite(S)".fg::<Yellow>().underline());
    let mut db_type = String::new();
    io::stdout().flush()?;
    io::stdin().read_line(&mut db_type)?;
    
    db = match db_type
        .trim()
        .to_lowercase()
        .chars()
        .next()
        .unwrap_or(('d')) { // d for default
            'p' => Some(DB_Type::Postgresql),
            'm' => Some(DB_Type::Mysql),
            's' => None,
            'd' => None,
            _ => panic!("Invalid DB name")
        };
    
    match db.clone() {
        Some(d) => {
            //db host
            match d {
                DB_Type::Mysql =>  print!("DataBase Host{}: ","[localhost]".fg::<Blue>().bold().italic().underline()),
                DB_Type::Postgresql => print!("DataBase Host{}: ","[localhost]".fg::<Green>().bold().underline())
            }
            let mut host = String::new();
            io::stdout().flush()?;
            io::stdin().read_line(&mut host)?;
            if host.trim() == "" {
                db_host = String::from("localhost");
            }else {
                db_host = host.trim().to_string();
            }

            //db options
            match d {
                DB_Type::Mysql => {
                    print!("DataBase Name{}: ","[db]".fg::<Blue>().bold().italic().underline());
                    io::stdout().flush()?;
                    io::stdin().read_line(&mut db_option.db_name)?;
                    if db_option.db_name.trim() == "" {
                        db_option.db_name = String::from("db");
                    }
                    print!("DataBase User{}: ","[root]".fg::<Blue>().bold().italic().underline());
                    io::stdout().flush()?;
                    io::stdin().read_line(&mut db_option.db_user)?;
                    if db_option.db_user.trim() == "" {
                        db_option.db_user = String::from("root");
                    }
                    print!("DataBase Password{}: ","[password]".fg::<Blue>().bold().italic().underline());
                    io::stdout().flush()?;
                    io::stdin().read_line(&mut db_option.db_password)?;
                    if db_option.db_password.trim() == "" {
                        panic!("Password can't be empty for MysqlDB");
                    }
                }
                DB_Type::Postgresql => {
                    print!("DataBase Name{}: ","[db]".fg::<Green>().bold().underline());
                    io::stdout().flush()?;
                    io::stdin().read_line(&mut db_option.db_name)?;
                    if db_option.db_name.trim() == "" {
                        db_option.db_name = String::from("db");
                    }
                    print!("DataBase User{}: ","[postgres]".fg::<Green>().bold().underline());
                    io::stdout().flush()?;
                    io::stdin().read_line(&mut db_option.db_user)?;
                    if db_option.db_user.trim() == "" {
                        db_option.db_user = String::from("postgres");
                    }
                    print!("DataBase Password{}: ","[password]".fg::<Green>().bold().underline());
                    io::stdout().flush()?;
                    io::stdin().read_line(&mut db_option.db_password)?;
                    if db_option.db_password.trim() == "" {
                        panic!("Password can't be empty for PostgresDB");
                    }
                }
            }
        }   
        None => {}
    }
    //use_docker
    print!("Dockerize({}/{}): ","Y".fg::<Green>().bold().underline(),"n".fg::<Red>());
    let mut dockerize = String::new();
    io::stdout().flush()?;
    io::stdin().read_line(&mut dockerize)?;
    match dockerize
        .trim()
        .to_lowercase()
        .as_str() {
            // todo : merge the code for "y" and "" since they are the same
            "y" => {
                match docker::check_docker() {
                    Ok(()) => {},
                    Err(e) => return Err(e)
                }
                // python tag
                let pythons = match docker::get_python() {
                    Ok(tags) => tags,
                    Err(e) => return Err(e)
                };
                
                let mut tags_str = String::new();
                for tag in pythons {
                    tags_str.push_str(&format!("{}, ",tag.version));
                }
                print!("Select a Python version from dockerHub [{}]: ",tags_str.fg::<FernGreen>());
                
                io::stdout().flush()?;
                io::stdin().read_line(&mut python_tag.version)?;

                // db tag
                let dbs: Vec<Tag>;
                if db.clone().is_some() {
                    match db.clone().unwrap() {
                        DB_Type::Postgresql => {
                            dbs = match docker::get_db(DB_Type::Postgresql) {Ok(d) => d, Err(e) => return Err(e)};
                            let mut tags_str = String::new();
                            for tag in dbs {
                                tags_str.push_str(&format!("{}, ",tag.version));
                            }
                            print!("Select a PostgresDB version from dockerHub [{}]: ",tags_str.fg::<Green>());
                        }
                        DB_Type::Mysql => {
                            dbs = match docker::get_db(DB_Type::Mysql) {Ok(d) => d, Err(e) => return Err(e)};
                            let mut tags_str = String::new();
                            for tag in dbs {
                                tags_str.push_str(&format!("{}, ",tag.version));
                            }
                            print!("Select a Mysql version from dockerHub [{}]:",tags_str.fg::<Blue>());
                        }
                    }
                    io::stdout().flush()?;
                    io::stdin().read_line(&mut db_tag.version)?;
                }
                
                use_docker = true;
            }
            "n" => {use_docker = false;}
            "" => {
                match docker::check_docker() {
                    Ok(()) => {},
                    Err(e) => return Err(e)
                }
                let pythons = match docker::get_python() {
                    Ok(tags) => tags,
                    Err(e) => return Err(e)
                };
                let mut tags_str = String::new();
                for tag in pythons {
                    tags_str.push_str(&format!("{}, ",tag.version));
                }
                print!("Select a Python version from dockerHub [{}]: ",tags_str.fg::<FernGreen>());
                io::stdout().flush()?;
                io::stdin().read_line(&mut python_tag.version)?;

                // db tag
                let dbs: Vec<Tag>;
                if db.clone().is_some() {
                    match db.clone().unwrap() {
                        DB_Type::Postgresql => {
                            dbs = match docker::get_db(DB_Type::Postgresql) {Ok(d) => d, Err(e) => return Err(e)};
                            let mut tags_str = String::new();
                            for tag in dbs {
                                tags_str.push_str(&format!("{}, ",tag.version));
                            }
                            print!("Select a PostgresDB version from dockerHub [{}]: ",tags_str.fg::<Green>());
                        }
                        DB_Type::Mysql => {
                            dbs = match docker::get_db(DB_Type::Mysql) {Ok(d) => d, Err(e) => return Err(e)};
                            let mut tags_str = String::new();
                            for tag in dbs {
                                tags_str.push_str(&format!("{}, ",tag.version));
                            }
                            print!("Select a Mysql version from dockerHub [{}]: ",tags_str.fg::<Blue>());
                        }
                    }
                    io::stdout().flush()?;
                    io::stdin().read_line(&mut db_tag.version)?;
                }
                
                use_docker = true;
            }
            _ => panic!("Invalid input for Dockerize, type y or n.")
        }

    //select path
    println!("Please select a {} for project","directory".fg::<Blue>().bold().underline().italic());
    let file = rfd::FileDialog::new()
        .set_directory("/")
        .pick_folder();
    match file {
        Some(p) => path = p,
        None => {panic!("Can't Open Folder")}
    }
    println!("Selected path : {}",path.to_str().unwrap().fg::<Black>().underline().italic());

    match starter::starter(
        crate::parser::DjangoOptions { name: project_name.trim().into(), apps: apps.trim().into() }, 
        description.trim().into(), 
        path.to_str().unwrap().to_string(), 
        use_docker, 
        db.clone(), 
        if db.is_some() {Some(db_option)} else {None}, 
        db_host, 
        python_tag,
        Some(db_tag) 
        ){
            Ok(()) => {},
            Err(e) => return Err(e)
    }
    return Ok(());
}
fn is_valid_name(name: &str) -> bool {
    let mut chars = name.chars();

    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
        _ => return false,
    }

    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}
