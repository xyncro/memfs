use memfs::FileSystem;

#[tokio::test]
async fn empty_fs() {
    let fs: FileSystem<u32, u32> = FileSystem::new();

    assert_eq!(fs.root().count().await, 0);
}
