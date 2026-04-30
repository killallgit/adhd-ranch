use std::io;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::PathBuf;
use std::sync::Arc;

use adhd_ranch_storage::FocusRepository;
use tokio::net::TcpListener;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;

use crate::router::router;

#[derive(Debug)]
pub enum ServeError {
    Io(io::Error),
}

impl std::fmt::Display for ServeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(f, "serve io error: {error}"),
        }
    }
}

impl std::error::Error for ServeError {}

impl From<io::Error> for ServeError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

pub struct ServerHandle {
    addr: SocketAddr,
    port_file: Option<PathBuf>,
    shutdown: Option<oneshot::Sender<()>>,
    join: Option<JoinHandle<()>>,
}

impl ServerHandle {
    pub fn addr(&self) -> SocketAddr {
        self.addr
    }

    pub fn port(&self) -> u16 {
        self.addr.port()
    }

    pub async fn shutdown(mut self) {
        if let Some(tx) = self.shutdown.take() {
            let _ = tx.send(());
        }
        if let Some(handle) = self.join.take() {
            let _ = handle.await;
        }
        if let Some(path) = self.port_file.take() {
            let _ = std::fs::remove_file(path);
        }
    }
}

impl Drop for ServerHandle {
    fn drop(&mut self) {
        if let Some(tx) = self.shutdown.take() {
            let _ = tx.send(());
        }
        if let Some(path) = self.port_file.take() {
            let _ = std::fs::remove_file(path);
        }
    }
}

pub async fn serve(
    repo: Arc<dyn FocusRepository>,
    port_file: Option<PathBuf>,
) -> Result<ServerHandle, ServeError> {
    let listener = TcpListener::bind(SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0)).await?;
    let addr = listener.local_addr()?;

    if let Some(path) = port_file.as_ref() {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, addr.port().to_string())?;
    }

    let app = router(repo);
    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
    let join = tokio::spawn(async move {
        let _ = axum::serve(listener, app)
            .with_graceful_shutdown(async move {
                let _ = shutdown_rx.await;
            })
            .await;
    });

    Ok(ServerHandle {
        addr,
        port_file,
        shutdown: Some(shutdown_tx),
        join: Some(join),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use adhd_ranch_storage::MarkdownFocusRepository;
    use std::fs;
    use tempfile::TempDir;

    fn write_focus(root: &std::path::Path, slug: &str, body: &str) {
        let dir = root.join(slug);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("focus.md"), body).unwrap();
    }

    fn focus_md(id: &str, title: &str, description: &str, created_at: &str) -> String {
        format!("---\nid: {id}\ntitle: {title}\ndescription: {description}\ncreated_at: {created_at}\n---\n")
    }

    #[tokio::test]
    async fn server_writes_port_file_and_serves_focuses() {
        let dir = TempDir::new().unwrap();
        let focuses_root = dir.path().join("focuses");
        write_focus(
            &focuses_root,
            "a",
            &focus_md("a", "Alpha", "x", "2026-04-30T12:00:00Z"),
        );
        let repo = Arc::new(MarkdownFocusRepository::new(focuses_root));
        let port_file = dir.path().join("run/port");

        let handle = serve(repo, Some(port_file.clone())).await.unwrap();
        let port = std::fs::read_to_string(&port_file)
            .unwrap()
            .trim()
            .parse::<u16>()
            .unwrap();
        assert_eq!(port, handle.port());

        let url = format!("http://127.0.0.1:{port}/focuses");
        let body = reqwest_get_text(&url).await;
        let entries: Vec<serde_json::Value> = serde_json::from_str(&body).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0]["id"], "a");

        handle.shutdown().await;
        assert!(
            !port_file.exists(),
            "port file should be removed on shutdown"
        );
    }

    async fn reqwest_get_text(url: &str) -> String {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        let url = url.strip_prefix("http://").unwrap();
        let (host_port, path) = url.split_once('/').unwrap();
        let mut stream = tokio::net::TcpStream::connect(host_port).await.unwrap();
        let req = format!("GET /{path} HTTP/1.1\r\nHost: {host_port}\r\nConnection: close\r\n\r\n");
        stream.write_all(req.as_bytes()).await.unwrap();
        let mut buf = Vec::new();
        stream.read_to_end(&mut buf).await.unwrap();
        let raw = String::from_utf8(buf).unwrap();
        let body_start = raw.find("\r\n\r\n").unwrap() + 4;
        raw[body_start..].to_string()
    }
}
