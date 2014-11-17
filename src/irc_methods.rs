use std::io::{IoResult, Writer};
use std::str::replace;

use capability::Capability;

#[experimental]
pub trait IrcMethods: Writer {
    #[experimental]
    fn pass(&mut self, pass: &str) -> IoResult<()> {
        write!(self, "PASS {}\r\n", pass)
    }

    #[experimental]
    fn cap_ls(&mut self, version: Option<&str>) -> IoResult<()> {
        match version {
            Some(version) => {
                write!(self, "CAP LS {}\r\n", version)
            },
            None => {
                self.write(b"CAP LS\r\n")
            }
        }
    }

    #[experimental]
    fn cap_list(&mut self) -> IoResult<()> {
        self.write(b"CAP LIST\r\n")
    }

    #[experimental]
    fn cap_req(&mut self, caps: &Vec<Capability>) -> IoResult<()> {
        write!(self, "CAP REQ :{}\r\n", caps.iter().map(|c| c.to_string()).fold(String::new(), |mut l, c| {
            l.push_str(" ");
            l.push_str(c.to_string().as_slice());
            l
        }))
    }

    #[experimental]
    fn cap_ack(&mut self, caps: &Vec<Capability>) -> IoResult<()> {
        write!(self, "CAP ACK :{}\r\n", caps.iter().map(|c| c.to_string()).fold(String::new(), |mut l, c| {
            l.push_str(" ");
            l.push_str(c.to_string().as_slice());
            l
        }))
    }

    #[experimental]
    fn cap_clear(&mut self) -> IoResult<()> {
        self.write(b"CAP CLEAR\r\n")
    }

    #[experimental]
    fn cap_end(&mut self) -> IoResult<()> {
        self.write(b"CAP END\r\n")
    }


    #[experimental]
    fn nick(&mut self, new_nick: &str) -> IoResult<()> {
        write!(self, "NICK {}\r\n", new_nick)
    }

    #[experimental]
    fn user(&mut self, user_name: &str, real_name: &str) -> IoResult<()> {
        write!(self, "USER {} * * :{}\r\n", user_name, real_name)
    }

    #[experimental]
    fn oper(&mut self, user: &str, pass: &str) -> IoResult<()> {
        write!(self, "OPER {} {}\r\n", user, pass)
    }

    #[experimental]
    fn __quit(&mut self, message: Option<&str>) -> IoResult<()> {
        match message {
            Some(message) => {
                write!(self, "QUIT :{}\r\n", message)
            },
            None => {
                write!(self, "QUIT\r\n")
            }
        }.and_then(|_| self.flush())
    }

    fn quit(&mut self, message: Option<&str>) -> IoResult<()> {
        self.__quit(message)
    }

    #[experimental]
    fn join(&mut self, channel: &str, key: Option<&str>) -> IoResult<()> {
        match key {
            Some(key) => {
                write!(self, "JOIN {} {}\r\n", channel, key)
            },
            None => {
                write!(self, "JOIN {}\r\n", channel)
            }
        }
    }

    #[experimental]
    fn part(&mut self, channel: &str, message: Option<&str>) -> IoResult<()> {
        match message {
            Some(message) => {
                write!(self, "PART {} :{}\r\n", channel, message)
            },
            None => {
                write!(self, "PART {}\r\n", channel)
            }
        }
    }

    #[experimental]
    fn mode(&mut self, target: &str, params: Option<Vec<&str>>) -> IoResult<()> {
        match params {
            Some(params) => {
                write!(self, "MODE {} {}\r\n", target, params.connect(" "))
            },
            None => {
                write!(self, "MODE {}\r\n", target)
            }
        }
    }

    #[experimental]
    fn topic(&mut self, channel: &str, new_topic: Option<&str>) -> IoResult<()> {
        match new_topic {
            Some(new_topic) => {
                write!(self, "TOPIC {} :{}\r\n", channel, new_topic)
            },
            None => {
                write!(self, "TOPIC {}\r\n", channel)
            }
        }
    }

    #[experimental]
    fn names(&mut self, channel: &str) -> IoResult<()> {
        write!(self, "NAMES {}\r\n", channel)
    }

    #[experimental]
    fn list(&mut self) -> IoResult<()> {
        self.write(b"LIST\r\n")
    }

    #[experimental]
    fn invite(&mut self, nick: &str, channel: &str) -> IoResult<()> {
        write!(self, "INVITE {} {}\r\n", nick, channel)
    }

    #[experimental]
    fn kick(&mut self, channel: &str, nick: &str, reason: Option<&str>) -> IoResult<()> {
        match reason {
            Some(reason) => {
                write!(self, "KICK {} {} :{}\r\n", channel, nick, reason)
            },
            None => {
                write!(self, "KICK {} {}\r\n", channel, nick)
            }
        }
    }

    #[experimental]
    fn version(&mut self, server: Option<&str>) -> IoResult<()> {
        match server {
            Some(server) => {
                write!(self, "VERSION :{}\r\n", server)
            },
            None => {
                self.write(b"VERSION\r\n")
            }
        }
    }

    #[experimental]
    fn stats(&mut self, query: &char, server: Option<&str>) -> IoResult<()> {
        match server {
            Some(server) => {
                write!(self, "STATS {} :{}\r\n", query, server)
            },
            None => {
                write!(self, "STATS {}\r\n", query)
            }
        }
    }

    #[experimental]
    fn links(&mut self, mask: Option<&str>, server: Option<&str>) -> IoResult<()> {
        match mask {
            Some(mask) => {
                match server {
                    Some(server) => {
                        write!(self, "LINKS {} {}\r\n", server, mask)
                    },
                    None => {
                        write!(self, "LINKS {}\r\n", mask)
                    }
                }
            },
            None => {
                self.write(b"LINKS\r\n")
            }
        }
    }

    #[experimental]
    fn time(&mut self, server: Option<&str>) -> IoResult<()> {
        match server {
            Some(server) => {
                write!(self, "TIME {}\r\n", server)
            },
            None => {
                self.write(b"TIME\r\n")
            }
        }
    }

    #[experimental]
    fn connect(&mut self, target_server: &str, port: Option<&str>, remote_server: Option<&str>) -> IoResult<()> {
        match port {
            Some(port) => {
                match remote_server {
                    Some(remote_server) => {
                        write!(self, "CONNECT {} {} {}\r\n", target_server, port, remote_server)
                    },
                    None => {
                        write!(self, "CONNECT {} {}\r\n", target_server, port)
                    }
                }
            },
            None => {
                write!(self, "CONNECT {}\r\n", target_server)
            }
        }
    }

    #[experimental]
    fn trace(&mut self, server: Option<&str>) -> IoResult<()> {
        match server {
            Some(server) => {
                write!(self, "TRACE {}\r\n", server)
            },
            None => {
                self.write(b"TRACE\r\n")
            }
        }
    }

    #[experimental]
    fn admin(&mut self, server: Option<&str>) -> IoResult<()> {
        match server {
            Some(server) => {
                write!(self, "ADMIN {}\r\n", server)
            },
            None => {
                self.write(b"ADMIN\r\n")
            }
        }
    }

    #[experimental]
    fn info(&mut self, server: Option<&str>) -> IoResult<()> {
        match server {
            Some(server) => {
                write!(self, "INFO {}\r\n", server)
            },
            None => {
                self.write(b"INFO\r\n")
            }
        }
    }

    #[experimental]
    fn privmsg(&mut self, target: &str, message: &str) -> IoResult<()> {
        write!(self, "PRIVMSG {} :{}\r\n", target, message)
    }

    #[experimental]
    fn notice(&mut self, target: &str, message: &str) -> IoResult<()> {
        write!(self, "NOTICE {} :{}\r\n", target, message)
    }

    #[experimental]
    fn ctcp_request(&mut self, ctcp_type: &str, target: &str, content: &str) -> IoResult<()> {
        let content = replace(content, "\x16", "\x16\x16").replace("\0", "\x160").replace("\n", "\x16n").replace("\r", "\x16r").replace("\x01", "\\a");
        write!(self, "PRIVMSG {} :\x01{} {}\x01", target, ctcp_type, content)
    }

    #[experimental]
    fn ctcp_reply(&mut self, ctcp_type: &str, target: &str, content: &str) -> IoResult<()> {
        let content = replace(content, "\x16", "\x16\x16").replace("\0", "\x160").replace("\n", "\x16n").replace("\r", "\x16r").replace("\x01", "\\a");
        write!(self, "NOTICE {} :\x01{} {}\x01", target, ctcp_type, content)
    }

    #[experimental]
    fn who(&mut self, name: Option<&str>, oper: Option<bool>) -> IoResult<()> {
        match name {
            Some(name) => {
                if oper.is_some() && oper.unwrap() {
                    write!(self, "WHO {} o\r\n", name)
                } else {
                    write!(self, "WHO {}\r\n", name)
                }
            },
            None => {
                self.write(b"WHO\r\n")
            }
        }
    }

    #[experimental]
    fn whois(&mut self, nickmask: &str, server: Option<&str>) -> IoResult<()> {
        match server {
            Some(server) => {
                write!(self, "WHOIS {}, {}\r\n", server, nickmask)
            },
            None => {
                write!(self, "WHOIS {}\r\n", nickmask)
            }
        }
    }

    #[experimental]
    fn whowas(&mut self, nickname: &str, count: Option<int>, server: Option<&str>) -> IoResult<()> {
        match count {
            Some(count) => {
                match server {
                    Some(server) => {
                        write!(self, "WHOWAS {} {} {}\r\n", nickname, count, server)
                    },
                    None => {
                        write!(self, "WHOWAS {} {}\r\n", nickname, count)
                    }
                }
            },
            None => {
                write!(self, "WHOWAS {}\r\n", nickname)
            }
        }
    }

    #[experimental]
    fn kill(&mut self, nickname: &str, comment: &str) -> IoResult<()> {
        write!(self, "KILL {} :{}\r\n", nickname, comment)
    }

    #[experimental]
    fn ping(&mut self, server1: &str, server2: Option<&str>) -> IoResult<()> {
        match server2 {
            Some(server2) => {
                write!(self, "PING {} {}\r\n", server1, server2)
            },
            None => {
                write!(self, "PING {}\r\n", server1)
            }
        }
    }

    #[experimental]
    fn pong(&mut self, daemon1: &str, daemon2: Option<&str>) -> IoResult<()> {
        match daemon2 {
            Some(daemon2) => {
                write!(self, "PONG {} {}\r\n", daemon1, daemon2)
            },
            None => {
                write!(self, "PONG {}\r\n", daemon1)
            }
        }
    }

    #[experimental]
    fn away(&mut self, message: Option<&str>) -> IoResult<()> {
        match message {
            Some(message) => {
                write!(self, "AWAY :{}\r\n", message)
            },
            None => {
                self.write(b"AWAY\r\n")
            }
        }
    }

    #[experimental]
    fn wallops(&mut self, message: &str) -> IoResult<()> {
        write!(self, "WALLOPS :{}\r\n", message)
    }

    #[experimental]
    fn motd(&mut self, target: Option<&str>) -> IoResult<()> {
        match target {
            Some(target) => {
                write!(self, "MOTD {}\r\n", target)
            },
            None => {
                self.write(b"MOTD\r\n")
            }
        }
    }

    #[experimental]
    fn lusers(&mut self, mask: Option<&str>, server: Option<&str>) -> IoResult<()> {
        match mask {
            Some(mask) => {
                match server {
                    Some(server) => {
                        write!(self, "LUSERS {} {}\r\n", mask, server)
                    },
                    None => {
                        write!(self, "LUSERS {}\r\n", mask)
                    }
                }
            },
            None => {
                self.write(b"LUSERS\r\n")
            }
        }
    }

    #[experimental]
    fn metadata_list(&mut self, target: &str, keys: Option<Vec<&str>>) -> IoResult<()> {
        match keys {
            Some(keys) => {
                write!(self, "METADATA {} LIST :{}\r\n", target, keys.connect(" "))
            },
            None => {
                write!(self, "METADATA {} LIST\r\n", target)
            }
        }
    }

    #[experimental]
    fn metadata_set(&mut self, target: &str, key: &str, value: Option<&str>) -> IoResult<()> {
        match value {
            Some(value) => {
                write!(self, "METADATA {} SET {} :{}\r\n", target, key, value)
            },
            None => {
                write!(self, "METADATA {} SET {}\r\n", target, key)
            }
        }
    }

    #[experimental]
    fn metadata_clear(&mut self, target: &str) -> IoResult<()> {
        write!(self, "METADATA {} CLEAR\r\n", target)
    }

    #[experimental]
    fn monitor_add(&mut self, targets: Vec<&str>) -> IoResult<()> {
        write!(self, "MONITOR + {}\r\n", targets.connect(","))
    }

    #[experimental]
    fn monitor_remove(&mut self, targets: Vec<&str>) -> IoResult<()> {
        write!(self, "MONITOR - {}\r\n", targets.connect(","))
    }

    #[experimental]
    fn monitor_clear(&mut self) -> IoResult<()> {
        self.write(b"MONITOR C\r\n")
    }

    #[experimental]
    fn monitor_list(&mut self) -> IoResult<()> {
        self.write(b"MONITOR L\r\n")
    }

    #[experimental]
    fn monitor_status(&mut self) -> IoResult<()> {
        self.write(b"MONITOR S\r\n")
    }
}
