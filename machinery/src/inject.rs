use anyhow::Result;

use crate::context::Context;

#[async_trait::async_trait]
pub trait AsyncInjectable {
    async fn inject(ctx: &Context) -> Result<Box<Self>>;
}

pub trait Injectable {
    fn inject(ctx: &Context) -> Result<Box<Self>>;
}

pub mod __internal {
    use anyhow::Result;

    use crate::context::Context;

    use super::{AsyncInjectable, Injectable};

    pub async fn inject_async<T: AsyncInjectable>(ctx: &Context) -> Result<T> {
        let boxed = T::inject(ctx).await?;
        Ok(*boxed)
    }

    pub fn inject<T: Injectable>(ctx: &Context) -> Result<T> {
        let boxed = T::inject(ctx)?;
        Ok(*boxed)
    }
}
