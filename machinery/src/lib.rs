mod machinery;

pub use machinery_meta::{load_services, machinery, message, service};

pub use serde::{Deserialize, Serialize};

pub use anyhow::Result;

pub type Void = ();

pub mod error {
    pub use anyhow::{anyhow, bail, Context, Error};
}

pub mod json {
    pub use serde_json::{from_str, to_string, to_string_pretty, Value};
}

pub use machinery::{Machinery, MachineryHandler};
