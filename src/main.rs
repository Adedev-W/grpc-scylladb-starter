use scylla::client::session::Session;
use scylla::client::session_builder::SessionBuilder;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
   let session: Session = SessionBuilder::new()
        .known_node("127.0.0.1:9042")
        .known_node("1.2.3.4:9876")
        .build()
        .await?;

   Ok(())
}