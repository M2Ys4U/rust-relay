use std::ascii::AsciiExt;
use std::fmt;
use std::str::{MaybeOwned, Owned};
#[cfg(test)]
use std::str::Slice;

use prefix::Prefix;
use tag::Tag;

#[unstable]
#[deriving(Clone)]
pub struct Message<'_> {
    tags: Option<Vec<Tag<'_>>>,
    prefix: Option<Prefix<'_>>,
    command: MaybeOwned<'_>,
    params: Option<Vec<MaybeOwned<'_>>>
}

#[experimental]
#[deriving(Clone, Show)]
pub enum MessageErr {
    EmptyInput,
    MalformedInput
}

#[unstable]
impl<'_> Message<'_> {
    #[stable]
    pub fn from_str(mut msg: &str) -> Result<Message<'static>, MessageErr> {
        msg = msg.trim();

        if msg == "" {
            return Err(EmptyInput);
        }

        let mut tags: Option<Vec<Tag>> = None;

        if msg.as_bytes()[0] as char == '@' {
            let sp = msg.find(' ');
            tags = match sp {
                None => { return Err(MalformedInput) },
                Some(s) => { 
                    match Tag::from_str(msg.slice(1, s)) {
                        Ok(tags) => { tags },
                        Err(_) => {
                           return Err(MalformedInput)
                        }
                    }
                }
            };
            msg = msg.slice_from(sp.unwrap() + 1);
        }

        if msg == "" {
            return Err(MalformedInput);
        }

        let mut prefix: Option<Prefix> = None;

        if msg.as_bytes()[0] as char == ':' {
            let sp = msg.find(' ');
            prefix = match sp {
                None => { return Err(MalformedInput)},
                Some(sp) => { Some(Prefix::from_str(msg.slice(1, sp))) }
            };
            msg = msg.slice_from(sp.unwrap() + 1);
        }

        if msg == "" {
            return Err(MalformedInput);
        }

        let mut params: Vec<MaybeOwned> = match msg.find_str(" :") {
            None => { msg.split_terminator(' ').filter(|x| *x != "").map(|x| Owned(x.to_string())).collect() },
            Some(n) => {
                let mut params:Vec<MaybeOwned> = msg.slice_to(n).split_terminator(' ').filter(|x| *x != "").map(|x| Owned(x.to_string())).collect();
                params.push(Owned(msg.slice_from(n + 2).to_string()));
                params
            }
        };

        let command = match params.remove(0) {
            None => {
                return Err(MalformedInput);
            },
            Some(cmd) => {
                Owned(cmd.as_slice().to_ascii_upper())
            }
        };

        Ok(Message {
            tags: tags,
            prefix: prefix,
            command: command,
            params: if params.iter().len() > 0 {
                    Some(params)
                } else {
                    None
                }
        })
    }

    #[experimental]
    pub fn from_parts<'a>(tags: Option<Vec<Tag<'a>>>, prefix: Option<Prefix<'a>>, command: MaybeOwned<'a>, params: Option<Vec<MaybeOwned<'a>>>) -> Message<'a> {
        Message {
            tags: tags,
            prefix: prefix,
            command: command,
            params: params
        }
    }

    #[experimental]
    pub fn tags<'a>(&'a self) -> &'a Option<Vec<Tag<'_>>> {
        &self.tags
    }

    #[experimental]
    pub fn tags_mut<'a>(&'a mut self) -> &'a mut Option<Vec<Tag<'_>>> {
        &mut self.tags
    }

    #[experimental]
    pub fn prefix<'a>(&'a self) -> &'a Option<Prefix<'_>> {
        &self.prefix
    }

    #[experimental]
    pub fn prefix_mut<'a>(&'a mut self) -> &'a mut Option<Prefix<'_>> {
        &mut self.prefix
    }

    #[experimental]
    pub fn command<'a>(&'a self) -> &'a MaybeOwned<'_> {
        &self.command
    }

    #[experimental]
    pub fn command_mut<'a>(&'a mut self) -> &'a mut MaybeOwned<'_> {
        &mut self.command
    }

    #[experimental]
    pub fn params<'a>(&'a self) -> &'a Option<Vec<MaybeOwned<'_>>> {
        &self.params
    }

    #[experimental]
    pub fn params_mut<'a>(&'a mut self) -> &'a mut Option<Vec<MaybeOwned<'_>>> {
        &mut self.params
    }
}

#[stable]
impl<'a> fmt::Show for Message<'a> {
    fn fmt<'a>(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.tags.is_some() {
            let t: Vec<String> = self.tags.as_ref().unwrap().iter().map(|tag| tag.to_string()).collect();
            try!(write!(f, "@{} ", t.connect(";")));
        }

        if self.prefix.is_some() {
            try!(write!(f, ":{} ", self.prefix.as_ref().unwrap()));
        }

        try!(write!(f, "{}", self.command));

        if self.params.is_some() {
            let params = self.params.as_ref().unwrap();
            let p: Vec<String> = params.iter().take_while(|p| !p.as_slice().contains_char(' ')).map(|p| p.to_string()).collect();
            let out = p.connect(" ");

            try!(if params.iter().len() > p.iter().len() {
                let trailing: Vec<String> = params.iter().skip(p.len()).map(|p| p.to_string()).collect();
                if out.len() > 0 {
                    write!(f, " {} :{}", out, trailing.connect(" "))
                } else {
                    write!(f, " :{}", trailing.connect(" "))
                }
            } else {
                if params.iter().rev().next().unwrap().as_slice() == "" {
                    write!(f, " {}:", out)
                } else {
                    write!(f, " {}", out)
                }
            });
        }

        write!(f, "\r\n")
    }
}

#[test]
fn command_only() {
    let msg = Message::from_str("PING");
    match msg {
        Err(e) => { panic!("Error: {}", e); },
        Ok(msg) => {
            assert!(msg.tags().is_none());
            assert!(msg.prefix().is_none());
            assert!(msg.command().as_slice() == "PING");
            assert!(msg.params().is_none());
            assert!(msg.to_string().as_slice() == "PING\r\n");
        }
    }
}

#[test]
fn command_one_param() {
    let msg = Message::from_str("CAP LS");
    match msg {
        Err(e) => { panic!("Error: {}", e); }
        Ok(msg) => {
            assert!(msg.tags().is_none());
            assert!(msg.prefix().is_none());
            assert!(msg.command().as_slice() == "CAP");
            assert!(msg.params().is_some());
            assert!(msg.params().as_ref().unwrap().as_slice() == &[Slice("LS")]);
            assert!(msg.to_string().as_slice() == "CAP LS\r\n");
        }
    }
}

#[test]
fn command_two_params() {
    let msg = Message::from_str("JOIN #test test");
    match msg {
        Err(e) => { panic!("Error: {}", e); }
        Ok(msg) => {
            assert!(msg.tags().is_none());
            assert!(msg.prefix().is_none());
            assert!(msg.command().as_slice() == "JOIN");
            assert!(msg.params().is_some());
            assert!(msg.params().as_ref().unwrap().as_slice() == &[Slice("#test"), Slice("test")]);
            assert!(msg.to_string().as_slice() == "JOIN #test test\r\n");
        }
    }
}

#[test]
fn command_two_params_trailing() {
    let msg = Message::from_str("PRIVMSG #test :Hello world");
    match msg {
        Err(e) => { panic!("Error: {}", e); }
        Ok(msg) => {
            assert!(msg.tags().is_none());
            assert!(msg.prefix().is_none());
            assert!(msg.command().as_slice() == "PRIVMSG");
            assert!(msg.params().is_some());
            assert!(msg.params().as_ref().unwrap().as_slice() == &[Slice("#test"), Slice("Hello world")]);
            assert!(msg.to_string().as_slice() == "PRIVMSG #test :Hello world\r\n");
        }
    }
}

#[test]
fn empty_trailing() {
    let msg = Message::from_str("PRIVMSG #test :");
    match msg {
        Err(e) => { panic!("Error: {}", e); }
        Ok(msg) => {
            assert!(msg.tags().is_none());
            assert!(msg.prefix().is_none());
            assert!(msg.command().as_slice() == "PRIVMSG");
            assert!(msg.params().is_some());
            assert!(msg.params().as_ref().unwrap().as_slice() == &[Slice("#test"), Slice("")]);
            assert!(msg.to_string().as_slice() == "PRIVMSG #test :\r\n");
        }
    }
}


#[test]
fn empty_message() {
    let msg = Message::from_str("");
    match msg {
        Err(e) => {
            match e {
                EmptyInput => {},
                _ => { panic!("Expected EmptyInput error but found {} instead", e); }
            }
        },
        _ => { panic!("msg should be Err(EmptyInput)") }
    }
}

#[test]
fn only_tags() {
    let msg = Message::from_str("@hello=world;foo=bar;baz");
    match msg {
        Err(e) => {
            match e {
                MalformedInput => {},
                _ => { panic!("Expected MalformedInput but found {} instead", e); }
            }
        },
        _ => { panic!("msg should be Err(MalformedInput)") }
    }
}

#[test]
fn one_tag_value() {
    let msg = Message::from_str("@hello=world PING");
    match msg {
        Err(e) => { panic!("Error: {}", e); }
        Ok(msg) => {
            assert!(msg.tags().is_some());
            assert!(msg.tags().as_ref().unwrap().as_slice() == &[Tag::from_parts(Slice("hello"), Some(Slice("world")))])
            assert!(msg.prefix().is_none());
            assert!(msg.command().as_slice() == "PING");
            assert!(msg.params().is_none());
            assert!(msg.to_string().as_slice() == "@hello=world PING\r\n");
        }
    }
}

#[test]
fn one_tag_no_value() {
    let msg = Message::from_str("@hello PING");
    match msg {
        Err(e) => { panic!("Error: {}", e); }
        Ok(msg) => {
            assert!(msg.tags().is_some());
            assert!(msg.tags().as_ref().unwrap().as_slice() == &[Tag::from_parts(Slice("hello"), None)])
            assert!(msg.prefix().is_none());
            assert!(msg.command().as_slice() == "PING");
            assert!(msg.params().is_none());
            assert!(msg.to_string().as_slice() == "@hello PING\r\n");
        }
    }
}

#[test]
fn two_tags_values() {
    let msg = Message::from_str("@hello=world;foo=bar PING");
    match msg {
        Err(e) => { panic!("Error: {}", e); }
        Ok(msg) => {
            assert!(msg.tags().is_some());
            assert!(msg.tags().as_ref().unwrap().as_slice() == &[Tag::from_parts(Slice("hello"), Some(Slice("world"))), Tag::from_parts(Slice("foo"), Some(Slice("bar")))])
            assert!(msg.prefix().is_none());
            assert!(msg.command().as_slice() == "PING");
            assert!(msg.params().is_none());
            assert!(msg.to_string().as_slice() == "@hello=world;foo=bar PING\r\n");
        }
    }
}

#[test]
fn two_tags_no_values() {
    let msg = Message::from_str("@hello;foo PING");
    match msg {
        Err(e) => { panic!("Error: {}", e); }
        Ok(msg) => {
            assert!(msg.tags().is_some());
            assert!(msg.tags().as_ref().unwrap().as_slice() == &[Tag::from_parts(Slice("hello"), None), Tag::from_parts(Slice("foo"), None)])
            assert!(msg.prefix().is_none());
            assert!(msg.command().as_slice() == "PING");
            assert!(msg.params().is_none());
            assert!(msg.to_string().as_slice() == "@hello;foo PING\r\n");
        }
    }
}

#[test]
fn two_tags_mixed_values() {
    let msg = Message::from_str("@hello=world;foo PING");
    match msg {
        Err(e) => { panic!("Error: {}", e); }
        Ok(msg) => {
            assert!(msg.tags().is_some());
            assert!(msg.tags().as_ref().unwrap().as_slice() == &[Tag::from_parts(Slice("hello"), Some(Slice("world"))), Tag::from_parts(Slice("foo"), None)])
            assert!(msg.prefix().is_none());
            assert!(msg.command().as_slice() == "PING");
            assert!(msg.params().is_none());
            assert!(msg.to_string().as_slice() == "@hello=world;foo PING\r\n");
        }
    }
}

#[test]
fn command_with_prefix() {
    let msg = Message::from_str(":irc.example.net PING");
    match msg {
        Err(e) => { panic!("Error: {}", e); },
        Ok(msg) => {
            assert!(msg.tags().is_none());
            assert!(msg.prefix().is_some());
            assert!(msg.prefix().as_ref().unwrap().to_string().as_slice() == "irc.example.net");
            assert!(msg.command().as_slice() == "PING");
            assert!(msg.params().is_none());
            assert!(msg.to_string().as_slice() == ":irc.example.net PING\r\n");
        }
    }
}

#[test]
fn all_parts() {
    let msg = Message::from_str("@intent=action :irc.example.net PRIVMSG #world :Waves hello");
    match msg {
        Err(e) => { panic!("Error: {}", e); },
        Ok(msg) => {
            assert!(msg.tags().is_some());
            assert!(msg.tags().as_ref().unwrap().as_slice() == &[Tag::from_parts(Slice("intent"), Some(Slice("action")))])
            assert!(msg.prefix().is_some());
            assert!(msg.prefix().as_ref().unwrap().to_string().as_slice() == "irc.example.net");
            assert!(msg.command().as_slice() == "PRIVMSG");
            assert!(msg.params().is_some());
            assert!(msg.params().as_ref().unwrap().as_slice() == &[Slice("#world"), Slice("Waves hello")]);
            assert!(msg.to_string().as_slice() == "@intent=action :irc.example.net PRIVMSG #world :Waves hello\r\n");
        }
    }
}
