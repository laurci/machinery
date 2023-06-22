use machinery::{context::Context, error::bail, inject::Injectable, Result};

pub struct User {
    pub id: String,
}

#[machinery::injectable]
impl Injectable for User {
    fn inject(ctx: &Context) -> Result<Box<User>> {
        let Some(user_id) = ctx.headers.get("user_id") else {
            bail!("user_id is not found");
        };

        let Ok(user_id) = user_id.to_str() else {
            bail!("user_id is not found");
        };

        Ok(Box::new(User {
            id: user_id.to_owned(),
        }))
    }
}
