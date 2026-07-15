use super::*;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
#[tokio::test]
async fn binds_to_ephemeral_loopback_port() {
    let path = std::env::temp_dir().join(format!(
        "culinograph-service-{}.sqlite3",
        uuid::Uuid::new_v4()
    ));
    let state =
        ServiceState::sqlite(path.clone(), path.with_file_name("settings.json")).expect("state");
    let config = ServiceConfig {
        state,
        access: AccessPolicy::new("token", ["http://localhost".to_owned()]),
        allowed_origins: vec!["http://localhost".to_owned()],
    };
    let service = bind(config, SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0))
        .await
        .expect("bind");
    assert_ne!(service.local_addr().expect("address").port(), 0);
    let _ = std::fs::remove_file(path);
}
