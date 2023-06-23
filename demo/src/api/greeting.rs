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

#[machinery::message]
pub struct GreetingInput {
    name: String,
}

#[machinery::service]
pub async fn hello(message: String, input: GreetingInput) -> Result<Greeting> {
    let user = inject!(User)?;

    if input.name == "Laurentiu" {
        bail!("You are not welcome here!");
    }

    Greeting {
        message: format!("{} {} {}", message, input.name, user.id),
        thing: Thing::A.into(),
    }
    .into()
}

#[machinery::service]
pub async fn hi() -> Result<Void> {
    Ok(())
}
