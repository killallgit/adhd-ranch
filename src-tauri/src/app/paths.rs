use std::io;
use std::path::PathBuf;

pub fn data_root() -> io::Result<PathBuf> {
    let home = std::env::var_os("HOME")
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "HOME not set"))?;
    Ok(PathBuf::from(home).join(".adhd-ranch"))
}

pub fn focuses_root() -> io::Result<PathBuf> {
    Ok(data_root()?.join("focuses"))
}

pub fn port_file() -> io::Result<PathBuf> {
    Ok(data_root()?.join("run/port"))
}

pub fn proposals_file() -> io::Result<PathBuf> {
    Ok(data_root()?.join("proposals.jsonl"))
}

pub fn decisions_file() -> io::Result<PathBuf> {
    Ok(data_root()?.join("decisions.jsonl"))
}

pub fn settings_file() -> io::Result<PathBuf> {
    Ok(data_root()?.join("settings.yaml"))
}
