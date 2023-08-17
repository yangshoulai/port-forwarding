use std::io::Error;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, TcpStream};
use std::str::FromStr;
use std::time::Duration;

use async_io::{Async, Timer};
use async_ssh2_lite::{AsyncChannel, AsyncSession};
use futures::{AsyncReadExt, AsyncWriteExt};
use futures_lite::FutureExt;
use serde::{Deserialize, Serialize};
use tokio::select;
use tokio::task::{JoinHandle, JoinSet};

use crate::error::ForwardingError;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Forwarding {
    pub ssh_host: String,
    pub ssh_password: String,
    pub ssh_username: String,
    pub ssh_port: u16,
    pub ports: Vec<Port>,
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Port {
    pub remote_host: String,
    pub remote_port: u16,
    pub local_port: u16,
    pub local_host: Option<String>,
    pub label: String,
}



pub async fn forward(forwardings: Vec<Forwarding>) {
    if forwardings.len() > 0 {
        let mut handles = Vec::with_capacity(10);
        for f in forwardings.iter() {
            if f.ports.len() > 0 {
                check_ssh_connection(f).await;
                for p in f.ports.iter() {
                    match bind_local_addr(&p).await {
                        Ok(listener) => {
                            let h = listen_local(listener, f.clone(), p.clone()).await;
                            handles.push(h);
                        }
                        Err(e) => {
                            eprintln!("{e}");
                        }
                    }
                }
            }
        }
        let mut set = JoinSet::new();
        for handle in handles {
            set.spawn(handle);
        }
        while let Some(Ok(Ok(r))) = set.join_next().await {
            if let Err(e) = r {
                eprintln!("{e}");
            }
        }
    }
}


async fn listen_local(listener: Async<TcpListener>, f: Forwarding, p: Port) ->
JoinHandle<Result<(), ForwardingError>> {
    let h: JoinHandle<Result<(), ForwardingError>> = tokio::spawn(async move {
        println!("[{:7}] [{:25}] [0.0.0.0:{:<5}] <=> [{:>15}:{:<5}] <=> [{:>15}:{:<5}]", "等待连接", p.label, p
            .local_port, f.ssh_host, f.ssh_port, p.remote_host, p.remote_port);
        loop {
            match listener.accept().await {
                Ok((local_stream, addr)) => {
                    match create_session(&f).await {
                        Ok(session) => {
                            match create_channel(&session, &p).await {
                                Ok(channel) => {
                                    println!("[{:7}] {:15} <=> 0.0.0.0:{:<5} <=> {:>15}:{:<5}",
                                             "已连接", addr, p.local_port, p.remote_host, p.remote_port);
                                    handle_local_connect(local_stream, channel).await;
                                }
                                Err(e) => {
                                    return Err(e);
                                }
                            }
                        }
                        Err(e) => {
                            return Err(e);
                        }
                    }
                }
                Err(e) => {
                    return Err(ForwardingError::ListeningError(format!("[{:7}] [{:25}] [0.0.0.0:{:<5}] [{e}]", "监听异常", p.label, p.local_port)));
                }
            }
        }
    });

    h
}

async fn bind_local_addr(p: &Port) -> Result<Async<TcpListener>, ForwardingError> {
    match Async::<TcpListener>::bind(SocketAddr::new(IpAddr::V4(Ipv4Addr::from_str("0.0.0.0").unwrap()), p.local_port)) {
        Ok(listener) => { Ok(listener) }
        Err(e) => {
            Err(ForwardingError::BindError(format!("[{:7}] [{:25}] [0.0.0.0:{:<5}] [{e}]", "绑定失败", p.label, p.local_port)))
        }
    }
}

async fn check_ssh_connection(f: &Forwarding) -> bool {
    if let Err(e) = create_session(f).await {
        eprintln!("{e}");
        return false;
    }
    true
}

async fn create_session(f: &Forwarding) -> Result<AsyncSession<TcpStream>, ForwardingError> {
    if let Ok(stream) = Async::<TcpStream>::connect(SocketAddr::new(IpAddr::V4(Ipv4Addr::from_str(f
        .ssh_host.as_str()).unwrap()), f.ssh_port)).or(
        async {
            Timer::after(Duration::from_secs(5)).await;
            Err(Error::new(std::io::ErrorKind::NotFound, format!("[{:7}] [{:>15}:{:<5}] by [{} / {}]",
                                                                 "连接服务器失败", f.ssh_host, f.ssh_port, f.ssh_username, f.ssh_password)))
        }
    ).await {
        match AsyncSession::new(stream, None) {
            Ok(mut session) => {
                if let Err(e) = session.handshake().await {
                    Err(ForwardingError::SshError(format!("[{:7}] [{:>15}:{:<5}] [{e}]", "握手异常", f.ssh_host, f.ssh_port)))
                } else {
                    if let Err(e) = session.userauth_password(f.ssh_username.as_str(), f.ssh_password
                        .as_str()).await {
                        Err(ForwardingError::SshError(format!("[{:7}] [{:>15}:{:<5}] by [{} / {}] [{e}]", "验证失败", f.ssh_host, f.ssh_port, f.ssh_username, f.ssh_password)))
                    } else {
                        Ok(session)
                    }
                }
            }
            Err(e) => {
                Err(ForwardingError::SshError(format!("[{:7}] [{:>15}:{:<5}] [{e}]", "新建会话失败", f.ssh_host, f.ssh_port)))
            }
        }
    } else {
        Err(ForwardingError::SshError(format!("[{:7}] [{:>15}:{:<5}] by [{}:{}]", "无法连接服务器", f.ssh_host, f.ssh_port, f.ssh_username, f.ssh_password
        )))
    }
}

async fn create_channel(session: &AsyncSession<TcpStream>, p: &Port) -> Result<AsyncChannel<TcpStream>, ForwardingError> {
    match session.channel_direct_tcpip(p.remote_host.as_str(), p.remote_port, None).await {
        Ok(channel) => {
            Ok(channel)
        }
        Err(e) => {
            Err(ForwardingError::RemoteChannelError(format!("[{:7}] [{:25}] [{:>15}:{:<5}] [{e}]", "创建隧道异常", p.label, p.remote_host, p.remote_port)))
        }
    }
}

async fn handle_local_connect(mut local_stream: Async<TcpStream>, mut channel: AsyncChannel<TcpStream>) {
    tokio::spawn(async move {
        let mut local_buf = [0; 1024];
        let mut channel_buf = [0; 1024];
        loop {
            select! {
                l = local_stream.read(&mut local_buf) => match l {
                    Ok(n) if n > 0 => {
                       if let Err(e) =  channel.write(&local_buf[..n]).await{
                            eprintln!("写入远程通道异常 {e}");
                            break;
                        }
                    },
                    _ =>{break;}
                },
                c = channel.read(&mut channel_buf) => match c {
                    Ok(n) if n > 0 => {
                       if let Err(e) =  local_stream.write(&channel_buf[..n]).await{
                            eprintln!("写入本地通道异常 {e}");
                            break;
                        }
                    },
                    _ =>{break;}
                }
            }
        }
    });
}

