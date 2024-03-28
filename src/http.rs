fn get_req(path: &str) -> String {
    format!(
        "
    GET {path} HTTP/1.1 \r\n
        Host: localhost \r\n
        Connection: close \r\n
        \r\n
    "
    )
}

pub struct Http;
impl Http {
    pub fn get(path: &str) -> impl Future<Output = String> {
        HttpGetFuture::new(path)
    }
}

struct HttpGetFuture {
    stream: Option<mio::net::TcpStream>,
    buffer: Vec<u8>,
    path: String,
}

impl HttpGetFuture {
    fn new(path: &'static str) -> Self {
        Self {
            stream: None,
            buffer: vec![],
            path: path.to_string(),
        }
    }
    fn write_request(&mut self) {
        let stream = std::net::TcpStream::connect("localhost:8000").unwrap();
        stream.set_nonblocking(true).unwrap();
        let mut stream = mio::net::TcpStream::from_std(stream);
        stream.write_all(get_req(&self.path).as_bytes()).unwrap();
        self.stream = Some(stream);
    }
}

// most important section
impl Future for HttpGetFuture {
    type Output = String;
    fn poll(&mut self) -> PollState<Self::Output> {
        // NOT STARTED STATE
        // if the poll is called first time.
        if self.stream.is_none() {
            println!("FIRST POLL - START OPERATION");
            self.write_request();
            return PollState::NotReady;
        }
        let mut buff = vec![0u8; 4096];
        loop {
            match self.stream.as_mut().unwrap().read(&mut buff) {
                // RESOLVED STATE
                // if all response bytes are fully read, n = 0, then clear the buffer into a String.
                Ok(0) => {
                    let s = String::from_utf8_lossy(&self.buffer);
                    break PollState::Ready(s.to_string());
                }
                // if response bytes are nonempty, n > 0, then store it in the buffer
                Ok(n) => {
                    self.buffer.extend(&buff[0..n]);
                    continue;
                }

                // PENDING STATE
                // more calls to the poll are needed to get the data we haven’t received yet
                Err(e) if e.kind() == ErrorKind::WouldBlock => {
                    break PollState::NotReady;
                }
                Err(e) if e.kind() == ErrorKind::Interrupted => {
                    continue;
                }
                Err(e) => panic!("{e:?}"),
            }
        }
    }
}
