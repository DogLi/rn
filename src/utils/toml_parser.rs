use toml;
use errors::*;
use std::fmt::Debug;
use std::path::Path;
use serde::de::DeserializeOwned;
use utils::util::load_file;
use std::cmp::PartialEq;
use shellexpand::tilde;


#[derive(Debug, Deserialize)]
pub struct global_config
{
    pub global_user: String,
    pub global_password: Option<String>,
    pub global_key: Option<String>,
    pub global_port: Option<u16>,
    pub projects: Option<Vec<project>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct project{
    pub name: String,
    pub src: String,
    pub dest: String,
    pub exclude: Option<Vec<String>>,
}

pub fn get_config<P>(toml_path: P) -> Result<global_config>
where P: AsRef<Path>
{
    // get the project settings config from *.toml file
    let toml_string = load_file(toml_path)?;
    let mut g_config: global_config = toml::from_str(toml_string.as_str())?;
    g_config.global_key = match g_config.global_key{
        None => None,
        Some(key) => Some(tilde(&key).into_owned())
    };
    Ok(g_config)
}


pub fn get_project_info<S>(project_name: S, config: &global_config) -> Option<project>
    where S: AsRef<str> + Debug + PartialEq{
    let mut result:Option<project> = None;

    match config.projects {
        None => None,
        Some(ref projects) => {
            let mut iter = projects.iter();
            while let Some(project) = iter.next() {
                if project.name == project_name.as_ref(){
                    let mut info = project.clone();
                    info.src = tilde(&info.src).into_owned();
                    return Some(info);
                }
            }
            None
        }
    }
}
