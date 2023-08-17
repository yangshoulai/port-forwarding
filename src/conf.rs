use std::fs;
use std::fmt::{Debug, Display};

use platform_dirs::AppDirs;
use serde::{Deserialize, Serialize};
use serde_yaml;

use crate::error::ConfFileNotFoundError;
use crate::forwarding::Forwarding;

const CONF_DIR_NAME: &str = "develop-tools";
const CONF_FILE_NAME: &str = "conf.yaml";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Conf {
    pub remote_mappings: Vec<Forwarding>,
}

pub async fn get_conf_file() -> Result<String, ConfFileNotFoundError> {
    if cfg!(debug_assertions) {
        // 调试模式 获取src/conf.yaml
        let p = std::env::current_dir().unwrap().join("src").join(CONF_FILE_NAME);
        if p.exists() {
            return Ok(p.into_os_string().into_string().unwrap());
        }
        return Err(ConfFileNotFoundError::new("没有找到配置文件"));
    } else {
        // 1. 程序执行目录下 conf.yaml
        let p = std::env::current_exe().unwrap();
        let p = p.parent().unwrap();
        let p = p.join(CONF_FILE_NAME);
        let f1 = p.clone();
        if p.exists() {
            return Ok(p.into_os_string().into_string().unwrap());
        }
        // 2. 获取用户目录 ~/.develop-tools/conf.yaml
        let p = get_app_dirs().config_dir.join(CONF_FILE_NAME);
        let f2 = p.clone();
        if p.exists() {
            return Ok(p.into_os_string().into_string().unwrap());
        }
        return Err(ConfFileNotFoundError::new(format!("没有找到配置文件，{} -> {}", f1.display(), f2
            .display())
            .as_str()));
    }
}


pub async fn load<'a>(filename: &str) -> Result<Conf, anyhow::Error> {
    let f = fs::read_to_string(filename)?;
    let r = serde_yaml::from_str(&f)?;
    Ok(r)
}

pub fn get_app_dirs() -> AppDirs {
    AppDirs::new(Some(CONF_DIR_NAME), true).unwrap()
}