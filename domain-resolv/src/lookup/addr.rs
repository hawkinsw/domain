//! Looking up host names for addresses.

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::str::FromStr;
use futures::{Async, Future, Poll};
use domain_core::bits::message::RecordIter;
use domain_core::bits::name::{Dname, DnameBuilder, ParsedDname};
use domain_core::iana::Rtype;
use domain_core::rdata::parsed::Ptr;
use ::conf::ResolvOptions;
use ::resolver::{Answer, Query, QueryError, Resolver};


//------------ lookup_addr ---------------------------------------------------

/// Creates a future that resolves into the host names for an IP address. 
///
/// The future will query DNS using the resolver represented by `resolv`.
/// It will query DNS only and not consider any other database the system
/// may have.
/// 
/// The value returned upon success can be turned into an iterator over
/// host names via its `iter()` method. This is due to lifetime issues.
pub fn lookup_addr(resolv: &Resolver, addr: IpAddr) -> LookupAddr {
    let name = dname_from_addr(addr, resolv.options());
    LookupAddr(resolv.query((name, Rtype::Ptr)))
}


//------------ LookupAddr ----------------------------------------------------

/// The future for [`lookup_addr()`].
///
/// [`lookup_addr()`]: fn.lookup_addr.html
pub struct LookupAddr(Query);

impl Future for LookupAddr {
    type Item = FoundAddrs;
    type Error = QueryError;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        Ok(Async::Ready(FoundAddrs(try_ready!(self.0.poll()))))
    }
}


//------------ FoundAddrs ----------------------------------------------------

/// The success type of the `lookup_addr()` function.
///
/// The only purpose of this type is to return an iterator over host names
/// via its `iter()` method.
pub struct FoundAddrs(Answer);

impl FoundAddrs {
    /// Returns an iterator over the host names.
    pub fn iter(&self) -> FoundAddrsIter {
        FoundAddrsIter {
            name: self.0.canonical_name(),
            answer: self.0.answer().ok().map(|sec| sec.limit_to::<Ptr>())
        }
    }
}

impl IntoIterator for FoundAddrs {
    type Item = ParsedDname;
    type IntoIter = FoundAddrsIter;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a> IntoIterator for &'a FoundAddrs {
    type Item = ParsedDname;
    type IntoIter = FoundAddrsIter;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}


//------------ FoundAddrsIter ------------------------------------------------

/// An iterator over host names returned by address lookup.
pub struct FoundAddrsIter {
    name: Option<ParsedDname>,
    answer: Option<RecordIter<Ptr>>,
}

impl Iterator for FoundAddrsIter {
    type Item = ParsedDname;

    #[allow(while_let_on_iterator)]
    fn next(&mut self) -> Option<Self::Item> {
        let name = if let Some(ref name) = self.name { name }
                   else { return None };
        let answer = if let Some(ref mut answer) = self.answer { answer }
                     else { return None };
        while let Some(Ok(record)) = answer.next() {
            if record.owner() == name {
                return Some(record.into_data().into_ptrdname())
            }
        }
        None
    }
}



//------------ Helper Functions ---------------------------------------------

/// Translates an IP address into a domain name.
fn dname_from_addr(addr: IpAddr, opts: &ResolvOptions) -> Dname {
    match addr {
        IpAddr::V4(addr) => dname_from_v4(addr),
        IpAddr::V6(addr) => dname_from_v6(addr, opts)
    }
}

/// Translates an IPv4 address into a domain name.
fn dname_from_v4(addr: Ipv4Addr) -> Dname {
    // XXX There’s a more efficient way to doing this.
    let octets = addr.octets();
    Dname::from_str(
        &format!(
            "{}.{}.{}.{}.in-addr.arpa.", octets[3],
            octets[2], octets[1], octets[0])
    ).unwrap()
}

/// Translate an IPv6 address into a domain name.
///
/// As there are several ways to do this, the functions depends on
/// resolver options, namely `use_bstring` and `use_ip6dotin`.
fn dname_from_v6(addr: Ipv6Addr, opts: &ResolvOptions) -> Dname {
    let mut res = DnameBuilder::new();
    for item in addr.segments().iter().rev() {
        let text = format!("{:04x}", item);
        let text = text.as_bytes();
        res.append_label(&text[3..4]).unwrap();
        res.append_label(&text[2..3]).unwrap();
        res.append_label(&text[1..2]).unwrap();
        res.append_label(&text[0..1]).unwrap();
    }
    res.append_label(b"ip6").unwrap();
    if opts.use_ip6dotint {
        res.append_label(b"int").unwrap();
    }
    else {
        res.append_label(b"arpa").unwrap();
    }
    res.into_dname().unwrap()
}

