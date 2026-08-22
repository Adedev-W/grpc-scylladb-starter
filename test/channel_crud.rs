use grpc_scylladb_starter::pb::{
    Channel, CreateChannelRequest, DeleteChannelRequest, GetChannelRequest, ListChannelsRequest,
    UpdateChannelRequest, channel_service_client::ChannelServiceClient,
};
use tonic::transport::Channel as TransportChannel;

const DEFAULT_ENDPOINT: &str = "http://127.0.0.1:50051";

#[tokio::test]
async fn channel_crud_lifecycle() -> Result<(), Box<dyn std::error::Error>> {
    let endpoint =
        std::env::var("TEST_GRPC_ENDPOINT").unwrap_or_else(|_| DEFAULT_ENDPOINT.to_string());
    let transport = TransportChannel::from_shared(endpoint.clone())?
        .connect()
        .await?;
    let mut client = ChannelServiceClient::new(transport);
    let name = format!("integration-channel-{}", unique_suffix());

    let created = client
        .create_channel(CreateChannelRequest { name: name.clone() })
        .await?
        .into_inner();
    assert_channel(&created, &name);

    let fetched = client
        .get_channel(GetChannelRequest { id: created.id })
        .await?
        .into_inner();
    assert_eq!(fetched, created);

    let updated_name = format!("{name}-updated");
    let updated = client
        .update_channel(UpdateChannelRequest {
            id: created.id,
            name: updated_name.clone(),
        })
        .await?
        .into_inner();
    assert_eq!(updated.id, created.id);
    assert_eq!(updated.name, updated_name);
    assert_eq!(updated.created_at_unix_ms, created.created_at_unix_ms);

    let listed = client
        .list_channels(ListChannelsRequest {
            offset: 0,
            limit: 100,
        })
        .await?
        .into_inner();
    assert!(listed.channels.iter().any(|channel| channel == &updated));

    client
        .delete_channel(DeleteChannelRequest { id: created.id })
        .await?;

    let missing = client
        .get_channel(GetChannelRequest { id: created.id })
        .await
        .expect_err("deleted channel should not be returned");
    assert_eq!(missing.code(), tonic::Code::NotFound);

    Ok(())
}

fn assert_channel(channel: &Channel, expected_name: &str) {
    assert!(channel.id > 0);
    assert_eq!(channel.name, expected_name);
    assert!(channel.created_at_unix_ms > 0);
}

fn unique_suffix() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos()
}
