use {
    std::{net, io},
    super::*,
    super::super::{http, Handle},
    crate::logger::help::*
};

pub struct LinePool {
    lines: Vec<Line>,
    max_line: usize,
    pub http_muxer: http::Muxer,
}

impl LinePool {
    pub fn new(max_line: usize) -> Self {
        LinePool {
            lines: vec![],
            max_line,
            http_muxer: http::Muxer::default(),
        }
    }

    pub fn handle(&mut self, s: net::TcpStream) {
        if self.lines.len() == 0 {
            self.add_new_line();
            self.handle(s);
        } else if self.lines.len() == self.max_line {
            warn!("out of capacity to handle incoming TCP stream");
            match s.shutdown(net::Shutdown::Both) {
                Ok(_) => {},
                Err(e) => error!("failed to shut down over capacity TCP stream: {}", e)
            };
        } else {
            self.send_to_line(s, 0);
        }
    }

    fn get_muxer(&mut self) -> impl FnMut(net::TcpStream) -> io::Result<()> + Send + Sync + 'static {
        let http_muxer = self.http_muxer.clone();
        move |s: net::TcpStream| {
            let mut buf_read = io::BufReader::new(&s);
            let mut req = http::Req::new(&mut buf_read)?;
            info!("{} {}", req.method(), req.path());
            let mut buf_write = io::BufWriter::new(&s);
            let mut res = http::Res::new(&mut buf_write);
            if let Some(mut handler) = http_muxer.get_handler(&mut req) {
                handler.handle(&mut req, &mut res);
                if !res.responded() {
                    res.set_status(500, "Empty Response");
                    res.respond(b"Empty Response")?;
                }
            } else {
                res.set_status(404, "Not Found");
                res.respond(b"Not Found")?;
            }
            s.shutdown(net::Shutdown::Both)?;

            Ok(())
        }
    }

    fn add_new_line(&mut self) {
        let m = self.get_muxer();
        self.lines.push(Line::new(m));
        debug!("new line added. line count:{}", self.lines.len());
    }

    fn send_to_line(&mut self, s: net::TcpStream, idx: usize) -> Option<net::TcpStream> {
        match self.lines[idx].send(s) {
            Ok(_) => None,
            Err((s_back, e)) => {
                debug!("e: {}", e);
                match e {
                    SendError::LineBusy => {
                        if idx + 1 == self.lines.len() {
                            self.add_new_line();
                        }
                        self.send_to_line(s_back, idx + 1)
                    },
                    SendError::Disconnected => {
                        self.lines.remove(idx);
                        debug!("line#{} removed due to disconnection", idx);
                        self.send_to_line(s_back, idx)
                    }
                }
            }
        }
    }
}