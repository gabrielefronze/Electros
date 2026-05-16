use parking_lot::Mutex;
use std::collections::HashMap;
use std::process::Child;
use std::sync::Arc;

pub struct AppState {
    pub daemon_logs: Mutex<Vec<String>>,
    pub daemon_child: Mutex<Option<Child>>,
    pub used_ports: Mutex<Vec<u16>>,
    pub rdp_processes: Mutex<HashMap<String, Child>>,
    pub rdp_ports: Mutex<HashMap<String, u16>>,
    pub resource_dir: Mutex<Option<std::path::PathBuf>>,
    pub last_rdp_window: Mutex<Option<String>>,
    pub no_daemons: bool,
}

impl AppState {
    pub fn new(no_daemons: bool) -> Arc<Self> {
        Arc::new(Self {
            daemon_logs: Mutex::new(Vec::new()),
            daemon_child: Mutex::new(None),
            used_ports: Mutex::new(Vec::new()),
            rdp_processes: Mutex::new(HashMap::new()),
            rdp_ports: Mutex::new(HashMap::new()),
            resource_dir: Mutex::new(None),
            last_rdp_window: Mutex::new(None),
            no_daemons,
        })
    }

    pub fn push_log(&self, line: impl Into<String>) {
        let mut buf = self.daemon_logs.lock();
        buf.push(line.into());
        const MAX: usize = 2000;
        if buf.len() > MAX {
            let drain = buf.len() - MAX;
            buf.drain(0..drain);
        }
    }
}

pub fn pick_port(state: &AppState) -> u16 {
    use std::net::TcpListener;
    let mut used = state.used_ports.lock();
    for _ in 0..100 {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind ephemeral");
        let port = listener.local_addr().unwrap().port();
        drop(listener);
        if !used.contains(&port) {
            used.push(port);
            return port;
        }
    }
    49152
}

pub fn release_port(state: &AppState, port: u16) {
    state.used_ports.lock().retain(|p| *p != port);
}
