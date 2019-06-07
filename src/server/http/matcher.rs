use {
    std::{
        sync,
        collections,
    },
    super::{
        req::Req,
        res::Res,
        method::Method,
    }
};



pub type Handler = FnMut(&mut Req, &mut Res) + Send + Sync + 'static;
pub type HandlerRef = sync::Arc<sync::Mutex<Handler>>;
pub type Matcher = Fn(&mut Req) -> bool + Send + Sync + 'static;
pub type MatcherRef = sync::Arc<Matcher>;

type MapEntry = (Method, &'static str);

#[derive(Clone)]
pub struct MatchEntry {
    matcher: MatcherRef,
    handler: HandlerRef,
}


#[derive(Default, Clone)]
pub struct Muxer {
    filters: Vec<MatchEntry>,
    map: collections::HashMap<MapEntry, HandlerRef>,
}

impl Muxer {
    pub fn filter(&mut self, m: impl Fn(&mut Req) -> bool + Send + Sync + 'static) -> MatchChain {
        let matcher: MatcherRef = sync::Arc::new(m);
        MatchChain {
            matchers: vec![matcher],
            muxer: self,
        }
    }

    fn add_handler_from_matchers(&mut self, matchers: Vec<MatcherRef>, handler: HandlerRef) {
        let m: MatcherRef;
        if matchers.len() == 1 {
            m = matchers[0].clone();
        } else {
            m = Self::combine(matchers);
        }
        self.filters.push(MatchEntry{
            matcher: m,
            handler: handler,
        })
    }

    pub fn add_handler(&mut self, m: Method, p: &'static str, h: impl FnMut(&mut Req, &mut Res) + Send + Sync + 'static) {
        let hf: HandlerRef = sync::Arc::new(sync::Mutex::new(h));
        self.map.insert((m, p), hf);
    }

    pub fn get_handler(&self, req: &mut Req) -> Option<HandlerRef> {
        if let Some(handler) = self.map.get(&(*req.method(), req.path())) {
            Some(handler.clone())
        } else {
            for m in self.filters.iter() {
                if (m.matcher)(req) {
                    return Some(m.handler.clone());
                }
            }
            None
        }
    }

    fn combine(matchers: Vec<MatcherRef>) -> MatcherRef{
        sync::Arc::new(move |req| -> bool {
            for m in matchers.iter(){
                if !m(req) {
                    return false;
                }
            }
            true
        })
    }
}

pub struct MatchChain<'a> {
    matchers: Vec<MatcherRef>,
    muxer: &'a mut Muxer,
}

impl<'a> MatchChain<'a> {
    #[allow(dead_code)]
    pub fn filter(mut self, m: impl Fn(&mut Req) -> bool + Send + Sync + 'static) -> Self {
        let matcher: MatcherRef = sync::Arc::new(m);
        self.matchers.push(matcher);
        Self {
            matchers: self.matchers,
            muxer: self.muxer,
        }
    }
    pub fn handle(self, h: impl FnMut(&mut Req, &mut Res) + Send + Sync + 'static) {
        let handler: HandlerRef = sync::Arc::new(sync::Mutex::new(h));
        self.muxer.add_handler_from_matchers(self.matchers, handler);
    }
}



#[cfg(test)]
mod matcher_test {
    use {
        std::{
            io,
        },
        super::*,
        super::super::method::Method,
    };

    const TEST_REQ_MSG_STR_1: &str = "GET /hi HTTP/1.1\r\nHost: www.xiwen.com\r\nContent-Type: text\r\n\r\nWassup!";
    const TEST_REQ_MSG_STR_2: &str = "GET /haha HTTP/1.1\r\nHost: www.xiwen.com\r\nContent-Type: text\r\n\r\nWassup!";
    const TEST_REQ_MSG_STR_3: &str = "POST /login HTTP/1.1\r\nHost: www.xiwen.com\r\nContent-Type: text\r\n\r\nWassup!";
 
    #[test]
    fn test_matched_handler() {
        let mux = create_test_muxer();

        let mut buf = io::BufReader::new(TEST_REQ_MSG_STR_1.as_bytes());
        let mut incoming_req = Req::new(&mut buf).unwrap();

        let match_res = mux.get_handler(&mut incoming_req);
        assert!(match_res.is_some());
        let matched = match_res.unwrap();
        
        let mut read_buf: Vec<u8> = vec![];
        let mut res = Res::new(&mut read_buf);
        (&mut *matched.lock().unwrap())(&mut incoming_req, &mut res);

        assert_eq!(res.status(), "Hello");
    }

    #[test]
    fn test_not_matched_handler() {
        let mux = create_test_muxer();

        let mut buf = io::BufReader::new(TEST_REQ_MSG_STR_2.as_bytes());
        let mut incoming_req = Req::new(&mut buf).unwrap();

        let match_res = mux.get_handler(&mut incoming_req);
        assert!(match_res.is_none());
    }

    fn create_test_muxer() -> Muxer {
        let mut mux = Muxer::default();

        mux.add_handler(Method::POST, "/login", |_, res: &mut Res| {
            res.set_status(403, "bad login");
        });

        mux.filter(|r: &mut Req| {
            r.method() == &Method::GET
        }).filter(|r: &mut Req| {
            r.path() == "/hi"
        }).handle(|_, res: &mut Res| {
            res.set_status(210, "Hello");
        });
        mux
    }

    #[test]
    fn test_match_mapped_handler() {
        let mux = create_test_muxer();

        let mut buf = io::BufReader::new(TEST_REQ_MSG_STR_3.as_bytes());
        let mut incoming_req = Req::new(&mut buf).unwrap();

        let match_res = mux.get_handler(&mut incoming_req);
        assert!(match_res.is_some());

        let mut read_buf: Vec<u8> = vec![];
        let mut res = Res::new(&mut read_buf);
        (&mut *match_res.unwrap().lock().unwrap())(&mut incoming_req, &mut res);

        assert_eq!(res.status(), "bad login"); 
    }
}