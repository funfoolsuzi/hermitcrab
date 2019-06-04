use std::{
    net,
    io,
};
use super::*;
use super::super::{
    http,
    super::logger::micro::*,
};


pub struct LineMuxer {
    lines: Vec<Line>,
    max_line: usize,
    pub http_muxer: http::Muxer,
}

impl LineMuxer {
    pub fn new(max_line: usize) -> Self {
        Self {
            lines: vec![],
            max_line,
            http_muxer: http::Muxer::default(), // TODO: move to server
        }
    }

    pub fn handle(&mut self, s: net::TcpStream) {
        if self.lines.len() == 0 {
            self.add_new_line();
            self.handle(s);
        } else if self.lines.len() == self.max_line {
            // TODO: 
            //   Full
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
            if let Some(handler_ref) = http_muxer.get_handler(&req) {
                let handler = &mut *handler_ref.lock().unwrap();
                handler(&mut req, &mut res);
                // TODO: write back
            } else {
                res.set_status(404, "Not Found.");
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