//! Reusable application library for the gRPC backend.

pub mod application;
pub mod bootstrap;
pub mod config;
pub mod domain;
pub mod infrastructure;
pub mod pb {
    tonic::include_proto!("channel");
}
pub mod auth_pb {
    tonic::include_proto!("auth");
}
pub mod transport;

// mod service;

// pub mod client;
// pub mod server;

// pub use service::ChannelServiceImpl;
