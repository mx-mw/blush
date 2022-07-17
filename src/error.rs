pub mod bag;
pub mod compiler;
pub mod runtime;

pub trait BlushError {}
impl BlushError for () {}
