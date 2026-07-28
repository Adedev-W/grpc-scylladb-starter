use rpc_api::{
    ChannelServiceImpl,
    pb::{
        CreateChannelRequest, GetChannelRequest, ListChannelsRequest,
        channel_service_client::ChannelServiceClient, channel_service_server::ChannelServiceServer,
    },
};
use tokio::sync::oneshot;
use tokio_stream::wrappers::TcpListenerStream;
use tonic::{Request, transport::Channel};

#[tokio::test]
async fn grpc_round_trip_smoke() -> Result<(), Box<dyn std::error::Error>> {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    let incoming = TcpListenerStream::new(listener);
    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

    let server = tokio::spawn(async move {
        tonic::transport::Server::builder()
            .add_service(ChannelServiceServer::new(ChannelServiceImpl::new()))
            .serve_with_incoming_shutdown(incoming, async move {
                let _ = shutdown_rx.await;
            })
            .await
    });

    let endpoint = format!("http://{addr}");
    let channel = Channel::from_shared(endpoint)?.connect().await?;
    let mut client = ChannelServiceClient::new(channel);

    let created = client
        .create_channel(Request::new(CreateChannelRequest {
            name: "general".to_string(),
        }))
        .await?
        .into_inner();

    assert_eq!(created.id, 1);
    assert_eq!(created.name, "general");

    let fetched = client
        .get_channel(Request::new(GetChannelRequest { id: created.id }))
        .await?
        .into_inner();

    assert_eq!(fetched.id, created.id);
    assert_eq!(fetched.name, created.name);

    let page = client
        .list_channels(Request::new(ListChannelsRequest {
            offset: 0,
            limit: 1,
        }))
        .await?
        .into_inner();

    assert_eq!(page.total_count, 1);
    assert_eq!(page.next_offset, 1);
    assert_eq!(page.channels.len(), 1);
    assert_eq!(page.channels[0].name, "general");

    let _ = shutdown_tx.send(());
    server.await.expect("server task panicked")?;

    Ok(())
}
