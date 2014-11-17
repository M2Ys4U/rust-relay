use std::fmt;
use std::str::{Owned, MaybeOwned};

#[unstable]
#[deriving(Clone, PartialEq)]
pub struct Tag<'_> {
    name: MaybeOwned<'_>,
    value: Option<MaybeOwned<'_>>
}

#[experimental]
#[deriving(Show)]
pub enum TagErr {
    EmptyInput,
    MalformedInput
}

#[unstable]
impl<'_> Tag<'_> {
    #[stable]
    pub fn from_str(input: &str) -> Result<Option<Vec<Tag<'_>>>, TagErr> {
        if input.len() == 0 {
            return Err(EmptyInput);
        }

        let mut tags: Vec<Tag> = Vec::new();

        for tag in input.split_terminator(';') {
            let mut t = tag.splitn(1, '=');

            let name = match t.next() {
                Some(s) => { s },
                None => { return Err(MalformedInput); }
            };

            let value = t.next().unwrap_or("");

            if value.chars().find(|c| {*c == '\0' || *c == '\x07' || *c == '\r' || *c == '\n' || *c == ' '}).is_some() {
                return Err(MalformedInput);
            }

            tags.push(Tag {
                name: Owned(unescape(name)),
                value: if value == "" { None } else { Some(Owned(unescape(value))) }
            })
        }

        if tags.iter().len() > 0 {
            Ok(Some(tags))
        } else {
            Ok(None)
        }
    }

    #[experimental]
    pub fn from_parts<'a>(name: MaybeOwned<'a>, value: Option<MaybeOwned<'a>>) -> Tag<'a> {
        Tag {
            name: name,
            value: value
        }
    }

    #[inline]
    #[experimental]
    pub fn name(&'_ self) -> &'_ MaybeOwned<'_> {
        &self.name
    }

    #[inline]
    #[experimental]
    pub fn name_mut(&'_ mut self) -> &'_ mut MaybeOwned<'_> {
        &mut self.name
    }

    #[inline]
    #[experimental]
    pub fn value(&'_ self) -> &'_ Option<MaybeOwned<'_>> {
        &self.value
    }

    #[inline]
    #[experimental]
    pub fn value_mut(&'_ mut self) -> &'_ mut Option<MaybeOwned<'_>> {
        &mut self.value
    }
}

#[stable]
impl<'_> fmt::Show for Tag<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.value {
            Some(ref value) => {
                write!(f, "{}={}", escape(&self.name), escape(value))
            },
            None => {
                write!(f, "{}", escape(&self.name))
            }
        }
    }
}

#[stable]
fn escape(input: &MaybeOwned) -> String {
    input.replace("\\", "\\\\").replace(";", "\\:").replace(" ", "\\s").replace("\0", "\\0").replace("\r", "\\r").replace("\n", "\\n")
}

#[stable]
fn unescape(input: &str) -> String {
    input.to_string().replace("\\n", "\n").replace("\\r", "\r").replace("\\0", "\0").replace("\\s", " ").replace("\\:", ";").replace("\\\\", "\\")
}
