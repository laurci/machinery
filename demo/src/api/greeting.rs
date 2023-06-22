use machinery::{error::bail, Result, Void};

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
    if message == "Laur" {
        bail!("I dont like you, {}", message);
    }

    Greeting {
        message: format!("Hello, {}", message),
        thing: Thing::A.into(),
    }
    .into()
}

#[machinery::service]
pub async fn hi() -> Result<Void> {
    Ok(())
}
