mod server;
mod logger;

pub use {
    server::*,
    logger::*,
};

#[cfg(test)]
mod main_test {
    use super::*;
    use std::env;
    use crate::logger::help::*;

    const DEFAULT_HTTP_PORT:i16 = 80;

    #[test]
    #[ignore]
    fn main_test() {
        logger::init_stdout_logger(10, logger::Level::Trace).unwrap();

        // let port = get_http_port();
        let port = 9999;
        let ncpu = num_cpus::get();

        info!("# of CPU: {}", ncpu);
        let mut s = server::Server::new(port, ncpu*2).unwrap();
        set_up_server_handlers(&mut s);
        
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

    fn set_up_server_handlers(server: &mut server::Server) {
        server.add(server::Method::GET, "/hello", |_, res: &mut server::Res| {
            res.respond(b"Hello").unwrap();
        });
        server.filter(|req: &mut server::Req| {
            req.path() == "/sample"
        }).handle(|_, res: &mut server::Res| {
            res.respond(b"Lorem ipsum").unwrap();
        });
        server.serve_static("/", "test_data").unwrap();
    }
}
