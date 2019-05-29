use std::{
    collections,
    sync,
};

use super::super::http;

pub type Handler = FnMut(&mut http::req::Req, &mut http::res::Res) + Send + Sync + 'static;
pub type HandlerRef = sync::Arc<sync::Mutex<Handler>>;

#[derive(Default, Clone)]
pub struct RouteMap {
    methods: collections::HashMap<http::method::Method, Vec<HandlerRef>>,
    paths: collections::HashMap<&'static str, Vec<HandlerRef>>,
}


impl RouteMap {
    pub fn register(&mut self, m: http::method::Method, p: &'static str, h: impl FnMut(&mut http::req::Req, &mut http::res::Res) + Send + Sync + 'static) {
        let r: HandlerRef = sync::Arc::new(sync::Mutex::new(h));
        self.register_handler_ref_by_method(m, &r);
        self.register_handler_ref_by_path(p, &r);
    }

    fn register_handler_ref_by_method(&mut self, m: http::method::Method, handler_ref: &sync::Arc<sync::Mutex<Handler>>) {
        match self.methods.get_mut(&m) {
            Some(v) => v.push(handler_ref.clone()),
            None => {
                self.methods.insert(m, vec![handler_ref.clone()]);
                if m != http::method::Method::UNKNOWN {
                    self.register_handler_ref_by_method(http::method::Method::UNKNOWN, handler_ref);
                }
            },
        };
    }

    fn register_handler_ref_by_path(&mut self, p: &'static str , handler_ref: &HandlerRef) {
        match self.paths.get_mut(p) {
            Some(v) => v.push(handler_ref.clone()),
            None => {
                self.paths.insert(p, vec![handler_ref.clone()]);
                if p != "" {
                    self.register_handler_ref_by_path("", handler_ref);
                }
            },
        }
    }

    pub fn get_handlers(&mut self, m: &http::method::Method, p: &String) -> Vec<sync::Arc<sync::Mutex<Handler>>> {
        if let Some(matched_by_path) = self.paths.get(p.as_str()) {
            if let Some(matched_by_method) = self.methods.get(&m) {
                return Self::find_matches(&matched_by_path, &matched_by_method);
            }
        }
        vec![]
    }

    fn find_matches(l1: &Vec<HandlerRef>, l2: &Vec<HandlerRef>) -> Vec<HandlerRef> {
        let mut matched = vec![];
        for h1 in l1.iter() {
            for h2 in l2.iter() {
                if std::ptr::eq(h1.as_ref(), h2.as_ref()) {
                    matched.push(h1.clone());
                }
            }
        }
        matched
    }
}

#[cfg(test)]
mod tests {
    use super::{http, RouteMap};

    #[test]
    fn test_route_map() {
        let mut res = http::res::Res::default();
        let mut req = http::req::Req::default();

        let mut rm = RouteMap::default();

        rm.register(http::method::Method::UNKNOWN, "/", |_, resp|{
            resp.set_status(100, "wo");
        });
        rm.register(http::method::Method::GET, "/hello", |_, resp|{
            resp.set_status(300, "hello");
        });

        let mut test_trigger = |rm: &mut RouteMap, m: http::method::Method, p: &'static str, expected_size: usize, expected_status: &'static str| {
            let hs = rm.get_handlers(&m, &p.to_string());
            if expected_size != 0 {
                assert_eq!(hs.len(), expected_size);
            }
            for h in hs {
                let hrm = &mut *h.lock().unwrap();
                hrm(&mut req, &mut res);
            }

            assert_eq!(res.status(), expected_status);
        };

        test_trigger(&mut rm, http::method::Method::UNKNOWN, "", 2, "hello");

        test_trigger(&mut rm, http::method::Method::UNKNOWN, "/", 1, "wo");

        test_trigger(&mut rm, http::method::Method::GET, "", 1, "hello");
    }
}