use memfs::{
    FileSystem,
    GetEndpointAction,
    GetIntermediateAction,
    GetOptions,
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
        GetIntermediateAction::CreateDefault,
        GetEndpointAction::CreateDefault,
    );

    let endpoint = fs.get_file("/test_1/test_2", options).await;

    match endpoint {
        Ok(Some(_)) => assert!(true),
        _ => assert!(false),
    }

    let intermediate = fs.get_dir("/test_1", Default::default()).await;

    match intermediate {
        Ok(Some(_)) => assert!(true),
        _ => assert!(false),
    }
}
