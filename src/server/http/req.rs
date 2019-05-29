
use std::{
    io,
    io::Read,
    collections,
    net,
};

use super::method;

const MAX_HTTP_LINE_LENGTH: usize = 4096;

pub struct Req {
    method: method::Method,
    path: String,
    version: String,
    headers: collections::HashMap<String, String>,
    params: collections::HashMap<String, String>,
}

impl Req {
    pub fn new(s: &mut net::TcpStream) -> io::Result<Self> {
        let mut req = Req::default();
        let first_line = read_until_new_line(s)?;
        let mut iter = first_line.split_whitespace();
        if let Some(mstr) = iter.next() {
            match method::Method::from(mstr) {
                method::Method::UNKNOWN => return Err(io::Error::new(io::ErrorKind::InvalidData, "tcp stream doesn't have valid http method")),
                m => { req.method = m}
            }
        } else {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "tcp stream doesn't have valid http first line"));
        }
        
        if let Some(path) = iter.next() {
            req.path = path.to_string();
        }

        if let Some(version) = iter.next() {
            req.version = version.to_string();
        }
        req.parse_headers(s)?;

        Ok(req)
    }

    fn parse_headers(&mut self, stream: &mut net::TcpStream) -> io::Result<()> {
        loop {
            let line = read_until_new_line(stream)?;
            let mut next_two = [0u8;2];
            stream.peek(&mut next_two)?;
            let (k, v) = split_header_line(line);
            self.headers.insert(k, v);
            if next_two == [13, 10] {
                stream.read(&mut next_two)?;
                return Ok(());
            }
        }
    }

    pub fn method(&self) -> &method::Method {
        &self.method
    }

    pub fn path(&self) -> &String {
        &self.path
    }

    // fn parse_params(&mut self, line: &str) {
    //     // TODO: implement
    // }
}

impl Default for Req {
    fn default()->Self {
        Self {
            method: method::Method::GET,
            path: "/".to_string(),
            version: "HTTP/1.1".to_string(),
            headers: collections::HashMap::default(),
            params: collections::HashMap::default(),
        }
    }
}

fn read_until_new_line(s: &mut net::TcpStream) -> io::Result<String> {
    let mut res = String::with_capacity(256);
    loop {
        let mut b = [0u8];
        s.read(&mut b)?;
        if b[0] == '\r' as u8 {
            let mut bb = [0u8];
            s.peek(&mut bb)?;
            if bb[0] == '\n' as u8 {
                s.read(&mut bb)?;
                break;
            }
        }
        if res.len() > MAX_HTTP_LINE_LENGTH { 
            return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("HTTP header line length exceed maximum({})", MAX_HTTP_LINE_LENGTH),
        ))}
        res.push(b[0] as char);
    }
    Ok(res)
}

fn split_header_line(line: String) -> (String, String) {
    let mut k = String::new();
    let mut v = String::new();
    for (i, x) in line.splitn(2, ":").enumerate() {
        if i == 0 { k = String::from(x.trim()); }
        else { v = String::from(x.trim()); }
    }
    (k,v)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        io,
        io::Write,
        net::{TcpListener, TcpStream},
        thread,
        time,
    };

    const HTTP_REQ_STR: &str = "GET /index.html HTTP/1.1\r\nHost: www.xiwen.com\r\nAccept-Language: en-us\r\nContent-Length: 5\r\n\r\nHello";

    #[test]
    fn test_http_header() -> io::Result<()> {
        let (server, port) = get_tcpserver_and_port()?;
        thread::spawn(move || {
            let mut client = TcpStream::connect(format!("127.0.0.1:{}", port)).unwrap();
            thread::sleep(time::Duration::from_millis(300));
            client.write(HTTP_REQ_STR.as_bytes()).unwrap();
        });
        let (mut conn, _) = server.accept()?;
        let req = Req::new(&mut conn)?;
        assert_eq!(req.method, method::Method::GET);
        assert_eq!(req.path, "/index.html");
        assert_eq!(req.version, "HTTP/1.1");
        assert_eq!(req.headers.len(), 3);
        assert!(req.headers.contains_key("Host"));
        assert!(req.headers.contains_key("Accept-Language"));
        assert!(req.headers.contains_key("Host"));
        assert_eq!(req.headers.get("Host").cloned(), Some(String::from("www.xiwen.com")));
        Ok(())
    }

    fn get_tcpserver_and_port() -> std::io::Result<(TcpListener, i32)> {
        let mut server: TcpListener;
        let mut port = 10000;
        loop {
            match TcpListener::bind(format!("127.0.0.1:{}", port)) {
                Ok(s) => {
                    server = s;
                    break;
                },
                Err(e) => {
                    if port != 60000 {
                        port = port + 1;
                        continue;
                    }
                    return Err(e);
                },
            }
        }
        Ok((server, port))
    }
}