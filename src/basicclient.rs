use std::collections::HashSet;
use std::io::{IoError, IoErrorKind, IoResult, Writer};
use std::io::net::ip::{SocketAddr, ToSocketAddr};
use std::io::net::tcp::TcpStream;
use std::time::duration::Duration;

use capability::{Capability, CapabilityModifier};
use connection::Connection;
use irc_methods::IrcMethods;
use message::Message;

#[experimental]
pub enum ConnectionStatus {
    NotConnected,
    Connecting,
    Connected,
    Disconnected,
    Error(IoError)
}

#[experimental]
pub struct BasicClient {
    connection: Option<Connection<TcpStream>>,
    remote_addr: Option<SocketAddr>,
    status: ConnectionStatus,
    nick: String,
    user_name: String,
    real_name: String,
    wanted_caps: HashSet<Capability>,
    available_caps: HashSet<Capability>,
    requested_caps: HashSet<Capability>,   
    enabled_caps: HashSet<Capability>,
    listed_caps: HashSet<Capability>,
    cap_partial_listing: bool
}

#[experimental]
impl BasicClient {
    #[experimental]
    pub fn new(nick: &str, user_name: &str, real_name: &str, wanted_caps: HashSet<Capability>) -> BasicClient {
        BasicClient {
            connection: None,
            remote_addr: None,
            status: NotConnected,
            nick: String::from_str(nick),
            user_name: String::from_str(user_name),
            real_name: String::from_str(real_name),
            wanted_caps: wanted_caps,
            available_caps: HashSet::new(),
            requested_caps: HashSet::new(),
            enabled_caps: HashSet::new(),
            listed_caps: HashSet::new(),
            cap_partial_listing: false
        }
    }

    #[experimental]
    #[inline]
    pub fn get_stream<'a>(&'a mut self) -> &'a mut TcpStream {
        self.connection.as_mut().unwrap().get_stream()
    }
    
    fn register(&mut self) -> IoResult<()> {
        match if self.wanted_caps.is_empty() {
            self.cap_end()
        } else {
            self.cap_ls(Some("302"))
        }.and({
            let nick = self.nick.clone();
            self.nick(nick.as_slice())
        })
        .and({
            let (user, real) = (self.user_name.clone(), self.real_name.clone());
            self.user(user.as_slice(), real.as_slice())
        }) {
            Ok(_) => {
                self.status = Connected;
                Ok(())
            },
            Err(e) => {
                self.status = Error(e.clone());
                Err(e)
            }
        }
    }

    #[experimental]
    pub fn read_message(&mut self) -> IoResult<Option<Message<'static>>> {
        match self.status {
            Connected | Connecting => {
                loop {
                    match self.connection.as_mut().unwrap().read_message() {
                        Ok(msg_opt) => {
                            match msg_opt {
                                Some(msg) => {
                                    match msg.command().as_slice() {
                                        "PING" => {
                                            match msg.params() {
                                                &Some(ref params) => {
                                                    let mut p_iter = params.iter().map(|p| p.as_slice());
                                                    try!(self.pong(p_iter.next().unwrap(), p_iter.next()));
                                                },
                                                &None => {
                                                    try!(self.write(b"PONG\r\n"));
                                                }
                                            }
                                        },
                                        "CAP" => {
                                            match self.negotiate_capabilities(msg) {
                                                Ok(msg_opt) => {
                                                    if msg_opt.is_some() {
                                                        return Ok(msg_opt);
                                                    }
                                                },
                                                err => return err
                                            }
                                        },
                                        "ERROR" => {
                                            let err = IoError {
                                                kind: IoErrorKind::OtherIoError,
                                                desc: "IRC Error",
                                                detail: msg.params().as_ref().map(|p| p[0].to_string())
                                            };
                                            self.status = Error(err.clone());
                                            let con = self.connection.as_mut().unwrap().get_stream();
                                            let _ = con.close_write().and(con.close_read());
                                            return Err(err);
                                        }
                                        _ => {
                                            return Ok(Some(msg));
                                        }
                                    }
                                },
                                None => return Ok(None)
                            }
                        },
                        err => return err
                    }
                }
            },
            _ => {
                Err(IoError {
                    kind: IoErrorKind::NotConnected,
                    desc: "Not connected",
                    detail: None
                })
            }
        }
    }

    fn negotiate_capabilities(&mut self, msg: Message<'static>) -> IoResult<Option<Message<'static>>> {
        match *msg.params() {
            Some(ref params) => {
                let mut p_iter = params.iter().map(|p| p.as_slice()).skip(1);
                match p_iter.next() {
                    Some(sub_cmd) if sub_cmd == "LS" => {
                        match p_iter.next() {
                            Some(p) if p == "*" => {
                                self.cap_partial_listing = true;
                                match p_iter.next() {
                                    Some(caps) => {
                                        for cap in caps.split_terminator(' ').map(|c| Capability::from_str(c)) {
                                            if cap.is_some() {
                                                self.available_caps.insert(cap.unwrap());
                                            }
                                        }
                                        Ok(None)
                                    },
                                    None => {
                                        try!(self.cap_end())
                                        Ok(None)
                                    }
                                }
                            },
                            Some(p) => {
                                if self.cap_partial_listing {
                                    self.cap_partial_listing = false;
                                }

                                for cap in p.split_terminator(' ').map(|c| Capability::from_str(c)) {
                                    if cap.is_some() {
                                        self.available_caps.insert(cap.unwrap());
                                    }
                                }

                                // FIXME(JA): We should break this in to smaller chunks to make sure we don't exceed the IRC line length.
                                let requested_caps: Vec<Capability> = self.wanted_caps.intersection(&self.available_caps).map(|c| c.clone()).collect();
                                try!(self.cap_req(&requested_caps));
                                self.requested_caps = requested_caps.into_iter().collect();
                                Ok(None)
                            },
                            None => {
                                try!(self.cap_end())
                                Ok(None)
                            }
                        }
                    },
                    Some(sub_cmd) if sub_cmd == "ACK" => {
                        match p_iter.next() {
                            Some(p) => {
                                let mut to_ack = Vec::new();
                                for cap in p.split_terminator(' ').map(|c| Capability::from_str(c)) {
                                    if cap.is_some() {
                                        let cap = cap.unwrap();
                                        match cap.modifier {
                                            Some(modifier) => {
                                                match modifier {
                                                    CapabilityModifier::Disable => {
                                                        self.enabled_caps.remove(&cap);
                                                    },
                                                    CapabilityModifier::Ack => {
                                                        to_ack.push(cap.clone());
                                                        self.enabled_caps.insert(cap);
                                                    },
                                                    CapabilityModifier::Sticky => {
                                                        self.enabled_caps.insert(cap);
                                                    }
                                                }
                                            },
                                            None => {
                                                self.enabled_caps.insert(cap);
                                            }
                                        }
                                    }
                                }

                                if !to_ack.is_empty() {
                                    try!(self.cap_ack(&to_ack));
                                }

                                if self.enabled_caps == self.requested_caps {
                                    try!(self.cap_end());
                                }
                                Ok(None)
                            },
                            None => {
                                try!(self.cap_end())
                                Ok(None)
                            }
                        }
                    },
                    Some(sub_cmd) if sub_cmd == "LIST" => {
                        match p_iter.next() {
                            Some(p) if p == "*" => {
                                if !self.cap_partial_listing {
                                    if !self.listed_caps.is_empty() {
                                        self.listed_caps.clear();
                                    }
                                    self.cap_partial_listing = true;
                                }
                                for cap in p.split_terminator(' ').map(|c| Capability::from_str(c)) {
                                    if cap.is_some() {
                                        self.listed_caps.insert(cap.unwrap());
                                    }
                                }
                            },
                            Some(p) => {
                                self.cap_partial_listing = false;
                                for cap in p.split_terminator(' ').map(|c| Capability::from_str(c)) {
                                    if cap.is_some() {
                                        self.listed_caps.insert(cap.unwrap());
                                    }
                                }

                                self.enabled_caps = self.listed_caps.clone();
                                self.listed_caps.clear();
                            },
                            None => {}
                        }
                        Ok(None)
                    },
                    Some(sub_cmd) if sub_cmd == "NAK" => {
                        try!(self.cap_list());
                        try!(self.cap_end());
                        Ok(None)
                    },
                    Some(sub_cmd) if sub_cmd == "NEW" => {
                        match p_iter.next() {
                            Some(p) => {
                                let mut to_req = Vec::new();
                                for cap in p.split_terminator(' ').map(|c| Capability::from_str(c)) {
                                    if cap.is_some() {
                                        let cap = cap.unwrap();
                                        if self.wanted_caps.contains(&cap) {
                                            to_req.push(cap);
                                        }
                                    }
                                }

                                if !to_req.is_empty() {
                                    try!(self.cap_req(&to_req));
                                }
                            },
                            None => {}
                        }
                        Ok(None)
                    },
                    Some(sub_cmd) if sub_cmd == "DEL" => {
                        match p_iter.next() {
                            Some(p) => {
                                for cap in p.split_terminator(' ').map(|c| Capability::from_str(c)) {
                                    if cap.is_some() {
                                        let cap = cap.unwrap();
                                        if self.enabled_caps.contains(&cap) {
                                            self.enabled_caps.remove(&cap);
                                        }
                                    }
                                }
                            },
                            None => {}
                        }
                        Ok(None)
                    }
                    _ => {
                        Ok(Some(msg.clone()))
                    }
                }
            },
            None => {
                try!(self.cap_end())
                Ok(None)
            }
        }
    }
}

#[experimental]
impl<A: ToSocketAddr> BasicClient {
    pub fn connect_to(&mut self, addr: A) -> IoResult<()> {
        match self.status {
            Connected | Connecting => {
                Err(IoError {
                    kind: IoErrorKind::OtherIoError,
                    desc: "Already connected",
                    detail: None
                })
            },
            _ => {
                match Connection::connect_to(addr) {
                    Ok(mut con) => {
                        match con.get_stream().peer_name() {
                            Ok(addr) => {
                                self.remote_addr = Some(addr);
                                self.connection = Some(con);
                                self.status = Connecting;
                                
                                self.register()
                            }
                            Err(e) => {
                                self.status = Error(e.clone());
                                Err(e)
                            }
                        }
                    },
                    Err(e) => {
                        self.status = Error(e.clone());
                        Err(e)
                    }
                }
            }
        }
    }

    pub fn connect_to_timeout(&mut self, addr: A, timeout: Duration) -> IoResult<()> {
        match self.status {
            Connected | Connecting => {
                Err(IoError {
                    kind: IoErrorKind::OtherIoError,
                    desc: "Already connected",
                    detail: None
                })
            },
            _ => {
                match Connection::connect_to_timeout(addr, timeout) {
                    Ok(mut con) => {
                        match con.get_stream().peer_name() {
                            Ok(addr) => {
                                self.remote_addr = Some(addr);
                                self.connection = Some(con);
                                self.status = Connected;
                                
                                self.register()
                            }
                            Err(e) => {
                                self.status = Error(e.clone());
                                Err(e)
                            }
                        }
                    },
                    Err(e) => {
                        self.status = Error(e.clone());
                        Err(e)
                    }
                }
            }
        }
    }
}

#[experimental]
impl IrcMethods for BasicClient {
    fn quit(&mut self, message: Option<&str>) -> IoResult<()> {
        match self.status {
            Connected | Connecting => {
                match self.connection.as_mut().unwrap().quit(message) {
                    Ok(_) => {
                        self.status = Disconnected;
                        Ok(())
                    },
                    err => err
                }
            },
            _ => {
                Err(IoError {
                    kind: IoErrorKind::NotConnected,
                    desc: "Not connected",
                    detail: None
                })
            }
        }
    }
}

#[stable]
impl Writer for BasicClient {
    fn write(&mut self, buf: &[u8]) -> IoResult<()> {
        match self.status {
            Connected | Connecting => {
                match self.connection.as_mut().unwrap().write(buf) {
                    Err(e) => {
                        self.status = Error(e.clone());
                        Err(e)
                    },
                    ok => ok
                }
            },
            _ => {
                Err(IoError {
                    kind: IoErrorKind::NotConnected,
                    desc: "Not connected",
                    detail: None
                })
            }
        }
    }

    fn flush(&mut self) -> IoResult<()> {
        match self.status {
            Connected | Connecting => {
                match self.connection.as_mut().unwrap().flush() {
                    Err(e) => {
                        self.status = Error(e.clone());
                        Err(e)
                    },
                    ok => ok
                }
            },
            _ => {
                Err(IoError {
                    kind: IoErrorKind::NotConnected,
                    desc: "Not connected",
                    detail: None
                })
            }
        }
    }
}

impl Drop for BasicClient {
    fn drop(&mut self) {
        let _ = self.quit(Some("rust-relay"));
    }
}
