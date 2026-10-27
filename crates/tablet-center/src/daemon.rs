//! Persistent line-based daemon transport, isolated from the UI thread.
use std::{
    io::{self, BufRead, BufReader, Write},
    os::unix::net::UnixStream,
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc,
    },
    time::Duration,
};
use tablet_config::{Config, Request, Response};
#[derive(Default)]
pub struct Client {
    connection: Option<BufReader<UnixStream>>,
}
impl Client {
    pub fn ask(&mut self, request: &Request) -> io::Result<Response> {
        for _ in 0..2 {
            if self.connection.is_none() {
                let stream = UnixStream::connect(tablet_config::socket_path())?;
                stream.set_read_timeout(Some(Duration::from_millis(500)))?;
                stream.set_write_timeout(Some(Duration::from_millis(500)))?;
                self.connection = Some(BufReader::new(stream));
            }
            if let Some(conn) = self.connection.as_mut() {
                match exchange(conn, request) {
                    Ok(response) => return Ok(response),
                    Err(e) => {
                        self.connection = None;
                        if matches!(
                            e.kind(),
                            io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
                        ) {
                            return Err(e);
                        }
                    }
                }
            }
        }
        Err(io::ErrorKind::NotConnected.into())
    }
}
fn exchange(conn: &mut BufReader<UnixStream>, request: &Request) -> io::Result<Response> {
    let mut bytes = serde_json::to_vec(request)?;
    bytes.push(b'\n');
    conn.get_mut().write_all(&bytes)?;
    let mut line = String::new();
    if conn.read_line(&mut line)? == 0 {
        return Err(io::ErrorKind::UnexpectedEof.into());
    }
    Ok(serde_json::from_str(&line)?)
}
pub struct Worker {
    pub visible: Arc<AtomicBool>,
    pub save: mpsc::Sender<(Config, serde_json::Value, u64)>,
    pub updates: mpsc::Receiver<Result<Response, String>>,
    pub errors: mpsc::Receiver<String>,
}
impl Worker {
    pub fn start(ctx: egui::Context, path: PathBuf) -> Self {
        let (save, commands) = mpsc::channel::<(Config, serde_json::Value, u64)>();
        let (tx, updates) = mpsc::channel();
        let (error_tx, errors) = mpsc::channel();
        let visible = Arc::new(AtomicBool::new(true));
        let active = visible.clone();
        std::thread::spawn(move || {
            let mut client = Client::default();
            loop {
                match commands.recv_timeout(Duration::from_millis(40)) {
                    Ok(mut command) => {
                        while let Ok(new) = commands.try_recv() {
                            command = new;
                        }
                        let (config, menus, epoch) = command;
                        let result = (|| -> io::Result<()> {
                            let _guard = crate::system::CONFIG_IO
                                .lock()
                                .map_err(|e| io::Error::other(e.to_string()))?;
                            if epoch != crate::system::config_epoch() {
                                return Ok(());
                            }
                            if let Some(parent) = path.parent() {
                                std::fs::create_dir_all(parent)?;
                            }
                            let tmp = path.with_extension("json.tmp");
                            std::fs::write(&tmp, serde_json::to_vec_pretty(&config)?)?;
                            std::fs::rename(tmp, &path)?;
                            crate::system::save_shell_menus(menus).map_err(io::Error::other)?;
                            client.ask(&Request::Reload).map_err(|_| {
                                io::Error::other("Settings saved. Start tabletd to apply them.")
                            })?;
                            Ok(())
                        })();
                        if let Err(e) = result {
                            let _ = error_tx.send(e.to_string());
                        }
                    }
                    Err(mpsc::RecvTimeoutError::Disconnected) => break,
                    Err(mpsc::RecvTimeoutError::Timeout) => {}
                }
                if !active.load(Ordering::Relaxed) {
                    continue;
                }
                let result = client.ask(&Request::Status).map_err(|e| e.to_string());
                if tx.send(result).is_err() {
                    break;
                }
                ctx.request_repaint();
            }
        });
        Self {
            visible,
            save,
            updates,
            errors,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn persistent_exchange_accepts_multiple_replies_on_one_connection() {
        let (ui, daemon) = UnixStream::pair().unwrap();
        let thread = std::thread::spawn(move || {
            let mut conn = BufReader::new(daemon);
            for expected in [Request::Status, Request::Reload] {
                let mut line = String::new();
                conn.read_line(&mut line).unwrap();
                assert_eq!(
                    serde_json::from_str::<serde_json::Value>(&line).unwrap(),
                    serde_json::to_value(expected).unwrap()
                );
                conn.get_mut()
                    .write_all(b"{\"ok\":true,\"error\":null,\"device\":null,\"pen\":null}\n")
                    .unwrap();
            }
        });
        let mut conn = BufReader::new(ui);
        assert!(exchange(&mut conn, &Request::Status).unwrap().ok);
        assert!(exchange(&mut conn, &Request::Reload).unwrap().ok);
        thread.join().unwrap();
    }
    #[test]
    fn timed_out_requests_are_not_replayed() {
        let (ui, daemon) = UnixStream::pair().unwrap();
        ui.set_read_timeout(Some(Duration::from_millis(20)))
            .unwrap();
        daemon
            .set_read_timeout(Some(Duration::from_millis(100)))
            .unwrap();
        let thread = std::thread::spawn(move || {
            let mut peer = BufReader::new(daemon);
            let mut line = String::new();
            peer.read_line(&mut line).unwrap();
            assert!(line.contains("status"));
            line.clear();
            assert!(!matches!(peer.read_line(&mut line),Ok(n) if n>0));
        });
        let mut client = Client {
            connection: Some(BufReader::new(ui)),
        };
        let error = client.ask(&Request::Status).unwrap_err();
        assert!(matches!(
            error.kind(),
            io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
        ));
        assert!(client.connection.is_none());
        thread.join().unwrap();
    }
    #[test]
    fn disconnected_socket_is_an_error() {
        let (ui, daemon) = UnixStream::pair().unwrap();
        drop(daemon);
        assert!(exchange(&mut BufReader::new(ui), &Request::Status).is_err());
    }
}
