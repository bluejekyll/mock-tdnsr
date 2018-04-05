extern crate futures;
extern crate tokio_core;
extern crate trust_dns_proto;
extern crate trust_dns_resolver;

use futures::Future;
use std::net::Ipv4Addr;
use tokio_core::reactor::{Core, Handle};
use trust_dns_proto::rr::RData;
use trust_dns_resolver::ResolverFuture;
use trust_dns_resolver::config::{ResolverConfig, ResolverOpts};
use trust_dns_resolver::error::*;
use trust_dns_resolver::lookup::Lookup;
use trust_dns_resolver::lookup_ip::LookupIp;

pub trait ResolverWrapper {
    fn lookup_ip(&self, host: &str) -> Box<Future<Item = LookupIp, Error = ResolveError>>;
}

pub struct ResolverWrapperReal {
    delegate: ResolverFuture,
}

impl ResolverWrapper for ResolverWrapperReal {
    fn lookup_ip(&self, host: &str) -> Box<Future<Item = LookupIp, Error = ResolveError>> {
        Box::new(self.delegate.lookup_ip(host))
    }
}

pub struct ResolverWrapperFactory;
impl ResolverWrapperFactory {
    fn resolver(
        &self,
        config: ResolverConfig,
        options: ResolverOpts,
        reactor: &Handle,
    ) -> Box<ResolverWrapper> {
        Box::new(ResolverWrapperReal {
            delegate: ResolverFuture::new(config, options, reactor),
        })
    }
}

pub struct MockResolveWrapper;
impl ResolverWrapper for MockResolveWrapper {
    fn lookup_ip(&self, _host: &str) -> Box<Future<Item = LookupIp, Error = ResolveError>> {
        let lookup: Lookup = RData::A(Ipv4Addr::new(127, 0, 0, 1)).into();
        let lookup_ip = lookup.into();

        Box::new(futures::future::ok(lookup_ip))
    }
}

pub struct MockResolveWrapperFactory;
impl MockResolveWrapperFactory {
    fn resolver(&self) -> Box<ResolverWrapper> {
        Box::new(MockResolveWrapper)
    }
}

fn lookup(
    resolver: Box<ResolverWrapper>,
    host: &str,
) -> Box<Future<Item = LookupIp, Error = ResolveError>> {
    resolver.lookup_ip(host)
}

fn main() {
    let mut core = Core::new().expect("core failed");

    let real =
        ResolverWrapperFactory.resolver(Default::default(), Default::default(), &core.handle());
    let mock = MockResolveWrapperFactory.resolver();

    println!(
        "real lookup: {:#?}",
        core.run(lookup(real, "www.example.com."))
            .expect("lookup failed")
    );
    println!(
        "mock lookup: {:#?}",
        core.run(lookup(mock, "www.example.com."))
            .expect("lookup failed")
    );
}
