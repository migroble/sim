#![deny(clippy::pedantic)]
#![allow(clippy::default_trait_access)]

type PinId = usize;
type ComponentKey = usize;

pub mod components;
mod component;
mod sim;

pub use component::{Component, MetaComponent, PinIO, IO};
pub use sim::Sim;

#[cfg(target_arch = "wasm32")]
pub use component::JsIO;
