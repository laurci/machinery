use machinery::{error::bail, Result, Void};

#[machinery::message]
pub enum TimeOfDay {
    Morning,
    Afternoon,
    Evening,
}

#[machinery::message]
pub struct Greeting {
    message: String,
    time_of_day: Option<TimeOfDay>,
}

#[machinery::message]
pub struct GreetingInput {
    name: String,
}

#[machinery::service]
pub async fn format(message: String, input: GreetingInput) -> Result<Greeting> {
    // tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    if input.name == "Laurentiu 4" {
        bail!("invalid name");
    }

    Greeting {
        message: format!("{} {}", message, input.name),
        time_of_day: TimeOfDay::Morning.into(),
    }
    .into()
}

#[machinery::service]
pub async fn say_hi() -> Result<Void> {
    log::info!("hi from say_hi()");
    Ok(())
}
