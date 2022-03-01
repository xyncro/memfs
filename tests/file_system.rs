use memfs::{
    FileSystem,
    Node,
    OpenEndpoint,
    OpenIntermediate,
};

#[tokio::test]
async fn empty_fs() {
    let fs: FileSystem<u32, u32> = FileSystem::new();

    assert_eq!(fs.count().await, 0);
}

#[tokio::test]
async fn open() {
    let fs: FileSystem<u32, u32> = FileSystem::new();
    let endpoint = fs
        .open(
            "/test_1/test_2",
            OpenEndpoint::Default,
            OpenIntermediate::Default,
        )
        .await;

    match endpoint {
        Ok(Some(Node::File(_))) => assert!(true),
        _ => assert!(false),
    }

    let intermediate = fs
        .open("/test_1", OpenEndpoint::None, OpenIntermediate::None)
        .await;

    match intermediate {
        Ok(Some(Node::Directory(_))) => assert!(true),
        _ => assert!(false),
    }
}
