use culinator_service::{AccessPolicy, ServiceConfig, ServiceState, bind};
use std::{
    env,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::PathBuf,
};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = Options::parse(env::args().skip(1))
        .map_err(|message| std::io::Error::new(std::io::ErrorKind::InvalidInput, message))?;
    let token = options
        .token
        .unwrap_or_else(|| Uuid::new_v4().simple().to_string());
    let origins = if options.origins.is_empty() {
        vec!["http://localhost:1420".to_owned()]
    } else {
        options.origins
    };
    let access = AccessPolicy::new(token.clone(), origins.clone())
        .allow_missing_origin(options.allow_missing_origin);
    let service = bind(
        ServiceConfig {
            state: ServiceState::sqlite(
                options.database.clone(),
                options.database.with_file_name("settings.json"),
            )?,
            access,
            allowed_origins: origins,
        },
        SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), options.port),
    )
    .await?;
    let address = service.local_addr()?;

    println!(
        "{}",
        serde_json::json!({
            "endpoint": format!("http://{address}"),
            "websocketUrl": format!("ws://{address}/ws"),
            "token": token,
        })
    );

    let shutdown = CancellationToken::new();
    let signal = shutdown.clone();
    tokio::spawn(async move {
        let _ = tokio::signal::ctrl_c().await;
        signal.cancel();
    });
    service.serve(shutdown).await?;
    Ok(())
}

struct Options {
    database: PathBuf,
    port: u16,
    token: Option<String>,
    origins: Vec<String>,
    allow_missing_origin: bool,
}

impl Options {
    fn parse(args: impl Iterator<Item = String>) -> Result<Self, String> {
        let mut options = Self {
            database: PathBuf::from("culinator-dev.sqlite3"),
            port: 0,
            token: None,
            origins: Vec::new(),
            allow_missing_origin: false,
        };
        let mut args = args.peekable();
        while let Some(argument) = args.next() {
            match argument.as_str() {
                "--db" => options.database = PathBuf::from(next_value(&mut args, "--db")?),
                "--port" => {
                    options.port = next_value(&mut args, "--port")?
                        .parse()
                        .map_err(|_| "--port must be a valid u16".to_owned())?;
                }
                "--token" => options.token = Some(next_value(&mut args, "--token")?),
                "--origin" => options.origins.push(next_value(&mut args, "--origin")?),
                "--allow-missing-origin" => options.allow_missing_origin = true,
                "--help" | "-h" => {
                    return Err("usage: culinator-service [--db PATH] [--port PORT] [--token TOKEN] [--origin ORIGIN] [--allow-missing-origin]".to_owned());
                }
                other => return Err(format!("unknown argument: {other}")),
            }
        }
        Ok(options)
    }
}

fn next_value(
    args: &mut std::iter::Peekable<impl Iterator<Item = String>>,
    flag: &str,
) -> Result<String, String> {
    args.next()
        .ok_or_else(|| format!("{flag} requires a value"))
}
#[cfg(test)]
#[path = "test.rs"]
mod test;
