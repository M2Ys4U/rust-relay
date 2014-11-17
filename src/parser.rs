use std::io::{IoError, IoErrorKind, IoResult, Reader};
use std::slice::bytes::{copy_memory, MutableByteVector};

#[cfg(test)]
use std::io::MemReader;
#[cfg(test)]
use std::str::Slice;

use message::{Message, MessageErr};

#[experimental]
pub struct Parser<T: Reader> {
    buffer: [u8, ..1024],
    buffer_length: uint,
    reader: T
}

#[unstable]
impl<T: Reader> Parser<T> {
    #[unstable]
    pub fn new(reader: T) -> Parser<T> {
        Parser {
            buffer: [0u8, ..1024],
            buffer_length: 0,
            reader: reader
        }
    }

    #[experimental]
    fn read_message_from_buffer(&mut self) -> IoResult<Option<Message<'static>>> {
        let mut start: uint = 0;
        let mut msg: Option<Message> = None;

        for i in range(0, self.buffer_length) {
            if start < self.buffer_length && (self.buffer[i] == 13u8 || self.buffer[i] == 10u8) {
                match Message::from_str(String::from_utf8_lossy(self.buffer.slice_mut(start, i)).as_slice()) {
                    Ok(m) => {
                        msg = Some(m);
                        start = i;
                        break;
                    },
                    Err(e) => {
                        match e {
                            MessageErr::MalformedInput => {
                                return Err(IoError {
                                    kind: IoErrorKind::OtherIoError,
                                    desc: "Malformed IRC message",
                                    detail: None
                                });
                            },
                            MessageErr::EmptyInput => {
                                start = i;
                            }
                        }
                    }
                }
            }
        }

        if start > 0 {
            let new_slice_len = self.buffer_length - start;
            let mut tmp_buf = Vec::with_capacity(new_slice_len);
            tmp_buf.push_all(self.buffer.slice(start, self.buffer_length));
            self.buffer.slice_from_or_fail_mut(&new_slice_len).set_memory(0u8);
            copy_memory(self.buffer, tmp_buf.as_slice());
            self.buffer_length -= start;
        }

        Ok(msg)
    }

    #[experimental]
    pub fn read_message(&mut self) -> IoResult<Option<Message<'static>>> {
        match self.read_message_from_buffer() {
            Ok(m) => {
                match m {
                    Some(msg) => {
                        Ok(Some(msg))
                    },
                    None => {
                        if self.buffer_length < 1023 {
                            let mut in_buf = Vec::from_fn(1024 - self.buffer_length, |_| 0u8);
                            match self.reader.read(in_buf.as_mut_slice()) {
                                Ok(bytes_read) => {
                                    if bytes_read > 0 {
                                        copy_memory(self.buffer.slice_from_or_fail_mut(&self.buffer_length), in_buf.as_slice().slice_to(bytes_read));
                                        self.buffer_length += bytes_read;
                                        self.read_message_from_buffer()
                                    } else {
                                        Ok(None)
                                    }
                                },
                                Err(e) => Err(e)
                            }
                        } else {
                            return Err(IoError {
                                kind: IoErrorKind::OtherIoError,
                                desc: "Message exceeded maximum size of 1024 bytes",
                                detail: None
                            });
                        }
                    }
                }
            },
            err => err
        }
    }
}

#[test]
fn reader_test() {
    let mem_reader = MemReader::new(b"CAP LS\r\nNICK test\r\nUSER test * * :test\r\n".to_vec());

    let mut parser = Parser::new(mem_reader);

    match parser.read_message() {
        Err(e) => { panic!("Error: {}", e); },
        Ok(msg_opt) => {
            match msg_opt {
                None => { panic!("Expected 'CAP LS'"); },
                Some(msg) => {
                    assert!(msg.tags().is_none());
                    assert!(msg.prefix().is_none());
                    assert!(msg.command().as_slice() == "CAP");
                    assert!(msg.params().is_some());
                    assert!(msg.params().as_ref().unwrap().as_slice() == &[Slice("LS")]);
                }
            }
        }
    }

    match parser.read_message() {
        Err(e) => { panic!("Error: {}", e); },
        Ok(msg_opt) => {
            match msg_opt {
                None => { panic!("Expected 'NICK test'"); },
                Some(msg) => {
                    println!("{}", msg);
                    assert!(msg.tags().is_none());
                    assert!(msg.prefix().is_none());
                    assert!(msg.command().as_slice() == "NICK");
                    assert!(msg.params().is_some());
                    assert!(msg.params().as_ref().unwrap().as_slice() == &[Slice("test")]);
                }
            }
        }
    }

    match parser.read_message() {
        Err(e) => { panic!("Error: {}", e); },
        Ok(msg_opt) => {
            match msg_opt {
                None => { panic!("Expected 'USER test * * :test'"); },
                Some(msg) => {
                    assert!(msg.tags().is_none());
                    assert!(msg.prefix().is_none());
                    assert!(msg.command().as_slice() == "USER");
                    assert!(msg.params().is_some());
                    assert!(msg.params().as_ref().unwrap().as_slice() == &[Slice("test"), Slice("*"), Slice("*"), Slice("test")]);
                }
            }
        }
    }

    match parser.read_message() {
        Err(e) => { 
            match e.kind {
                IoErrorKind::EndOfFile => {},
                err => {
                    panic!("Error: {}", err);
                }
            }
        },
        Ok(msg_opt) => {
            assert!(msg_opt.is_none());
        }
    }
}
