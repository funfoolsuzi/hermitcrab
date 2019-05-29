extern crate num_cpus;

use std::{
    io,
    net,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};
use super::{
    line,
    super::logger::micro::*,
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
}

