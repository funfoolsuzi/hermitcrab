use {
    super::{
        matcher::HandlerRef,
        method::Method,
        res::Res,
    },
};

#[derive(Clone)]
enum Node {
    Passby(String, Vec<Box<Node>>),
    Terminal(Method, HandlerRef),
}

impl Node {
    fn append(&mut self, remain: &str, method: &Method, handler: &HandlerRef) {
        if let Node::Passby(_, children) = self {
            if remain.is_empty() {
                let mut swap_idx: Option<usize> = None;
                let mut insert_idx: Option<usize> = None;
                for (idx, child) in children.iter_mut().enumerate() {
                    if let Node::Terminal(m, h) = &**child {
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
                    return;
                    // log warning
                }
                children.push(Box::new(Node::Terminal(*method, handler.clone())));
                return;
            }
            let mut insertion_idx: Option<usize> = None;
            'children: for (idx, child) in children.iter_mut().enumerate() {
                let mut child_append_str: Option<String> = None;
                if let Node::Passby(word, grandchildren) = &mut **child {
                    let mut shared_length = 0usize;
                    let mut word_chars = word.chars().peekable();
                    let mut remain_chars = remain.chars().peekable();
                    loop {
                        if let (Some(r_char), Some(cw_char)) = (word_chars.peek(), remain_chars.peek()) {
                            if r_char != cw_char {
                                break;
                            }
                            word_chars.next();
                            remain_chars.next();
                            shared_length += 1;
                        } else {
                            break;
                        }
                    };
                    if shared_length == 0 {
                        if child.is_ahead_of(remain) {
                            insertion_idx = Some(idx);
                            break
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
                    for _ in 1..shared_length {
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
            }
        }
    }

    fn get(&self, path: &str, method: &Method) -> HandlerRef {
        std::sync::Arc::new(std::sync::Mutex::new(|_: &mut super::req::Req, _: &mut Res| {
        }))
    }

    fn is_ahead_of(&self, target: &str) -> bool {
        if let Node::Passby(word, _) = self {
            return word.as_str() > target;
        }
        false
    }

    fn print(&self, indent:usize) {
        let mut indent_str = "".to_string();
        for _ in 0..indent {
            indent_str.push(' ');
        }
        match self {
            Node::Passby(word, children) => {
                print!("{}{} - ", indent_str, word);
                for (idx, child) in children.iter().enumerate() {
                    if idx == 0 {
                        child.print(0);
                    } else {
                        child.print(indent + word.len() + 3);
                    }
                }
            },
            Node::Terminal(method, _) => {
                print!("{}{}\n", indent_str, method);
            }
        }
    }
}

#[derive(Default, Clone)]
struct Trie {
    root: Option<Node>,
}

impl Trie {
    // pub fn insert(&mut self, m: &Method, p: &str, h: &HandlerRef) {
    //     match &self.root {
    //         None => {
    //             let mut n = Node{
    //                 key: Key::Word(p.to_string()),
    //                 handler_ref: None,
    //                 children: vec![],
    //             };
    //             n.append_terminal(m, h);
    //             self.root = Some(n);
    //         },
    //         Some(node) => {

    //         },
    //     }
    // }

    // pub fn get(&self, m: Method, p: &str) -> HandlerRef {

    // }
}


#[cfg(test)]
mod trie_tests {
    use {
        super::*,
        std::sync::{Arc, Mutex},
        super::super::{
            req::Req,
        },
    };

    #[test]
    fn playground() {
        println!("size: {}", std::mem::size_of::<Option<Box<Node>>>());
        let mut w: String = "wow".to_string();
        w.pop();

        println!("word: {}", w);
    }

    #[test]
    fn node_can_append() {
        let hand_ref_get_hero = get_handler_ref(2);
        let mut root_node = get_node_with_whenwhere();
        root_node.append("what", &Method::GET, &hand_ref_get_hero);
        //let retrieved_handler = root_node.get("what", &Method::GET);
        root_node.print(0);
    }

    #[test]
    fn can_insert_and_get() {
        let mut tr = Trie::default();
        let hand_ref = get_handler_ref(1);
        // tr.insert(Method::GET, "hello/world", hand_ref);
        // let h = tr.get(Method::Get, "hello/word");
        // assert_eq!(h, hand_ref);
    }

    // #[test]
    // fn can_get_shared_length() {
    //     let len_a = Node::get_shared_length("/where", "/wh");
    //     assert_eq!(len_a, 3);

    //     let len_b = Node::get_shared_length("/where", "/what");
    //     assert_eq!(len_b, 3);
    // }

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