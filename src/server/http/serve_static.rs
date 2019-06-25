use {
    std::{io, path, fs, sync},
    super::{
        trie::Trie,
        method::Method,
        req::Req,
        res::Res,
        handler::HandlerRef,
        headers::*,
    },
    crate::logger::micro::*,
};

pub fn add_directory_to_trie(prefix: &path::Path, dir: &path::Path, trie: &mut Trie) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                let _new_prefix = path.file_name();
                let new_prefix = path.strip_prefix(dir).ok()
                    .map(|p| prefix.join(p))
                    .ok_or(io::Error::new(io::ErrorKind::Other, "Failed to add directory to trie"))?;
                add_directory_to_trie(&new_prefix, &path, trie)?;
            } else {
                add_file_to_trie(&prefix, &path, trie)?;
            }
        }
    }
    return Ok(());
}

fn add_file_to_trie(prefix: &path::Path, file: &path::Path, trie: &mut Trie) -> io::Result<()> {
    println!("debug add_file_to_trie {} {}", prefix.display(), file.display());
    let data = fs::read(file)?;
    let owned_file = file.to_owned();
    let p = file.file_name()
        .map(|fname| prefix.join(fname))
        .and_then(|p| p.to_str().map(|s| s.to_owned()))
        .ok_or(io::Error::new(io::ErrorKind::InvalidData, format!("Path: {} can't be converted to utf8", file.display())))
        .unwrap();
    let hr: HandlerRef = sync::Arc::new(sync::Mutex::new(move|_: &mut Req, res:&mut Res| {
        if let Err(e) = res.respond(data.as_slice()) {
            error!("failed to respond static file: {} error: {}", owned_file.display(), e);
        }
    }));
    println!("debug insert_to_trie {}", p);
    trie.insert(&p, &Method::GET, &hr);
    return Ok(())
}


#[cfg(test)]
mod serve_static_tests {
    use {
        super::*,
        super::super::Handle,
    };

    #[test]
    fn test_static() {
        let mut t = Trie::default();
        let p = path::Path::new("./test_data");
        assert!(add_directory_to_trie(path::Path::new("/"), &p, &mut t).is_ok());
        t._print();
        let mut css_handler = t.get("/assets/main.css", &Method::GET).unwrap();

        let mut buf = std::io::BufReader::new("GET /assets/main.css HTTP/1.1\r\nHost: www.xiwen.com\r\nAccept-Language: en-us\r\nContent-Length: 5\r\n\r\nHello".as_bytes());
        let mut req = Req::new(&mut buf).unwrap();

        let mut write_buf: Vec<u8> = vec![];
        let mut res = Res::new(&mut write_buf);

        css_handler.handle(&mut req, &mut res);
        let css = String::from_utf8(write_buf.clone()).unwrap();
        println!("css:\n {}", css);
        assert_eq!(write_buf.as_slice(), "HTTP/1.x 200 OK\r\nContent-Length: 33\r\n\r\n#main-title {\n    color: green;\n}".as_bytes());
    }
}