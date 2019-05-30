use std::{
    collections,
    io,
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
    #[allow(unused)]
    pub fn set_status(&mut self, status_code: u16, status: &'static str) {
        self.status_code = status_code;
        self.status = status.to_string();
    }

    #[allow(unused)]
    pub fn status(&self) -> &String {
        &self.status
    }

    pub fn respond(&self, writer: &mut io::Write, body: &[u8]) -> io::Result<()> {
        writer.write(format!("{} {} {}\r\n", self.version, self.status_code, self.status).as_bytes())?;
        for (key, value) in self.headers.iter() {
            writer.write(format!("{}: {}\r\n", key, value).as_bytes())?;
        }
        writer.write(b"\r\n")?;
        writer.write(body)?;
        writer.flush()?;
        Ok(())
    }
}