
use std::{
    io,
    collections,
};

use super::method;
use crate::logger::help::*;

const MAX_HTTP_HEADER_LINE_LENGTH: usize = 4096;

pub struct Req<'a> {
    method: method::Method,
    path: String,
    version: String,
    headers: collections::HashMap<String, String>,
    body: &'a mut io::BufRead,
    // params: collections::HashMap<String, String>,
}

impl<'a> Req<'a> {
    pub fn new(s: &'a mut io::BufRead) -> io::Result<Self> {
        let mut req = Req {
            method: method::Method::UNKNOWN,
            path: String::new(),
            version: String::new(),
            headers: collections::HashMap::new(),
            body: s,
        };
        let first_line = read_new_line(req.body)?;
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
        
        req.parse_headers()?;

        Ok(req)
    }

    fn parse_headers(&mut self) -> io::Result<Option<()>> {
        let line = read_new_line(self.body)?;
        if line.is_empty() {
            return Ok(None);
        }
        let (k, v) = split_header_line(line);
        trace!("header parsed {}: {}", k, v);
        self.headers.insert(k, v);
        self.parse_headers()
    }

    pub fn method(&self) -> &method::Method {
        &self.method
    }

    pub fn path(&self) -> &String {
        &self.path
    }
}

fn read_new_line(s: &mut io::BufRead) -> io::Result<String> {
    let mut res = String::new();
    s.read_line(&mut res)?;

    if res.len() < 2 {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "each line should at least contain \"\\r\\n\" at the end"));
    }
    
    while res.as_str()[res.len()-2..] != *"\r\n" {
        if res.len() > MAX_HTTP_HEADER_LINE_LENGTH {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "exceeding max header line limit"));
        }
        let mut additional = String::new();
        s.read_line(&mut additional)?;
        res += additional.as_str();
    }
    res.pop();
    res.pop();
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
    };

    const HTTP_REQ_STR: &str = "GET /index.html HTTP/1.1\r\nHost: www.xiwen.com\r\nAccept-Language: en-us\r\nContent-Length: 5\r\n\r\nHello";

    #[test]
    fn test_http_header() -> io::Result<()> {
        let http_req = String::from(HTTP_REQ_STR);
        let mut buf = io::BufReader::new(http_req.as_bytes());
        let req = Req::new(&mut buf)?;
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
}