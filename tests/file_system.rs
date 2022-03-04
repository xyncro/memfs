use std::path::PathBuf;

use anyhow::Result;
use memfs::{
    directory::{
        Count,
        GetExt,
    },
    node::{
        DataExt,
        Located,
        Root,
    },
    FileSystem,
};

#[tokio::test]
async fn empty_fs() {
    let fs: FileSystem<u32, u32> = FileSystem::new();

    assert!(fs.is_root().await);
    assert_eq!(fs.count().await, 0);
}

#[tokio::test]
async fn get() -> Result<()> {
    let fs: FileSystem<u32, u32> = FileSystem::new();
    let file = fs.get_file_default("/test_1/test_2").await?;
    assert_eq!(file.is_root().await, false);

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

    assert_eq!(PathBuf::from("/test_1/test_2"), file.path().await);

    Ok(())
}
