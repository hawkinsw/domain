//! Generic Record Data Types

use std::fmt;
use super::super::compose::ComposeBytes;
use super::super::flavor::Flavor;
use super::super::error::{ComposeResult, ParseResult};
use super::super::iana::RRType;
use super::super::nest::Nest;
use super::super::parse::ParseFlavor;
use super::traits::RecordData;


pub struct GenericRecordData<'a, F: Flavor<'a>> {
    rtype: RRType,
    data: F::Nest,
}

impl<'a, F: Flavor<'a>> GenericRecordData<'a, F> {
    pub fn new(rtype: RRType, data: F::Nest) -> Self {
        GenericRecordData { rtype: rtype, data: data }
    }

    pub fn rtype(&self) -> RRType { self.rtype }
    pub fn data(&self) -> &F::Nest { &self.data }

    /*
    pub fn fmt<R: RecordData<'a, F>>(&'a self, f: &mut fmt::Formatter)
                                     -> fmt::Result {
        let mut parser = self.data.parser();
        match R::parse(self.rtype, &mut parser) {
            Err(..) => Ok(()),
            Ok(None) => Ok(()),
            Ok(Some(data)) => data.fmt(f)
        }
    }
    */
}


impl<'a, F: Flavor<'a>> RecordData<'a, F> for GenericRecordData<'a, F> {

    fn rtype(&self) -> RRType {
        self.rtype
    }

    fn compose<C: ComposeBytes>(&self, target: &mut C) -> ComposeResult<()> {
        self.data.compose(target)
    }

    fn parse<P>(rtype: RRType, parser: &mut P) -> ParseResult<Option<Self>>
             where P: ParseFlavor<'a, F> {
        let len = parser.left();
        let nest = try!(parser.parse_nest(len));
        Ok(Some(GenericRecordData::new(rtype, nest)))
    }
}


impl<'a, F: Flavor<'a>> fmt::Display for GenericRecordData<'a, F> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        /*
        use super::rfc1035::*;

        match self.rtype {
            RRType::A => self.fmt::<A>(f),
            RRType::NS => self.fmt::<NS<F>>(f),
            _ => "...".fmt(f)
        }
        */
        "...".fmt(f)
    }
}
