//! Library surface for integration tests and shared registry setup.
pub mod notify;
pub mod rpc;
pub mod server;
pub mod services;
pub mod types;
pub mod utils;

use crate::services::ServiceRegistry;

/// Register all sidecar RPC handlers (same set as the `ags-sidecar` binary).
pub fn build_registry() -> ServiceRegistry {
    let mut registry = ServiceRegistry::new();
    services::power::register(&mut registry);
    services::network::register(&mut registry);
    services::system::register(&mut registry);
    services::brightness::register(&mut registry);
    services::vpn::register(&mut registry);
    services::weather::register(&mut registry);
    services::gamemode::register(&mut registry);
    services::storage::register(&mut registry);
    services::audio::register(&mut registry);
    services::bluetooth::register(&mut registry);
    services::performance::register(&mut registry);
    services::security::register(&mut registry);
    services::devops::register(&mut registry);
    services::productivity::register(&mut registry);
    services::calendar::register(&mut registry);
    services::logs::register(&mut registry);
    services::packages::register(&mut registry);
    services::automation::register(&mut registry);
    services::communication::register(&mut registry);
    services::fitness::register(&mut registry);
    services::hyprland::register(&mut registry);
    services::shell::register(&mut registry);
    services::mpris::register(&mut registry);
    services::processes::register(&mut registry);
    services::notifications::register(&mut registry);
    services::keybinds::register(&mut registry);
    registry
}
