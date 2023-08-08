use std::{fs, io};
use serde::{Serialize, Deserialize};
use serde_yaml;
use platform_dirs::AppDirs;
use crate::error::ConfFileNotFoundError;
use crate::forwarding::Forwarding;

const CONF_DIR_NAME: &str = "develop-tools";
const CONF_FILE_NAME: &str = "conf.yaml";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Conf {
    pub remote_mappings: Vec<Forwarding>,
}

pub async fn get_conf_file() -> Result<String, ConfFileNotFoundError> {
    // 1. 程序执行目录下 conf.yaml
    let p = std::env::current_exe().unwrap();
    let p = p.parent().unwrap();
    let p = p.join(CONF_FILE_NAME);
    if p.exists() {
        return Ok(p.into_os_string().into_string().unwrap());
    }
    // 2. 获取用户目录 ~/.develop-tools/conf.yaml
    let p = get_app_dirs().config_dir.join(CONF_FILE_NAME);
    if p.exists() {
        return Ok(p.into_os_string().into_string().unwrap());
    }

    // 3. 获取src/conf.yaml
    let p = std::env::current_dir().unwrap().join("src").join(CONF_FILE_NAME);
    if p.exists() {
        return Ok(p.into_os_string().into_string().unwrap());
    }

    return Err(ConfFileNotFoundError::new("配置文件无法获取"));



}

pub async fn load<'a>(filename: &str) -> Result<Conf, String> {
    let f = fs::read_to_string(filename);
    if let Ok(y) = f {
        let r: serde_yaml::Result<Conf> = serde_yaml::from_str(&y);
        if let Ok(c) = r {
            Ok(c)
        } else {
            println!("{}", r.err().unwrap());
            Err("配置文件解析出错".to_string())
        }
    } else {
        let k: io::Error = f.err().unwrap();
        match k.kind() {
            io::ErrorKind::NotFound => Err(String::from("配置文件不存在")),
            _ => {
                println!("{k}");
                Err(String::from("配置文件加载出错"))
            }
        }
    }
}

pub fn get_app_dirs() -> AppDirs {
    AppDirs::new(Some(CONF_DIR_NAME), true).unwrap()
}
