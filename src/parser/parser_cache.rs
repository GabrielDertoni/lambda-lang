
use std::collections::HashMap;
use std::any::{ Any, TypeId };

use super::error::Error;
use super::Result;

pub struct ParsedType {
    pub parses_until: usize,
    pub ty: Box<dyn Any>,
}

impl ParsedType {
    pub fn new<T: Any>(parses_until: usize, ty: T) -> ParsedType {
        ParsedType {
            parses_until,
            ty: Box::new(ty),
        }
    }

    #[inline]
    pub fn get_ref<T: Any>(&self) -> Option<&T> {
        self.ty.downcast_ref::<T>()
    }
}

impl<T: Any> AsRef<T> for ParsedType {
    fn as_ref(&self) -> &T {
        self.get_ref()
            .expect("Tried to convert ParsedType to a different type")
    }
}

// A map of how many bytes there were previous to parsing to the things that
// can be parsed from there.
pub type ParserCache = HashMap<usize, HashMap<TypeId, ParsedType>>;

/*
pub struct ParserCache {
    pub cache: HashMap<usize, HashMap<TypeId, Box<dyn Any>>>,
}
*/

/*
impl ParserCache {
    pub fn new() -> ParserCache {
        ParserCache {
            cache: HashMap::new(),
        }
    }
}
*/
