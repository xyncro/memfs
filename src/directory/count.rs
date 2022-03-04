use async_trait::async_trait;

// Count

#[async_trait]
pub trait Count {
    async fn count(&self) -> usize;

    async fn count_dir(&self) -> usize;

    async fn count_file(&self) -> usize;
}
