#[allow(warnings)]
#[allow(clippy::all)]
mod generated {
    include!(concat!(env!("OUT_DIR"), "/prior.gate.v1.rs"));
}

pub use generated::*;

