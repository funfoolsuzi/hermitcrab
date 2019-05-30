use std::{
    net,
    io,
};
use super::*;
use super::super::{http, super::logger::micro::*};

pub struct LineMuxer {
    lines: Vec<Line>,
    max_line: usize,
    pub routes: RouteMap,
}

impl LineMuxer {
    pub fn new(max_line: usize) -> Self {
        Self {
            lines: vec![],
            max_line,
            routes: RouteMap::default(),
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
        let mut routes = self.routes.clone();
        move |mut s: net::TcpStream| {
            let mut buf_read = io::BufReader::new(&s);
            let mut req = http::req::Req::new(&mut buf_read)?;
            info!("{} {}", req.method(), req.path());
            let mut res = http::res::Res::default();
            let hs = routes.get_handlers(req.method(), req.path());
            for h in hs {
                let hrm = &mut *h.lock().unwrap();
                hrm(&mut req, &mut res);
            }
            // TODO: write back valid body
            res.respond(&mut s, b"hello")?;
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