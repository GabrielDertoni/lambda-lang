
use std::collections::HashMap;
use std::any::{ Any, TypeId };

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

use std::rc::Rc;
use std::marker::PhantomData;
use std::ops::Deref;

pub struct TypePtr<T> {
    rc: Rc<dyn Any>,
    _m: PhantomData<T>,
}

impl<T: 'static + Any> TypePtr<T> {
    pub fn try_new(rc: Rc<dyn Any>) -> Option<TypePtr<T>> {
        match rc.as_ref().downcast_ref::<T>() {
            Some(_) => Some({
                TypePtr {
                    rc,
                    _m: PhantomData,
                }
            }),
            None => None,
        }
    }
}

impl<T: 'static + Any> Deref for TypePtr<T> {
    type Target = T;

    fn deref(&self) -> &T {
        // This is ok because we have already checked that this referece really
        // points to something that is of type T.
        self.rc.as_ref().downcast_ref::<T>().unwrap()
    }
}

// A map of how many bytes there were previous to parsing to the things that
// can be parsed from there.
// pub type ParserCache = HashMap<usize, HashMap<TypeId, ParsedType>>;
pub type ParserCache = HashMap<usize, HashMap<TypeId, Result<ParsedType>>>;
// pub type ParserCache = BTreeMap<Span, HashMap<TypeId, Result<Box<dyn Any>>>>;
// pub type ParserCache = BTreeMap<Span, (TypeId, Result<ParsedType>)>;


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
