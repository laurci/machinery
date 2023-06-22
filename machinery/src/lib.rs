mod machinery;

pub use machinery_meta::{
    inject, inject_async, injectable, load_services, machinery, message, service,
};

pub use serde::{Deserialize, Serialize};

pub use anyhow::Result;

pub type Void = ();

pub mod error {
    pub use anyhow::{anyhow, bail, Context, Error};
}

pub mod __internal_async {
    pub use async_trait::async_trait;
}

pub mod json {
    pub use serde_json::{from_str, to_string, to_string_pretty, Value};
}

pub use machinery::{Machinery, MachineryHandler, MachineryStandaloneConfig};

pub mod context;
pub mod inject;
