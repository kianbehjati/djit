use crate::errors;
use crate::parser::DjangoOptions;
use anyhow::{self, Context, Ok};
use chrono::NaiveDateTime;
use dirs;
use serde_json::{self, Value};
use std::fs;
use std::io::{Read, Write};
use std::path::PathBuf;
use trash;
pub struct DjangoProject {
    pub django_options: DjangoOptions,
    pub date: NaiveDateTime,
    pub description: String,
    pub path: PathBuf,
}

pub fn list() -> anyhow::Result<Value> {
    let mut list: Vec<DjangoProject> = Vec::new();
    // read from file and push to list

    let path = dirs::home_dir()
        .ok_or(errors::ManagerError::HomePathNotFound)?
        .join(".djit")
        .join("projects.json");
    let mut file = std::fs::File::open(&path).context("can't open projects.json")?;
    let mut string: String = String::new();
    file.read_to_string(&mut string)
        .context("stream is not valid UTF-8")?;

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

    let mut projects: serde_json::Value =
        serde_json::from_str(&string).context("Str to Json failed with serde_json")?;

    for project in projects["projects"]
        .as_array()
        .ok_or(errors::ManagerError::Value("as_array() failed".to_string()))?
    {
        let project_data = projects[project
            .as_str()
            .ok_or(errors::ManagerError::Value("as_str() failed".to_string()))?]
        .clone();
        let django_project = DjangoProject {
            django_options: DjangoOptions {
                name: project.as_str().context("as_str() failed")?.to_string(),
                apps: project_data["apps"]
                    .as_str()
                    .unwrap_or("Can't get apps...")
                    .to_string(),
            },
            date: NaiveDateTime::parse_from_str(
                project_data["date"]
                    .as_str()
                    .unwrap_or("0000-00-00 00:00:00"),
                "%Y-%m-%d %H:%M:%S",
            )?,
            description: project_data["description"]
                .as_str()
                .unwrap_or("Can't get description...")
                .into(),
            path: PathBuf::from(project_data["path"].as_str().unwrap_or("")),
        };
        list.push(django_project);
    }

    return Ok(projects);
}

pub fn save(
    django_option: DjangoOptions,
    description: String,
    path: PathBuf,
    projects: &mut Value,
) -> anyhow::Result<()> {
    let new_project = DjangoProject {
        django_options: django_option,
        date: chrono::Local::now().naive_local(),
        description,
        path,
    };

    if projects["projects"]
        .as_array()
        .ok_or(errors::ManagerError::Value("as_array() failed".to_string()))?
        .contains(&Value::String(new_project.django_options.name.clone()))
    {
        return Err(errors::ManagerError::Duplicate(
            new_project.django_options.name.clone().to_string(),
        )
        .into());
    };
    projects["projects"]
        .as_array_mut()
        .ok_or(errors::ManagerError::AsMutArrayFailed)?
        .push(Value::String(new_project.django_options.name.clone()));

    projects[new_project.django_options.name.clone()] = serde_json::json!({
        "path": new_project.path.to_str().unwrap(),
        "apps": new_project.django_options.apps,
        "date": new_project.date.format("%Y-%m-%d %H:%M:%S").to_string(),
        "description": new_project.description
    });

    // save to file
    let path = dirs::home_dir()
        .ok_or(errors::ManagerError::HomePathNotFound)?
        .join(".djit")
        .join("projects.json");

    let mut file = std::fs::File::create(&path)?;
    let json_string = serde_json::to_string(&projects)?;
    file.write(json_string.as_bytes())?;

    return Ok(());
}

pub fn delete(
    name: String,
    path: PathBuf,
    projects: &mut Value,
    permanent: bool,
) -> anyhow::Result<()> {
    // delete from list
    let index = projects["projects"]
        .as_array()
        .ok_or(errors::ManagerError::Value("as_array() failed".to_string()))?
        .iter()
        .position(|x| x.as_str() == Some(&name))
        .ok_or(errors::ManagerError::Index)?;

    if projects[&name]["path"].as_str() == path.to_str() {
        projects.as_object_mut().unwrap().remove(&name);
        projects["projects"].as_array_mut().unwrap().remove(index);
    }

    //delete from disk
    if permanent {
        fs::remove_dir_all(path)?
    } else {
        trash::delete(path)?;
    }

    // delete from file(projects.json)
    let path = dirs::home_dir()
        .ok_or(errors::ManagerError::HomePathNotFound)?
        .join(".djit")
        .join("projects.json");

    let mut file = std::fs::File::create(&path)?;
    let json_string = serde_json::to_string(&projects)?;
    file.write(json_string.as_bytes())?;

    return Ok(());
}
