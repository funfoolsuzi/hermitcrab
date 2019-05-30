
use std::{
    net,
    sync::mpsc,
    thread,
    time,
    io,
    fmt,
};

use super::super::super::logger::micro::*;

const LINE_STREAM_TIMEOUT_SECS: u64 = 10;
const SYNC_CHANNEL_BUFFER_SIZE: usize = 2;

#[derive(Debug, PartialEq)]
pub enum SendError {
    LineBusy,
    Disconnected,
}

impl fmt::Display for SendError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SendError::LineBusy => write!(f, "SendError:LineBusy"),
            SendError::Disconnected => write!(f, "SendError::DisConnected"),
        }
    }
}

pub struct Line {
    s: mpsc::SyncSender<Option<net::TcpStream>>,
    // TODO: add timestamp
}

impl Line {
    pub fn new(mut stream_handler: impl FnMut(net::TcpStream) -> io::Result<()> + Send + Sync + 'static) -> Self {
        let (s, r) = mpsc::sync_channel::<Option<net::TcpStream>>(SYNC_CHANNEL_BUFFER_SIZE);
        thread::spawn(move || {
            for stream in r {
                if let Some(st) = stream {
                    let t = Some(time::Duration::from_secs(LINE_STREAM_TIMEOUT_SECS));
                    st.set_read_timeout(t);
                    st.set_write_timeout(t);
                    stream_handler(st);
                    // TODO: handle result
                } else {
                    break;
                }
            }
        });
        Self {
            s,
        }
    }

    pub fn send(&mut self, stream: net::TcpStream) -> Result<(), (net::TcpStream, SendError)> {
        self.s.try_send(Some(stream)).map_err(|e| {
            match e {
                mpsc::TrySendError::Full(s) => (s.unwrap(), SendError::LineBusy),
                mpsc::TrySendError::Disconnected(s) => (s.unwrap(), SendError::Disconnected),
            }
        })
    }


}

impl Drop for Line {
    fn drop(&mut self) {
        self.s.send(None).unwrap_or_else(|e|{
            error!("Failed to drop a processing line. {}", e)
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        io::{Read, Write},
        sync::{Mutex,Arc},
        net::{TcpListener,TcpStream},
        time,
    };

    #[test]
    fn handler_can_mutate_environment() -> std::io::Result<()> {
        let (server, server_port) = get_tcpserver_and_port()?;

        // handler closure and its captured values
        let buf = Arc::new(Mutex::new(vec![0u8]));
        let buf_ref = buf.clone();
        let mut l = Line::new(move |mut stream: net::TcpStream| {
            let mut buf_guard = buf_ref.lock().unwrap();
            buf_guard.clear();

            let mut tempbuf = [0u8;3];
            assert_eq!(stream.read(&mut tempbuf).unwrap_or(0), 3);

            for b in &tempbuf {
                buf_guard.push(*b);
            }
            
            Ok(())
        });

        // first client connect & send a message
        let mut client1 = TcpStream::connect(format!("127.0.0.1:{}", server_port))?;
        let (conn1, _) = server.accept()?;
        l.send(conn1).unwrap();
        client1.write("abc".as_bytes())?;
        thread::sleep(time::Duration::from_millis(300)); // server takes time to modify the target
        assert_eq!(buf.lock().unwrap().as_slice(), "abc".as_bytes());

        Ok(())
    }

    #[test]
    fn receive_busy_while_processing_stream() {
        let (server, server_port) = get_tcpserver_and_port().unwrap();
        
        let mut l = Line::new(move |_| {
            thread::sleep(time::Duration::from_millis(600));
            Ok(())
        });

        TcpStream::connect(format!("127.0.0.1:{}", server_port)).unwrap();
        TcpStream::connect(format!("127.0.0.1:{}", server_port)).unwrap();
        TcpStream::connect(format!("127.0.0.1:{}", server_port)).unwrap();
        let (conn1, _) = server.accept().unwrap();
        let (conn2, _) = server.accept().unwrap();
        let (conn3, _) = server.accept().unwrap();
        assert!(l.send(conn1).is_ok());
        thread::sleep(time::Duration::from_millis(100));
        assert!(l.send(conn2).is_ok());
        thread::sleep(time::Duration::from_millis(100));
        assert!(l.send(conn3).is_ok());

        thread::sleep(time::Duration::from_millis(300));
        TcpStream::connect(format!("127.0.0.1:{}", server_port)).unwrap();
        let (conn4, _) = server.accept().unwrap();
        assert_eq!(l.send(conn4).map_err(|(_,e)| e), Err(SendError::LineBusy));

        thread::sleep(time::Duration::from_millis(600));
        TcpStream::connect(format!("127.0.0.1:{}", server_port)).unwrap();
        let (conn5, _) = server.accept().unwrap();
        assert!(l.send(conn5).is_ok());
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