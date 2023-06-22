use machinery::{error::bail, inject, Result, Void};

use crate::user::User;

#[machinery::message]
pub enum Thing {
    A,
    B,
    C,
}

#[machinery::message]
pub struct Greeting {
    message: String,
    thing: Option<Thing>,
}

#[machinery::service]
pub async fn hello(message: String) -> Result<Greeting> {
    let user = inject!(User)?;

    if message == "Laur" {
        bail!("I dont like you, {}", message);
    }

    Greeting {
        message: format!("Hello, {} {}", message, user.id),
        thing: Thing::A.into(),
    }
    .into()
}

#[machinery::service]
pub async fn hi() -> Result<Void> {
    Ok(())
}
