
use {
    std::{
        collections,
        io,
    },
    super::headers::*,
    crate::logger::micro::*,
};

pub struct Res<'a> {
    version: String,
    status_code: u16,
    status: String,
    headers: collections::HashMap<String, String>,
    body_writer: &'a mut io::Write,
    responded: bool,
}

impl<'a> Res<'a> {
    pub fn new(w: &'a mut io::Write) -> Self {
        Self {
            version: String::from("HTTP/1.x"),
            status_code: 200,
            status: String::from("OK"),
            headers: collections::HashMap::new(),
            body_writer: w,
            responded: false,
        }
    }

    pub fn set_status(&mut self, status_code: u16, status: &'static str) {
        self.status_code = status_code;
        self.status = status.to_string();
    }

    pub fn respond(&mut self, content: &[u8]) -> io::Result<()> {
        if self.responded {
            return Err(io::Error::new(io::ErrorKind::Other, "HTTP Already responded"));
        }
        self.body_writer.write(format!("{} {} {}\r\n", self.version, self.status_code, self.status).as_bytes())?;
        self.body_writer.write(format!("{}: {}\r\n", HTTP_HEADER_CONTENT_LENGTH, content.len()).as_bytes())?;
        for (key, value) in self.headers.iter() {
            self.body_writer.write(format!("{}: {}\r\n", key, value).as_bytes())?;
        }
        self.body_writer.write(b"\r\n")?;
        self.body_writer.write(content)?;
        self.body_writer.flush()?;
        self.responded = true;
        debug!("HTTP responded {} with {} bytes", self.status_code, content.len());
        Ok(())
    }

    // getters:
    #[allow(unused)]
    pub fn status(&self) -> &String {
        &self.status
    }
    pub fn responded(&self) -> bool {
        self.responded
    }
}