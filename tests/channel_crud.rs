mod common;

use grpc_scylladb_starter::pb::{
    Channel, CreateChannelRequest, DeleteChannelRequest, GetChannelRequest, ListChannelsRequest,
    UpdateChannelRequest,
};

#[tokio::test]
async fn channel_crud_lifecycle() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = common::connect_client(None).await?;
    let name = format!("integration-channel-{}", unique_suffix());

    let created = client
        .create_channel(CreateChannelRequest { name: name.clone() })
        .await?
        .into_inner();
    println!("create: {created:?}");
    assert_channel(&created, &name);

    let fetched = client
        .get_channel(GetChannelRequest {
            id: created.id.clone(),
        })
        .await?
        .into_inner();
    println!("get: {fetched:?}");
    assert_eq!(fetched, created);

    let updated_name = format!("{name}-updated");
    let updated = client
        .update_channel(UpdateChannelRequest {
            id: created.id.clone(),
            name: updated_name.clone(),
        })
        .await?
        .into_inner();
    println!("update: {updated:?}");
    assert_eq!(updated.id, created.id);
    assert_eq!(updated.name, updated_name);
    assert_eq!(updated.created_at_unix_ms, created.created_at_unix_ms);

    let listed = client
        .list_channels(ListChannelsRequest {
            page_token: Vec::new(),
            limit: 100,
        })
        .await?
        .into_inner();
    println!("list: {listed:?}");
    assert!(listed.channels.iter().any(|channel| channel == &updated));

    client
        .delete_channel(DeleteChannelRequest {
            id: created.id.clone(),
        })
        .await?;
    println!("delete: channel {} deleted", created.id);

    let missing = client
        .get_channel(GetChannelRequest { id: created.id })
        .await
        .expect_err("deleted channel should not be returned");
    println!(
        "get after delete: code={:?}, message={}",
        missing.code(),
        missing.message()
    );
    assert_eq!(missing.code(), tonic::Code::NotFound);

    Ok(())
}

fn assert_channel(channel: &Channel, expected_name: &str) {
    assert!(!channel.id.is_empty());
    assert_eq!(channel.name, expected_name);
    assert!(channel.created_at_unix_ms > 0);
}

fn unique_suffix() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos()
}
