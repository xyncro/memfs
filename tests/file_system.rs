use memfs::FileSystem;

#[tokio::test]
async fn empty_fs() {
    let fs: FileSystem<u32, u32> = FileSystem::new();

    assert_eq!(fs.count().await, 0);
}

#[tokio::test]
async fn get() {
    let fs: FileSystem<u32, u32> = FileSystem::new();

    let endpoint = fs.get_file_default("/test_1/test_2").await;

    match endpoint {
        Ok(_) => assert!(true),
        _ => assert!(false),
    }

    let intermediate = fs.get_dir("/test_1").await;

    match intermediate {
        Ok(Some(_)) => assert!(true),
        _ => assert!(false),
    }
}
