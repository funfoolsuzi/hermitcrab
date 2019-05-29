
extern crate log;

use std::{
    io,
    sync,
    sync::{mpsc},
    thread,
};

use log::{Record, Metadata};
pub use log::{
    Log,
    Level,
    SetLoggerError, LevelFilter,
    set_boxed_logger, set_max_level,
};

pub mod micro {
    pub use super::log::{trace, debug, error, warn, info};
}

const DROP_MSG: &str = "!!!xxx_dropping_logger_xxx!!!";

pub struct HcLogger{
    sender: mpsc::SyncSender<String>,
    jhand: Option<thread::JoinHandle<()>>,
    level: Level,
}

impl HcLogger {
    pub fn new<T: io::Write + Send + Sync + 'static>(buf_size: usize, destination: T, level: Level) -> Self {
        let (tx, rx) = mpsc::sync_channel(buf_size);
        let d = sync::Arc::new(sync::Mutex::new(destination));
        let jh = thread::spawn(move || {
            loop {
                let r = match rx.recv() {
                    Ok(msg) => {
                        let msg_str = msg as String;
                        if msg_str.as_str() == DROP_MSG {
                            break;
                        }
                        d.lock().unwrap().write((msg_str + "\n").as_bytes())
                    },
                    Err(e) => {
                        let msg = format!("Log error:\n*****\n{}\n*****\nLogger exiting\n", e);
                        d.lock().unwrap().write(msg.as_bytes())
                    },
                };
                match r {
                    Err(e) => {println!("logger error: {}", e);},
                    Ok(_) => {},
                }
            }
        });
        Self {
            sender: tx,
            jhand: Some(jh),
            level: level,
        }
    }
}

impl log::Log for HcLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            match self.sender.send(format!("[{}] {}", record.level(), record.args())){
                Err(e) => {println!("logger error: {}", e);},
                Ok(_) => {},
            };
        }
    }

    fn flush(&self) {
    }
}

impl Drop for HcLogger {
    #[allow(unused)]
    fn drop(&mut self) {
        self.sender.send(String::from(DROP_MSG));
        self.jhand.take().unwrap().join();
    }
}

pub fn init_stdout_logger(msg_buffer_size: usize, level: Level) -> Result<(), SetLoggerError> {
    set_boxed_logger(Box::new(HcLogger::new(msg_buffer_size, io::stdout(), level))).map(|()|{
        set_max_level(level.to_level_filter());
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::micro::*;
    use std::{
        io,
        time,
    };


    #[derive(Default)]
    struct TestWriter {
        content: sync::Arc<sync::Mutex<String>>,
    }

    impl io::Write for TestWriter {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            let mut c = self.content.lock().unwrap();
            let s1 = c.len();
            *c += &String::from_utf8_lossy(buf);
            Ok(c.len() - s1)
        }

        fn flush(&mut self) -> io::Result<()> {
            let mut c = self.content.lock().unwrap();
            c.clear();
            Ok(())
        }
    }

    impl TestWriter {
        fn get_content_spy(&self) -> sync::Arc<sync::Mutex<String>> {
            self.content.clone()
        }
    }

    #[test]
    fn can_create() {
        HcLogger::new(100, io::stdout(), Level::Debug);
    }

    #[test]
    fn can_send() {
        let writer = TestWriter::default();
        let spy = writer.get_content_spy();
        let logger = HcLogger::new(10, writer, Level::Info);

        logger.log(&Record::builder().args(format_args!("hello")).level(Level::Info).build());
        thread::sleep(time::Duration::from_millis(200));

        {
            assert_eq!(*spy.lock().unwrap(), String::from("[INFO] hello\n"));
        }
    }

    #[allow(unused)]
    #[test]
    fn can_init_logger() {
        let writer = TestWriter::default();
        let spy = writer.get_content_spy();
        let logger = HcLogger::new(10, writer, Level::Debug);
        set_boxed_logger(Box::new(logger)).map(|()| {
            log::set_max_level(LevelFilter::Info);
        });

        info!("hiii");
        thread::sleep(time::Duration::from_millis(200));

        {
            assert_eq!(*spy.lock().unwrap(), String::from("[INFO] hiii\n"));
        }
    }
}