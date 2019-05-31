use std::{
    collections,
    io,
};

pub struct Res<'a> {
    version: String,
    status_code: u16,
    status: String,
    headers: collections::HashMap<String, String>,
    body: &'a mut io::Write,
}

#[allow(dead_code)]
impl<'a> Res<'a> {
    pub fn new(w: &'a mut io::Write) -> Self {
        Self {
            version: String::from("HTTP/1.x"),
            status_code: 200,
            status: String::from("OK"),
            headers: collections::HashMap::new(),
            body: w,
        }
    }

    pub fn set_status(&mut self, status_code: u16, status: &'static str) {
        self.status_code = status_code;
        self.status = status.to_string();
    }

    pub fn status(&self) -> &String {
        &self.status
    }

    pub fn respond(&mut self, body: &[u8]) -> io::Result<()> {
        self.body.write(format!("{} {} {}\r\n", self.version, self.status_code, self.status).as_bytes())?;
        for (key, value) in self.headers.iter() {
            self.body.write(format!("{}: {}\r\n", key, value).as_bytes())?;
        }
        self.body.write(b"\r\n")?;
        self.body.write(body)?;
        self.body.flush()?;
        Ok(())
    }
}