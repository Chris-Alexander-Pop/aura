//! Offensive security RPC handlers — only built with `offensive-security` feature.

mod cve;
mod dns;
mod exploit;
mod nmap;
mod osint;
mod portscan;
mod session;
mod shell;
mod subdomain;
mod web;

use crate::services::ServiceRegistry;

pub fn register(registry: &mut ServiceRegistry) {
    nmap::register(registry);
    subdomain::register(registry);
    dns::register(registry);
    portscan::register(registry);
    web::register(registry);
    osint::register(registry);
    cve::register(registry);
    exploit::register(registry);
    shell::register(registry);
    session::register(registry);
}
