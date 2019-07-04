use {
    std::{path},
    super::{
        handler::HandlerRef,
        method::Method,
    },
    crate::logger::help::{warn, trace},
};

#[derive(Clone)]
enum Node {
    Passby(String, Vec<Box<Node>>),
    Terminal(Method, HandlerRef),
}

impl Node {
    fn attach(&mut self, p: &str, p_begin: usize, m: &Method, handler: &HandlerRef) {
        if let Node::Passby(_, children) = self {
            if p_begin == p.len() { // found a path match, going thru terminals
                Self::insert_or_replace_existing(children, p, m, Node::Terminal(*m, handler.clone()));
                return;
            }
            // passby node operation
            if children.len() == 0 { // empty passby
                children.push(Box::new(Self::new_passby_and_attach(p, p_begin, m, handler)));
                return;
            }
            Self::attach_passby(children, p, p_begin, m, handler);
        }
    }

    fn get(&self, path: &str, method: &Method) -> Option<HandlerRef> {
        match self {
            Node::Passby(word, children) => {
                if !path.starts_with(word) {
                    return None;
                }
                let mut word_chars = word.chars().peekable();
                let mut remain_chars = path.chars().peekable();
                Self::iter_over_shared_chars(&mut word_chars, &mut remain_chars);
                let new_remain = remain_chars.clone().collect::<String>();
                for child in children {
                    if child.is_ahead_of(new_remain.as_str()) {
                        break;
                    }

                    let ref_res = child.get(&new_remain, method);
                    if ref_res.is_some() {
                        return ref_res;
                    }
                }
                return None
            },
            Node::Terminal(m, handler) => {
                if m == method {
                    return Some(handler.clone());
                } else {
                    return None;
                }
            }
        };
    }

    fn is_ahead_of(&self, target: &str) -> bool {
        if let Node::Passby(word, _) = self {
            return word.as_str() > target;
        }
        false
    }

    fn iter_over_shared_chars(
        p1: &mut std::iter::Peekable<std::str::Chars>,
        p2: &mut std::iter::Peekable<std::str::Chars>,
    ) -> usize {
        let mut shared_length = 0usize;
        loop {
            if let (Some(c1), Some(c2)) = (p1.peek(), p2.peek()) {
                if c1 != c2 {
                    break;
                }
                p1.next();
                p2.next();
                shared_length += 1;
            } else {
                break;
            }
        };
        shared_length
    }

    fn attach_passby(v: &mut Vec<Box<Self>>, p: &str, p_begin: usize, m: &Method, handler: &HandlerRef) {
        let num_nodes = v.len();
        for (idx, child) in v.iter_mut().enumerate() {
            if let Node::Passby(word, grandchildren) = &mut **child {
                let mut remain_chars = p[p_begin..].chars();
                let shared_length = Self::iter_over_shared_chars2(&mut word.chars(), &mut remain_chars);
                let new_p_begin = p_begin + shared_length;
                if shared_length != 0 {
                    if new_p_begin == p.len() { // this is weird, cuz checked in the parent function
                        child.attach(p, new_p_begin, m, handler);
                        return;
                    }
                    // split path; move grandchildren to new passby; attach passby to current child
                    let new_passby = Self::new_passby_and_attach(p, new_p_begin, m, handler);
                    let split_passby = Node::Passby(word[shared_length..].to_string(), grandchildren.clone());
                    let mut new_children = vec![Box::new(new_passby)];
                    if new_children[0].is_ahead_of(&word[shared_length..]) {
                        new_children.insert(0, Box::new(split_passby));
                    } else {
                        new_children.push(Box::new(split_passby));
                    }
                    let mut word_latter_half = word[..shared_length].to_string();
                    std::mem::swap(word, &mut word_latter_half);
                    std::mem::swap(grandchildren, &mut new_children);
                    return;
                }
                if child.is_ahead_of(&p[new_p_begin..]) {
                    v.insert(idx, Box::new(Self::new_passby_and_attach(p, new_p_begin, m, handler)));
                    return;
                }
                if idx == num_nodes-1 {
                    v.push(Box::new(Self::new_passby_and_attach(p, new_p_begin, m, handler)));
                    return
                }
            }
        }
    }

    fn new_passby_and_attach(p: &str, p_begin: usize, m: &Method, handler: &HandlerRef) -> Self {
        let mut new_passby = Node::Passby(p[p_begin..].to_string(), vec![]);
        new_passby.attach(p, p.len(), m, handler);
        new_passby
    }

    fn iter_over_shared_chars2(p1: &mut std::str::Chars, p2: &mut std::str::Chars) -> usize {
        let mut shared_length = 0usize;
        loop {
            if let (Some(c1), Some(c2)) = (p1.next(), p2.next()) {
                if c1 != c2 {
                    break;
                }
                shared_length += 1;
            } else {
                break;
            }
        };
        shared_length
    }

    fn insert_or_replace_existing(v: &mut Vec<Box<Self>>, path: &str, method: &Method, new: Self) {
        for (idx, child) in v.iter_mut().enumerate() {
            if let Node::Terminal(m, _) = &**child {
                if m == method {
                    std::mem::swap(child, &mut Box::new(new));
                    warn!("overwriting handler with {} {}", path, method);
                    return;
                } else if method > m {
                    v.insert(idx, Box::new(new));
                    return;
                }
            }
        }
        v.push(Box::new(new));
    }

    fn _print(&self, indent:usize) {
        let mut indent_str = "".to_string();
        for _ in 0..indent {
            indent_str.push(' ');
        }
        match self {
            Node::Passby(word, children) => {
                print!("- {}\"{}\"\n", indent_str, word);
                for child in children.iter() {
                    child._print(indent + 2);
                }
            },
            Node::Terminal(method, _) => {
                print!("- {}{}\n", indent_str, method);
            }
        }
    }
}

#[derive(Clone)]
pub struct Trie {
    root: Node,
}

impl Default for Trie {
    fn default() -> Self {
        Self {
            root: Node::Passby(String::new(), vec![]),
        }
    }
}

impl Trie {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, p: &str, m: &Method, h: &HandlerRef) {
        trace!("http handler inserted: {} {}", m, p);
        self.root.attach(p, 0, m, h)
    }

    pub fn get(&self, p: &str, m: &Method) -> Option<HandlerRef> {
        self.root.get(p, m)
    }

    pub fn _print(&self) {
        self.root._print(0);
    }
}


#[cfg(test)]
mod trie_tests {
    use {
        super::*,
        std::sync::{Arc, Mutex},
        super::super::{
            req::Req,
            res::Res,
        },
        crate::server::http::handler::Handle,
    };

    #[test]
    fn node_can_attach() {
        let hand_ref_get_hero = get_handler_ref(2);
        let mut root_node = get_node_with_whenwhere();
        root_node.attach("what", 0, &Method::GET, &hand_ref_get_hero);

        let mut buf = std::io::BufReader::new("GET /index.html HTTP/1.1\r\nHost: www.xiwen.com\r\nAccept-Language: en-us\r\nContent-Length: 5\r\n\r\nHello".as_bytes());
        let mut incoming_req = Req::new(&mut buf).unwrap();

        let mut write_buf: Vec<u8> = vec![];
        let mut res = Res::new(&mut write_buf);

        let mut retrieved_handler = root_node.get("what", &Method::GET).unwrap();
        retrieved_handler.handle(&mut incoming_req, &mut res);
        assert_eq!(write_buf.as_slice(), "HTTP/1.x 200 OK\r\nContent-Length: 17\r\n\r\nsample handler #2".as_bytes());
    }

    #[test]
    fn trie_can_insert_and_get() {
        let mut tr = Trie::default();
        let hand_ref = get_handler_ref(1);
        tr.insert("hello/world", &Method::GET, &hand_ref);
        let h = tr.get("hello/world", &Method::GET);
        assert!(h.is_some());
    }

    #[test]
    fn node_return_none_when_nothing_found() {
        let n = get_node_with_whenwhere();
        assert!(n.get("/wowow", &Method::GET).is_none());
    }

    fn get_handler_ref(index: u32) -> HandlerRef {
        Arc::new(Mutex::new(move|_: &mut Req, res: &mut Res| {
            res.respond(format!("sample handler #{}", index).as_bytes()).unwrap();
        }))
    }

    fn get_node_with_whenwhere() -> Node {
        let hand_ref_get_when = get_handler_ref(0);
        let hand_ref_post_where = get_handler_ref(1);
        Node::Passby(
            "".to_string(),
            vec![
                Box::new(
                    Node::Passby(
                        "whe".to_string(),
                        vec![
                            Box::new(Node::Passby("n".to_string(), vec![
                                Box::new(Node::Terminal(Method::GET, hand_ref_get_when)),
                            ])),
                            Box::new(Node::Passby("re".to_string(), vec![
                                Box::new(Node::Terminal(Method::POST, hand_ref_post_where)),
                            ])),
                        ],
                    )                    
                ),
            ]
        )

    }
}