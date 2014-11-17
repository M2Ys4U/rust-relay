# Rust-Relay

Rust-Relay is an IRC(v3) client library written in [Rust](https://rust-lang.org/).

## Components

This is a brief overview of the components provided by this library. For full details, see the [generated RustDoc files](http://allnutt.eu/rust/doc/rust-relay/index.html).

### `BasicClient` Struct

This is, as the name suggests, a basic IRC client. It can connect to an IRC server, perform [IRCv3 capability negotiation](https://github.com/ircv3/ircv3-specifications/blob/master/specification/capability-negotiation-3.1) and keep the connection live by responding to `PING` commands.

Implements the `IrcMethods` trait for convenience.

### `Connection` Struct

Represents a connection to an IRC server.

Implements the `IrcMethods` traid for convenience.

### `IrcMethods` Trait

Convenience methods that can be applied to any `Writer` to send IRC commands (or at least the ones defined in RFC 1459 and the IRCv3 extensions.

### `Parser` Struct

Reads IRC messages from a `Reader` and produces `Message`s.

## Project Status

This is a rough first-draft implementation.

Handing `Message`s is a little messy as they are currently backed by `MaybeOwned`s to avoid allocations for certain use-cases. I expect this will get easier as the Rust stdlib stablises.

Feedback on the APIs would be much appreciated.

## Example code

Here is a *very* crude CLI IRC client implemented using the library:

```Rust
extern crate relay;

use std::collections::HashSet;
use std::io::stdio::stdin;
use std::comm::{Empty, Disconnected};

use relay::{Message, BasicClient, Capability, MessageErr};

fn main() {
    let mut caps = HashSet::new();
    caps.insert(Capability::from_str("multi-prefix").unwrap());

    let mut client = BasicClient::new("Relay_Test", "relay", "relay", caps);
    match client.connect_to("media-server.local:6667") {
        Err(e) => {
            panic!(e);
        },
        Ok(_) =>{}
    }
    let mut stream = client.get_stream().clone();

    let (tx, rx) = channel();
    spawn(proc() {
        let tx = tx;
        loop {
            match client.read_message() {
                Ok(msg_opt) => {
                    match msg_opt {
                        Some(msg) => {
                            tx.send(msg);
                        },
                        None => {}
                    }
                },
                Err(e) => {
                    panic!(e);
                }
            }
        }
    });

    let mut std_in = stdin();
    loop {
        'dance: loop {
            match rx.try_recv() {
                Ok(msg) => {
                    print!("-> {}", msg);
                },
                Err(e) => {
                    match e {
                        Empty => {
                            break 'dance;
                        },
                        Disconnected => {
                            return;
                        }
                    }
                }
            }
        }
        match Message::from_str(std_in.read_line().unwrap().as_slice()) {
            Ok(msg) => {
                let _ = write!(stream, "{}", msg);
            },
            Err(e) => {
                match e {
                    MessageErr::EmptyInput => {},
                    _ => {
                        println!("{}", e);
                    }
                }
            }
        }
    }

}
```

## License
Rust-Relay is licenced under version 3 of the GNU Lesser General Public License (GNU LGPL) or any later version. See the `COPYING` and `COPYING.LESSER` files for more information.