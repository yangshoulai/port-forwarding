use crate::conf::Conf;
use std::{sync::RwLock, path::PathBuf};

mod conf;
mod error;
mod forwarding;

pub static BASE_PATH: RwLock<Option<PathBuf>> = RwLock::new(None);

#[tokio::main]
async fn main() {
    let exe_path = std::env::current_exe().unwrap();
    let exe_dir = exe_path.parent().unwrap();
    println!("Executed in dir: {}", exe_dir.display());
    match conf::get_conf_file().await {
        Ok(conf_file) => {
            let r: Result<Conf, String> = conf::load(conf_file.as_str()).await;
            if let Ok(conf) = r {
                forwarding::forward(conf.remote_mappings).await;
            } else {
                println!("{}", r.err().unwrap())
            }
        }
        Err(e) => {
            eprintln!("{e}");
        }
    }
}
