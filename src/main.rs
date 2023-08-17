use crate::conf::Conf;

mod conf;
mod error;
mod forwarding;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let exe_path = std::env::current_exe()?;
    let exe_dir = exe_path.parent().unwrap();
    println!("Executed in dir: {}", exe_dir.display());
    match conf::get_conf_file().await {
        Ok(conf_file) => {
            let r: Result<Conf, anyhow::Error> = conf::load(conf_file.as_str()).await;
            if let Ok(conf) = r {
                forwarding::forward(conf.remote_mappings).await;
            } else {
                println!("加载配置文件失败: {}", r.err().unwrap())
            }
        }
        Err(e) => {
            eprintln!("{e}");
        }
    };
    Ok(())
}

