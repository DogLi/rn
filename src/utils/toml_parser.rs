use toml;
use errors::*;
use std::fmt::Debug;
use std::path::Path;
use utils::util::load_file;
use std::cmp::PartialEq;
use shellexpand::tilde;
use std::env::current_dir;



#[derive(Debug, Deserialize)]
pub struct GlobalConfig {
    pub global_user: String,
    pub global_password: Option<String>,
    pub global_key: Option<String>,
    pub global_port: Option<u16>,
    pub global_dest_root: String,
    pub global_exclude: Option<Vec<String>>,
    pub projects: Option<Vec<Project>>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct Project {
    pub name: String,
    pub src: String,
    pub dest: String,
    pub exclude: Option<Vec<String>>,
}

pub fn get_config(toml_path: &Path) -> Result<GlobalConfig> {
    // get the project settings config from *.toml file
    let toml_string = load_file(toml_path)?;
    let mut g_config: GlobalConfig = toml::from_str(toml_string.as_str())?;
    // change ~ into $HOME in the key
    g_config.global_key = match g_config.global_key {
        None => None,
        Some(key) => Some(tilde(&key).into_owned()),
    };

    Ok(g_config)
}

fn get_project_current_path(config: &GlobalConfig) -> Result<Project> {
    let current_path = current_dir()?;
    let dir_name = current_path.file_name().unwrap();
    let dir_name = dir_name.to_string_lossy();
    let dest_dir = Path::new(&config.global_dest_root).join(dir_name.to_string());
    let src: String = current_path.to_str().unwrap().to_string();
    let dest: String = dest_dir.to_str().unwrap().to_string();
    let project = Project {
        name: ".".to_string(),
        src,
        dest,
        exclude: config.global_exclude.clone(),
    };
    Ok(project)
}

pub fn get_project_info<S>(project_name: S, config: &GlobalConfig) -> Result<Project>
where
    S: AsRef<str> + Debug + PartialEq,
{
    if project_name.as_ref() == "." {
        return get_project_current_path(config)
    } else if config.projects.is_some() {
        for project in config.projects.as_ref().unwrap() {
            if project.name == project_name.as_ref() {
                let mut info = project.clone();
                info.src = tilde(&info.src).into_owned();
                return Ok(info);
            }
        }
    }
    bail!(format!("Can not find project {:?}", project_name));
}



#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::prelude::*;

    #[test]
    fn test_get_config() {
        let tmp_path = Path::new("/tmp/a.toml");
        let mut tmp_file = File::create(&tmp_path).unwrap();
        let content = r##"global_user = "root"
global_key = "~/.ssh/id_rsa"
#global_password = "nogame"

[[projects]]
name = "default"
src = "~/Desktop/cloud/"
dest = "~/qdata-cloud/"
exclude = [".git", "prometheus.yaml"]
"##;
        tmp_file.write_all(content.as_bytes()).unwrap();
        let global_config = get_config(tmp_path).unwrap();

        let project_name = "default";
        let project = get_project_info(project_name, &global_config).unwrap();
        assert_eq!(
            project,
            Project {
                name: "default".to_string(),
                src: tilde("~/Desktop/cloud/").into_owned(),
                dest: "~/qdata-cloud/".to_string(),
                exclude: Some(vec![".git".to_string(), "prometheus.yaml".to_string()]),
            }
        )
    }
}
