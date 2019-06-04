extern crate num_cpus;

use {
    crate::logger::micro::*,
    std::{
        io, net,
        sync::{Arc, atomic::{AtomicBool, Ordering}},        
    },
    super::{line, http},
};

pub struct Server {
    listener: net::TcpListener,
    stop: Arc<AtomicBool>,
    muxer: line::LineMuxer,
}

impl Server {
    pub fn new(port: i16, max_line: usize) -> io::Result<Self> {
        let addr = format!("127.0.0.1:{}", port);
        info!("server created @ {}", addr);
        let listener = match net::TcpListener::bind(addr) {
            Ok(tl) => tl,
            Err(e) => return Err(e),
        };
        Ok(Server{
            listener: listener,
            stop: Arc::new(AtomicBool::new(false)),
            muxer: line::LineMuxer::new(max_line),
        })
    }

    pub fn start(&mut self) -> io::Result<()> {
        info!("server start listening");
        while !self.stop.load(Ordering::SeqCst) {
            let (stream, addr) = match self.listener.accept() {
                Ok(res) => res,
                Err(e) => return Err(e),
            };
            trace!("incoming connection from {}", addr);
            self.muxer.handle(stream);
        }
        
        Ok(())
    }

    pub fn add(&mut self, m: http::Method, p: &'static str, h: impl FnMut(&mut http::Req, &mut http::Res) + Send + Sync + 'static) {
        self.muxer.http_muxer.add_handler(m, p, h)
    }

    pub fn filter(&mut self, m: impl Fn(&http::Req) -> bool + Send + Sync + 'static) -> http::MatchChain {
        self.muxer.http_muxer.filter(m)
    }
}

