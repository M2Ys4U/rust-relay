use std::io::{IoResult, Stream, Writer};
use std::io::net::ip::ToSocketAddr;
use std::io::net::tcp::TcpStream;
use std::time::duration::Duration;

use irc_methods::IrcMethods;
use message::Message;
use parser::Parser;

#[unstable]
pub struct Connection<T: Stream + Clone> {
    stream: T,
    parser: Parser<T>,
}

#[unstable]
impl<T: Stream + Clone> Connection<T> {
    #[stable]
    pub fn new(stream: T) -> Connection<T> {
        Connection {
            stream: stream.clone(),
            parser: Parser::new(stream),
        }
    }

    #[experimental]
    #[inline]
    pub fn get_stream<'a>(&'a mut self) -> &'a mut T {
        &mut self.stream
    }

    #[unstable]
    #[inline]
    pub fn read_message(&mut self) -> IoResult<Option<Message<'static>>> {
        self.parser.read_message()
    }
}

impl<A: ToSocketAddr> Connection<TcpStream> {
    #[stable]
    #[inline]
    pub fn connect_to(addr: A) -> IoResult<Connection<TcpStream>> {
        //TcpStream::connect(addr).map(Connection::new)
        match TcpStream::connect(addr) {
            Ok(stream) => Ok(Connection::new(stream)),
            Err(e) => Err(e)
        }
    }

    #[stable]
    #[inline]
    pub fn connect_to_timeout(addr: A, timeout: Duration) -> IoResult<Connection<TcpStream>> {
        TcpStream::connect_timeout(addr, timeout).map(Connection::new)
    }
}

#[experimental]
impl IrcMethods for Connection<TcpStream> {
    fn quit(&mut self, message: Option<&str>) -> IoResult<()> {
        self.__quit(message).and_then(|_| self.flush()).and_then(|_| self.stream.close_write()).and_then(|_| self.stream.close_read())
    }
}

#[stable]
impl<T: Stream + Clone> Writer for Connection<T> {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> IoResult<()> {
        self.stream.write(buf)
    }

    #[inline]
    fn flush(&mut self) -> IoResult<()> {
        self.stream.flush()
    }
}
