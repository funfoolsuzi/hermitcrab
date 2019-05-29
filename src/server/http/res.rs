use std::{
    collections
};

pub struct Res {
    version: String,
    status_code: u16,
    status: String,
    headers: collections::HashMap<String, String>,
}

impl Default for Res {
    fn default() -> Self {
        Res {
            version: String::from("HTTP/1.x"),
            status_code: 200,
            status: String::from("OK"),
            headers: collections::HashMap::new(),
        }
    }
}

impl Res {
    pub fn set_status(&mut self, status_code: u16, status: &'static str) {
        self.status_code = status_code;
        self.status = status.to_string();
    }

    pub fn status(&self) -> String {
        self.status.clone()
    }
}