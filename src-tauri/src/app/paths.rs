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
