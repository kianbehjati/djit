use anyhow::{self, Context};
use handlebars;
use reqwest::blocking::get;
use serde_json::{self, json};
use std::{fs::{self}, process};
use crate::errors;

pub enum DB_Type {
    Postgresql,
    Mysql,
}
#[derive(Debug)]
pub struct Tag {
    pub version: String,
    pub tag_status: String,
}

#[derive(serde::Serialize)]
pub struct DB_Options {
    pub db_name: String,
    pub db_password: String,
    pub db_user: String
}
pub fn get_python() -> anyhow::Result<Vec<Tag>> {
    let page_size: u8 = 15;
    let python_url = format!(
        "https://hub.docker.com/v2/repositories/library/python/tags?page_size={}",
        page_size
    );

    let response = get(python_url)?;
    let mut tags: Vec<Tag> = Vec::<Tag>::new();
    if response.status().is_success() {
        let js: serde_json::Value = serde_json::from_str(response.text()?.as_str())?;
        for res in js["results"]
            .as_array()
            .ok_or(errors::ManagerError::Value("as_array failed".into()))?
        {
            tags.push(Tag {
                version: res["name"].to_string(),
                tag_status: res["tag_status"].to_string(),
            });
        }
    } else {
        return Err(errors::ManagerError::Network("Can't reach DockerHub".into()).into());
    }
    return Ok(tags);
}

pub fn get_db(db: DB_Type) -> anyhow::Result<Vec<Tag>> {
    let page_size: u8 = 10;

    let db_name = match db {
        DB_Type::Postgresql => "postgres",
        DB_Type::Mysql => "mysql",
    };

    let db_url = format!(
        "https://hub.docker.com/v2/repositories/library/{}/tags?page_size={}",
        db_name, page_size
    );

    let response = get(db_url)?;
    let mut tags: Vec<Tag> = Vec::<Tag>::new();
    if response.status().is_success() {
        let js: serde_json::Value = serde_json::from_str(response.text()?.as_str())?;
        for res in js["results"]
            .as_array()
            .ok_or(errors::ManagerError::Value("as_array failed".into()))?
        {
            tags.push(Tag {
                version: res["name"].to_string(),
                tag_status: res["tag_status"].to_string(),
            });
        }
    } else {
        return Err(errors::ManagerError::Network("Can't reach DockerHub".into()).into());
    }
    return Ok(tags);
}

pub fn start_docker(python: Tag, db: Option<DB_Type>, db_tag: Option<Tag>, db_option: Option<DB_Options>) -> anyhow::Result<()> {
    /*
        to do:
            - health check django project
    */

    let mut handlebars = handlebars::Handlebars::new();
    
    let compose_tpl = include_str!("docker-compose.tpl");
    let dockerfile_tpl = include_str!("Dockerfile.tpl");

    let mut is_postgres:bool = false;
    let mut is_mysql:bool = false;
    
    let use_db = match db {
        Some(db_type) => {
            match db_type {
                DB_Type::Mysql => {
                    is_mysql = true;
                    "mysql"
                },
                DB_Type::Postgresql => {
                    is_postgres = true;
                    "postgres"
                }
            }
        },
        None => ""
    };
    let db_tag = match db_tag {
        Some(tag) => tag.version,
        None => "".into()
    };
    
    //handling docker-compose
    let compose_file = fs::File::create("docker-compose.yml")
        .context("can't create docker-compose.tpl in your project dir")?;
    fs::write("docker-compose.tpl", compose_tpl)
        .context("can't write docker-compose.tpl in your project dir")?;
    handlebars.register_template_file("compose", "./docker-compose.tpl")
        .context("Failed to create template from docker-compise.tpl in handlebars")?;
    handlebars.render_to_write("compose",
        &json!({
            "use_db" : use_db,
            "db_tag" : db_tag,
            "is_postgres": is_postgres,
            "is_mysql": is_mysql,
            "db_option": db_option.unwrap_or(DB_Options { db_name: "".into() , db_password: "".into() , db_user: "".into() })
        }),
        &compose_file
    )?;

    //handling Dockerfile
    let docker_file = fs::File::create("Dockerfile")
        .context("can't create Dockerfile in your project dir")?;
    fs::write("Dockerfile.tpl", dockerfile_tpl)
        .context("can't write Dockerfile.tpl in your project dir")?;
    handlebars.register_template_file("dockerfile", "./Dockerfile.tpl")
        .context("Failed to create template from Dockerfile.tpl in handlebars")?;
    handlebars.render_to_write("dockerfile",
        &json!({
            "python_version" : python.version
        }),
        &docker_file
    )?;

    //clean
    fs::remove_file("./docker-compose.tpl").context("can't clean docker-compose.tpl")?;
    fs::remove_file("./Dockerfile.tpl").context("can't clean Dockerfile.tpl")?;

    return Ok(());
}
