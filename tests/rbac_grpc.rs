mod common;

use grpc_scylladb_starter::pb::{CreateChannelRequest, DeleteChannelRequest, ListChannelsRequest};

#[tokio::test]
async fn roles_are_enforced_by_the_grpc_service() -> Result<(), Box<dyn std::error::Error>> {
    let endpoint = std::env::var("TEST_GRPC_ENDPOINT").unwrap_or_default();
    if !endpoint.starts_with("https://") {
        eprintln!("skipping mTLS RBAC test because TEST_GRPC_ENDPOINT is not HTTPS");
        return Ok(());
    }

    assert!(
        common::connect_without_client_certificate().await.is_err(),
        "the server must reject clients without a certificate"
    );

    let mut reader = common::connect_client(Some("reader.example")).await?;
    let denied_create = reader
        .create_channel(CreateChannelRequest {
            name: "reader-must-not-create".into(),
        })
        .await
        .expect_err("reader must not create channels");
    assert_eq!(denied_create.code(), tonic::Code::PermissionDenied);

    let mut writer = common::connect_client(Some("writer.example")).await?;
    let created = writer
        .create_channel(CreateChannelRequest {
            name: "writer-owned-channel".into(),
        })
        .await?
        .into_inner();
    let denied_delete = writer
        .delete_channel(DeleteChannelRequest {
            id: created.id.clone(),
        })
        .await
        .expect_err("writer must not delete channels");
    assert_eq!(denied_delete.code(), tonic::Code::PermissionDenied);

    let mut admin = common::connect_client(Some("admin.example")).await?;
    admin
        .delete_channel(DeleteChannelRequest { id: created.id })
        .await?;
    let listed = admin
        .list_channels(ListChannelsRequest {
            page_token: Vec::new(),
            limit: 1,
        })
        .await?;
    assert!(listed.into_inner().channels.len() <= 1);

    Ok(())
}
