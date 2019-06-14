use std::{
    fmt,
};

#[derive(Copy, Clone, PartialEq, Debug, Eq, Hash, PartialOrd)]
pub enum Method {
    GET,
    POST,
    PUT,
    DELETE,
    UNKNOWN,
}

impl From<&str> for Method {
    fn from(method_str: &str) -> Self {
        match method_str {
            "GET" => Method::GET,
            "POST" => Method::POST,
            "PUT" => Method::PUT,
            "DELETE" => Method::DELETE,
            _ => Method::UNKNOWN,
        }
    }
}

impl Default for Method {
    fn default() -> Self {Method::UNKNOWN}
}

impl fmt::Display for Method {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Method::GET => write!(f, "GET"),
            Method::POST => write!(f, "POST"),
            Method::PUT => write!(f, "PUT"),
            Method::DELETE => write!(f, "DELETE"),
            Method::UNKNOWN => write!(f, "UNKNOWN"),
        }
    }
}