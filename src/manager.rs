use std::io::{Read, Write};
use crate::parser::DjangoOptions;
use dirs;
use serde_json::{self, Value};
use chrono::{NaiveDateTime};
use std::path::PathBuf;

pub struct DjangoProject {
    pub django_options : DjangoOptions,
    pub date : NaiveDateTime,
    pub description : String,
    pub path: PathBuf
}

pub fn list() -> Value{
    let mut list: Vec<DjangoProject> = Vec::new();
    // read from file and push to list
    
    let path = dirs::home_dir()
    .unwrap()
    .join(".djit")
    .join("projects.json");

    let mut file = std::fs::File::open(&path).unwrap();
    let mut string: String = String::new();
    file.read_to_string(&mut string).unwrap();

    /*
        projects.json : 
            {
	
                "projects":["webapp","otherapp"],

                "webapp":{
                        "path":"C:\\Users\\test\\Desktop\\codes\\webapp",
                        "apps":"test1,test2",
                        "date":"2012-09-03 23:56:04",
                        "description":"Shitty"
                    },
                "otherapp":{
                        "path":"C:\\Users\\test\\Desktop\\codes\\otherapp",
                        "apps":"test3,test4",
                        "date":"2015-09-05 23:56:04",
                        "description":"not bad"
                }
                ...
            }
     */

    let mut projects:serde_json::Value = serde_json::from_str(&string).unwrap();
    for project in projects["projects"].as_array().unwrap() {
        let project_data = projects[project.as_str().unwrap()].clone();
        let django_project = DjangoProject {
            django_options: DjangoOptions {
                name: project.as_str().unwrap().to_string(),
                apps: project_data["apps"].as_str().unwrap_or("").to_string(),
            },
            date: NaiveDateTime::parse_from_str(project_data["date"].as_str().unwrap(),"%Y-%m-%d %H:%M:%S").unwrap(),
            description: project_data["description"].as_str().unwrap_or("").into(),
            path: PathBuf::from(project_data["path"].as_str().unwrap())
            
        };
        list.push(django_project);
    }

    return projects;
} 

pub fn save(django_option: DjangoOptions, description: String, path: PathBuf, projects: &mut Value) {
    
    let new_project = DjangoProject {
        django_options: django_option,
        date: chrono::Local::now().naive_local(),
        description,
        path
    };
    
    projects["projects"].as_array_mut().unwrap().push(Value::String(new_project.django_options.name.clone()));    
    projects[new_project.django_options.name.clone()] = serde_json::json!({
        "path": new_project.path.to_str().unwrap(),
        "apps": new_project.django_options.apps,
        "date": new_project.date.format("%Y-%m-%d %H:%M:%S").to_string(),
        "description": new_project.description
    });

    // save to file
    let path = dirs::home_dir()
    .unwrap()
    .join(".djit")
    .join("projects.json");

    let mut file = std::fs::File::create(&path).unwrap();
    let json_string = serde_json::to_string(&projects).unwrap();
    file.write(json_string.as_bytes()).unwrap();
}