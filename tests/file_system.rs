use anyhow::Result;
use memfs::{
    DataExt,
    FileSystem,
};

#[tokio::test]
async fn empty_fs() {
    let fs: FileSystem<u32, u32> = FileSystem::new();

    assert_eq!(fs.count().await, 0);
}

#[tokio::test]
async fn get() -> Result<()> {
    let fs: FileSystem<u32, u32> = FileSystem::new();
    let file = fs.get_file_default("/test_1/test_2").await?;

    let value = file.read(|value| *value).await;
    assert_eq!(value, 0);

    file.write(|mut value| *value += 1).await;

    let value = file.read(|value| *value).await;
    assert_eq!(value, 1);

    let intermediate = fs.get_dir("/test_1").await?;

    match intermediate {
        Some(_) => assert!(true),
        _ => assert!(false),
    }

    Ok(())
}
