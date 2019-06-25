pub mod req;
pub mod res;
pub mod method;
pub mod matcher;
pub mod serve_static;
mod headers;
mod trie;

pub use {
    req::Req,
    res::Res,
    method::Method,
    matcher::{Muxer, MatchChain},
    handler::Handle,
};

use {
    std::{sync},
};

pub mod handler {
    use super::*;
    pub type Handler = FnMut(&mut Req, &mut Res) + Send + Sync + 'static;
    pub type HandlerRef = sync::Arc<sync::Mutex<Handler>>;

    pub trait Handle {
        fn handle(&mut self, request: &mut Req, response: &mut Res);
    }
    impl Handle for HandlerRef {
        fn handle(&mut self, request: &mut Req, response: &mut Res) {
            (&mut *self.lock().unwrap())(request, response);
        }
    }
}