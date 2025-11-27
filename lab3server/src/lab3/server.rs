


static CANCEL_FLAG: AtomicBool = AtomicBool::new(false);


struct Server {
    listener: Option<TcpListener>,
    listening_addr: String,
}


impl Server {
    pub fn new() -> Self {
        Self {
            listener: None,
            listening_addr: "".to_string(),
        }
    }

    fn is_open(&self) -> bool {
        if let None = self.listener {
            false
        } else {
            true
        }
    }

    fn open(&mut self, addr: &str) {
        if let Ok(lstnr) = TcpListener::bind(addr) {
            self.listener = Some(lstnr);
            self.listening_addr = addr.to_string();
        }
    }
}
