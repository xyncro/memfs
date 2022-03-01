use memfs::{
    EndpointAction,
    FileSystem,
    GetOptions,
    IntermediateAction,
    Node,
};

#[tokio::test]
async fn empty_fs() {
    let fs: FileSystem<u32, u32> = FileSystem::new();

    assert_eq!(fs.count().await, 0);
}

#[tokio::test]
async fn get() {
    let fs: FileSystem<u32, u32> = FileSystem::new();
    let options = GetOptions::create(
        IntermediateAction::CreateDefault,
        EndpointAction::CreateDefault,
    );

    let endpoint = fs.get("/test_1/test_2", options).await;

    match endpoint {
        Ok(Some(Node::File(_))) => assert!(true),
        _ => assert!(false),
    }

    let intermediate = fs.get("/test_1", Default::default()).await;

    match intermediate {
        Ok(Some(Node::Directory(_))) => assert!(true),
        _ => assert!(false),
    }
}
