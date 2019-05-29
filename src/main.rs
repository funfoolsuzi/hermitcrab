extern crate num_cpus;

mod server;
mod logger;

use std::env;
use logger::micro::*;

const DEFAULT_HTTP_PORT:i16 = 80;

fn main() {
    logger::init_stdout_logger(10, logger::Level::Trace).unwrap();

    // let port = get_http_port();
    let port = 9999;
    let ncpu = num_cpus::get();

    info!("# of CPU: {}", ncpu);
    let mut s = server::Server::new(port, ncpu*2).unwrap();
    s.start().unwrap();
}

#[allow(unused)]
fn get_http_port() -> i16 {
    match env::var("HERMIT_CRAB_ALTERNATE_HTTP_PORT") {
        Ok(port_str) => {
            match port_str.parse::<i16>() {
                Ok(port) => port,
                Err(e) => {
                    warn!("{}. using default http port", e);
                    DEFAULT_HTTP_PORT
                },
            }
        },
        Err(_) => DEFAULT_HTTP_PORT,
    } 
}