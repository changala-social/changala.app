//! Handler implementations for Changala XRPC procedures and queries.
//!
//! The generated route stubs in `src/generated/routes.rs` wire XRPC paths
//! to auto-generated placeholder handlers. As you implement business logic,
//! move the implementations here, organised by service domain.

pub mod admin;
pub mod archive;
pub mod auth;
pub mod brain;
pub mod course;
pub mod events;
pub mod feed;
pub mod graph;
pub mod identity;
pub mod moderation;
pub mod note;
pub mod notification;
pub mod search;
pub mod session;
