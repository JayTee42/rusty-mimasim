mod descriptor;
mod fetch;
mod execute;

pub use descriptor::Descriptor;
pub(crate) use fetch::descriptor as fetch_descriptor;
pub(crate) use execute::descriptor as execute_descriptor;
