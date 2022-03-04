use async_trait::async_trait;

// Name

#[async_trait]
pub trait Named {
    async fn name(&self) -> Option<String>;
}
