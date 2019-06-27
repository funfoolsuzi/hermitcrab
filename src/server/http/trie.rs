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
    fn append2(&mut self, p: &path::Path, p_begin: usize, p_end: usize, m: &Method, handler: &HandlerRef) {

    }
    fn append(&mut self, remain: &str, method: &Method, handler: &HandlerRef) {
        if let Node::Passby(_, children) = self {
            if remain.is_empty() {
                let mut swap_idx: Option<usize> = None;
                let mut insert_idx: Option<usize> = None;
                for (idx, child) in children.iter_mut().enumerate() {
                    if let Node::Terminal(m, _) = &**child {
                        if m == method {
                            swap_idx = Some(idx);
                            break;
                        } else if method > m {
                            insert_idx = Some(idx);
                            break;
                        }
                    }
                }
                if let Some(idx) = swap_idx {
                    children.push(Box::new(Node::Terminal(*method, handler.clone())));
                    children.swap_remove(idx);
                    return;
                }
                if let Some(idx) = insert_idx {
                    children.insert(idx, Box::new(Node::Terminal(*method, handler.clone())));
                    warn!("overwriting handler with {} {}", remain, method);
                    return;
                }
                children.push(Box::new(Node::Terminal(*method, handler.clone())));
                return;
            }
            let mut insertion_idx: Option<usize> = None;
            let children_len = children.len();
            if children_len == 0 {
                insertion_idx = Some(0);
            }
            for (idx, child) in children.iter_mut().enumerate() {
                let mut child_append_str: Option<String> = None;
                if let Node::Passby(word, grandchildren) = &mut **child {
                    let mut word_chars = word.chars().peekable();
                    let mut remain_chars = remain.chars().peekable();
                    let shared_length = Node::iter_over_shared_chars(&mut word_chars, &mut remain_chars);
                    if shared_length == 0 {
                        if child.is_ahead_of(remain) {
                            insertion_idx = Some(idx);
                            break
                        } else if idx == children_len-1 {
                            insertion_idx = Some(idx + 1);
                        }
                        continue;                                
                    }
                    let word_2nd_half = word_chars.collect::<String>();
                    if word_2nd_half.len() == 0 {
                        child.append(&remain_chars.collect::<String>(), method, handler);
                        return
                    }
                    let new_passby = Node::Passby(word_2nd_half, grandchildren.clone());
                    child_append_str = Some(remain_chars.collect());
                    grandchildren.clear();
                    grandchildren.push(Box::new(new_passby));
                    for _ in 0..(word.len() - shared_length) {
                        word.pop();
                    }
                }
                if let Some(append_str) = child_append_str {
                    child.append(&append_str, method, handler);
                    break;
                }
            }
            if let Some(idx) = insertion_idx {
                let mut new_passby = Node::Passby(remain.to_string(), vec![]);
                new_passby.append("", method, handler);
                children.insert(idx, Box::new(new_passby));
                return
            }

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
                Node::iter_over_shared_chars(&mut word_chars, &mut remain_chars);
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

    fn _print(&self, indent:usize) {
        let mut indent_str = "".to_string();
        for _ in 0..indent {
            indent_str.push(' ');
        }
        match self {
            Node::Passby(word, children) => {
                print!("- {}{}\n", indent_str, word);
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
        self.root.append(p, m, h)
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
    fn node_play_append2() {
        let mut n = get_node_with_whenwhere();
        let handler_here = get_handler_ref(2);
        n.append2(path::Path::new("/here"), 0, 5, &Method::GET, &handler_here);
    }

    #[test]
    fn node_can_append() {
        let hand_ref_get_hero = get_handler_ref(2);
        let mut root_node = get_node_with_whenwhere();
        root_node.append("what", &Method::GET, &hand_ref_get_hero);

        let mut buf = std::io::BufReader::new("GET /index.html HTTP/1.1\r\nHost: www.xiwen.com\r\nAccept-Language: en-us\r\nContent-Length: 5\r\n\r\nHello".as_bytes());
        let mut incoming_req = Req::new(&mut buf).unwrap();

        let mut write_buf: Vec<u8> = vec![];
        let mut res = Res::new(&mut write_buf);

        let mut retrieved_handler = root_node.get("what", &Method::GET).unwrap();
        retrieved_handler.handle(&mut incoming_req, &mut res);
        assert_eq!(write_buf.as_slice(), "HTTP/1.x 200 OK\r\nContent-Length: 17\r\n\r\nsample handler #2".as_bytes());
    }

    #[test]
    fn node_can_insert_and_get() {
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