use std::fmt;
use std::str::{MaybeOwned, Owned};

#[unstable]
#[deriving(Clone, PartialEq, Eq)]
pub struct ClientPrefix<'a> {
    pub nick: MaybeOwned<'a>,
    pub ident: MaybeOwned<'a>,
    pub host: MaybeOwned<'a>
}

#[stable]
impl<'a> fmt::Show for ClientPrefix<'a> {
    #[stable]
    fn fmt<'a>(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}!{}@{}", self.nick.as_slice(), self.ident.as_slice(), self.host.as_slice())
    }
}

#[unstable]
#[deriving(Clone, PartialEq, Eq)]
pub enum Prefix<'a> {
    Client(ClientPrefix<'a>),
    Server(MaybeOwned<'a>)
}

#[unstable]
impl<'a> Prefix<'a> {
    #[unstable]
    pub fn from_str(s: &str) -> Prefix<'static> {
        let re = regex!(r"^(.+)!(.+)@(.+)$");
        match re.captures(s) {
            Some(captures) => {
                Client(ClientPrefix{
                    nick: Owned(String::from_str(captures.at(1))),
                    ident: Owned(String::from_str(captures.at(2))),
                    host: Owned(String::from_str(captures.at(3)))
                })
            },
            None => {
                Server(Owned(String::from_str(s)))
            }
        }
    }

    #[stable]
    #[inline]
    pub fn is_client(&self) -> bool {
        match *self {
            Client(_) => true,
            Server(_) => false
        }
    }

    #[stable]
    #[inline]
    pub fn is_server(&self) -> bool {
        !self.is_client()
    }

    #[unstable]
    pub fn nick<'a>(&'a self) -> Option<&'a str> {
        match *self {
            Client(ref client_prefix) => {
                Some(client_prefix.nick.as_slice())
            },
            Server(_) => None
        }
    }

    #[unstable]
    pub fn ident<'a>(&'a self) -> Option<&'a str> {
        match *self {
            Client(ref client_prefix) => {
                Some(client_prefix.ident.as_slice())
            },
            Server(_) => None
        }
    }

    #[unstable]
    pub fn host<'a>(&'a self) -> &'a str {
        match *self {
            Client(ref client_prefix) => {
                client_prefix.host.as_slice()
            },
            Server(ref server_prefix) => server_prefix.as_slice()
        }
    }
}

#[stable]
impl<'a> fmt::Show for Prefix<'a> {
    fn fmt<'a>(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Client(ref client_prefix) => {
                write!(f, "{}", client_prefix)
            },
            Server(ref server_prefix) => {
                write!(f, "{}", server_prefix)
            }
        }
    }
}
