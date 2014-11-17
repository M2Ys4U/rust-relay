use std::fmt::{Formatter, Result, Show};
use std::hash::{Hash, Writer};

#[stable]
#[deriving(Clone, PartialEq, Eq)]
pub enum CapabilityModifier {
    #[stable]
    Disable,
    #[stable]
    Ack,
    #[stable]
    Sticky
}

#[unstable]
#[deriving(Clone)]
pub struct Capability {
    #[stable]
    pub identifier: String,
    #[unstable]
    pub modifier: Option<CapabilityModifier>,
    #[stable]
    pub value: Option<String>
}

#[unstable]
impl Capability {
    #[unstable]
    pub fn from_str(mut s: &str) -> Option<Capability> {
        s = s.trim();
        if s.len() == 0 {
            None
        } else {
            // FIXME(JA): This should be expanded to allow multiple modifiers
            let modifier = match s.char_at(0) {
                '-' => Some(Disable),
                '~' => Some(Ack),
                '=' => Some(Sticky),
                _   => None
            };
            let slice_from = modifier.is_some() as uint;
            if s.len() < slice_from + 1 {
                None
            } else {
                let val_sep = s.find('=');
                let identifier = String::from_str(s.slice(slice_from, val_sep.unwrap_or(s.len())));
                match val_sep {
                    Some(val_sep) => {
                        Some(Capability {
                            identifier: identifier,
                            modifier: modifier,
                            value: Some(String::from_str(s.slice_from(val_sep)))
                        })
                    },
                    None => {
                        Some(Capability {
                            identifier: identifier,
                            modifier: modifier,
                            value: None
                        })
                    }
                }
            }
        }
    }
}

#[experimental]
impl PartialEq for Capability {
    #[inline]
    fn eq(&self, other: &Capability) -> bool {
        self.identifier.eq(&other.identifier)
    }
}

#[experimental]
impl Eq for Capability {}

#[experimental]
impl<H: Writer> Hash<H> for Capability {
    #[inline]
    fn hash(&self, state: &mut H) {
        self.identifier.hash(state)
    }
}

#[stable]
impl Show for Capability {
    fn fmt<'a>(&self, f: &mut Formatter) -> Result {
        if self.modifier.is_some() {
            try!(write!(f, "{}", match self.modifier.as_ref().unwrap() {
                &Disable => '-',
                &Ack => '~',
                &Sticky => '='
            }));
        }

        try!(write!(f, "{}", self.identifier.as_slice()));

        if self.value.is_some() {
            try!(write!(f, "={}", self.value.as_ref().unwrap().as_slice()));
        }

        Ok(())
    }
}