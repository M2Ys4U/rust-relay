#![feature(default_type_params)]
#![feature(phase)]
#[phase(plugin)]
extern crate regex_macros;
extern crate regex;

pub use basicclient::{BasicClient, ConnectionStatus};
pub use capability::{Capability, CapabilityModifier};
pub use connection::Connection;
pub use irc_methods::IrcMethods;
pub use message::{Message, MessageErr};
pub use parser::Parser;
pub use prefix::{ClientPrefix, Prefix};
pub use tag::{Tag, TagErr};

mod basicclient;
mod capability;
mod connection;
mod irc_methods;
mod message;
mod parser;
mod prefix;
mod tag;
