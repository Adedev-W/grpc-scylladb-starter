//! In-memory gRPC Channel service with shared client and server helpers.

pub mod pb {
    tonic::include_proto!("channel");
}

mod service;
mod store;

pub mod client;
pub mod server;

pub use service::ChannelServiceImpl;
