use crate::pb;
use std::fmt::Display;
use thiserror::Error;
use tonic::{Request, transport::Channel};

const DEFAULT_ENDPOINT: &str = "http://127.0.0.1:50051";
const DEFAULT_LIST_LIMIT: u32 = 50;
const USAGE: &str = "Usage: rpc-api [--endpoint URI] <create|get|list> ...\n\nCommands:\n  create <name>\n  get <id>\n  list [offset] [limit]\n";

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("{0}")]
    Usage(String),
    #[error("{0}")]
    InvalidEndpoint(String),
    #[error(transparent)]
    Transport(#[from] tonic::transport::Error),
    #[error(transparent)]
    Status(#[from] tonic::Status),
    #[error(transparent)]
    ParseInt(#[from] std::num::ParseIntError),
}

pub async fn run_from_env() -> Result<(), ClientError> {
    run_from_args(std::env::args().skip(1)).await
}

pub async fn run_from_args<I>(args: I) -> Result<(), ClientError>
where
    I: IntoIterator<Item = String>,
{
    let cli = parse_args(args.into_iter())?;
    execute(cli).await
}

struct Cli {
    endpoint: String,
    command: Command,
}

enum Command {
    Create { name: String },
    Get { id: u64 },
    List { offset: u64, limit: u32 },
}

fn parse_args(mut args: impl Iterator<Item = String>) -> Result<Cli, ClientError> {
    let mut endpoint = DEFAULT_ENDPOINT.to_string();
    let mut command_name = None;
    let mut remainder = Vec::new();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => return Err(ClientError::Usage(USAGE.to_string())),
            "-e" | "--endpoint" => {
                endpoint = args
                    .next()
                    .ok_or_else(|| ClientError::Usage("--endpoint requires a value".to_string()))?;
            }
            other => {
                command_name = Some(other.to_string());
                remainder.extend(args);
                break;
            }
        }
    }

    let command_name = command_name.ok_or_else(|| ClientError::Usage(USAGE.to_string()))?;
    let command = parse_command(&command_name, remainder.into_iter())?;

    Ok(Cli { endpoint, command })
}

fn parse_command(
    command: &str,
    mut args: impl Iterator<Item = String>,
) -> Result<Command, ClientError> {
    match command {
        "create" => {
            let name = args
                .next()
                .ok_or_else(|| ClientError::Usage("create requires a name".to_string()))?;
            ensure_no_extra(args, "create")?;
            Ok(Command::Create { name })
        }
        "get" => {
            let id = parse_u64_arg(args.next(), "get requires an id")?;
            ensure_no_extra(args, "get")?;
            Ok(Command::Get { id })
        }
        "list" => {
            let offset = match args.next() {
                Some(value) => value.parse()?,
                None => 0,
            };
            let limit = match args.next() {
                Some(value) => value.parse()?,
                None => DEFAULT_LIST_LIMIT,
            };
            ensure_no_extra(args, "list")?;
            Ok(Command::List { offset, limit })
        }
        other => Err(ClientError::Usage(format!(
            "unknown command `{other}`\n\n{USAGE}"
        ))),
    }
}

fn parse_u64_arg(value: Option<String>, message: &str) -> Result<u64, ClientError> {
    let value = value.ok_or_else(|| ClientError::Usage(message.to_string()))?;
    Ok(value.parse()?)
}

fn ensure_no_extra(
    mut args: impl Iterator<Item = String>,
    command: &str,
) -> Result<(), ClientError> {
    if let Some(extra) = args.next() {
        return Err(ClientError::Usage(format!(
            "{command} does not accept extra argument `{extra}`\n\n{USAGE}"
        )));
    }

    Ok(())
}

fn normalize_endpoint(endpoint: &str) -> String {
    if endpoint.contains("://") {
        endpoint.to_string()
    } else {
        format!("http://{endpoint}")
    }
}

async fn execute(cli: Cli) -> Result<(), ClientError> {
    // Build the channel once so every RPC reuses the same HTTP/2 transport.
    let channel = Channel::from_shared(normalize_endpoint(&cli.endpoint))
        .map_err(|err| ClientError::InvalidEndpoint(err.to_string()))?
        .connect()
        .await?;
    let mut client = pb::channel_service_client::ChannelServiceClient::new(channel);

    match cli.command {
        Command::Create { name } => {
            let response = client
                .create_channel(Request::new(pb::CreateChannelRequest { name }))
                .await?
                .into_inner();
            print_channel(&response);
        }
        Command::Get { id } => {
            let response = client
                .get_channel(Request::new(pb::GetChannelRequest { id }))
                .await?
                .into_inner();
            print_channel(&response);
        }
        Command::List { offset, limit } => {
            let response = client
                .list_channels(Request::new(pb::ListChannelsRequest { offset, limit }))
                .await?
                .into_inner();
            print_list(response);
        }
    }

    Ok(())
}

fn print_channel(channel: &pb::Channel) {
    println!(
        "id={} name={} created_at_unix_ms={}",
        channel.id, channel.name, channel.created_at_unix_ms
    );
}

fn print_list(response: pb::ListChannelsResponse) {
    println!(
        "total_count={} next_offset={}",
        response.total_count, response.next_offset
    );

    for channel in response.channels {
        print_channel(&channel);
    }
}

impl Display for Cli {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.command {
            Command::Create { name } => write!(f, "endpoint={} create {}", self.endpoint, name),
            Command::Get { id } => write!(f, "endpoint={} get {}", self.endpoint, id),
            Command::List { offset, limit } => {
                write!(f, "endpoint={} list {} {}", self.endpoint, offset, limit)
            }
        }
    }
}
