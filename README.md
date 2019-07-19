# Hermit Crab

## Purpose

[__NIH__(Not Invented Here)](https://en.wikipedia.org/wiki/Not_invented_here)
Just for fun.

## How to use

Add dependency in `Cargo.toml`:

```toml
// ...
[dependencies]
hermitcrab = { git = "https://github.com/funfoolsuzi/hermitcrab" }
// ...
```

In the app:

```rust
fn main_test() {
    logger::init_stdout_logger(10, logger::Level::Trace).unwrap();

    let port = 9999;

    // let ncpu = num_cpus::get();
    // info!("# of CPU: {}", ncpu);

    // specify how many thread can be generated.
    let mut s = server::Server::new(port, 8).unwrap();
    set_up_server_handlers(&mut s);

    s.start().unwrap();
}

fn set_up_server_handlers(server: &mut server::Server) {
    // example 1: directly bind a http handler to a path.
    server.add(server::Method::GET, "/hello", |_, res: &mut server::Res| { // _ can be replaced with req: &mut server::Req if need to read from request
        res.respond(b"Hello").unwrap();
    });
    // example 2: use chained http filter.
    server.filter(|req: &mut server::Req| {
        req.path() == "/sample"
    }).handle(|_, res: &mut server::Res| {
        res.respond(b"Lorem ipsum").unwrap();
    });
    // example 3: serve static folder. The directory path can be relative to the binary or rust main.
    server.serve_static("/", "test_data").unwrap();
}
```

## Features & Potential Improvements

- Multi-thread-pooled TCP stream handler. Fixed number cap on threads. Each TCP session only handle a single HTTP request(No handling of "Keep-Alive" header). No mechanism to kill idle threads(No scheduler).
- TCP stream is handled as Read and Write traits. Future TLS incorporation would be easy.
- HTTP handler is stored as Arc&Mutex, which means later calls to the same endpoint need to wait for ealier call to complete. Arc&Mutex is a catch-all for HTTP handlers so that handlers(closures) can always mutate their environments. But it is not the case when handler doesn't need to mutate its environment and cause unnecessary delay.
- It can serve static directory. It currently load the entire directory into memory.(This need to be flexible) However, just like what's mentioned above, each call will lock the mutex on the handler until the call is fully reponded.
- Besides, when handling requests to static content, it should respond with "Last-Modified" or "E-Tag" for HTTP validation. It should also include "Expire" for caching. These features don't exist currently.

## Future goal

routing with macro
