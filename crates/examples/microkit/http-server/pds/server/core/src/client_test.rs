use alloc::sync::Arc;
use alloc::vec;
use core::str;
use core::time::Duration;

use smoltcp::wire::DnsQueryType;

use mbedtls::rng::CtrDrbg;
use mbedtls::ssl::async_io::AsyncIoExt;
use mbedtls::ssl::config::{Endpoint, Preset, Transport};
use mbedtls::ssl::{Config, Context};

use sel4_async_network::{ManagedInterface, TcpSocketError};
use sel4_async_network_mbedtls::{
    get_mozilla_ca_list, insecure_dummy_rng, DbgCallbackBuilder, TcpSocketWrapper,
};
use sel4_async_time::Instant;
use sel4_async_time::TimerManager;

use rustls::version::{TLS12, TLS13};
use rustls::{
    pki_types::ServerName,
    AppDataRecord, ClientConfig, ConnectionState, EncodeError, EncryptError, InsufficientSizeError,
    RootCertStore, UnbufferedStatus,
};

pub async fn run(
    now_fn: impl Fn() -> Instant,
    network_ctx: ManagedInterface,
    timers_ctx: TimerManager,
) {
    timers_ctx
        .sleep_until((now_fn()) + Duration::from_secs(1))
        .await;

    let query = network_ctx
        .dns_query("example.com", DnsQueryType::A)
        .await
        .unwrap();

    let mut socket = network_ctx.new_tcp_socket();
    socket.connect((query[0], 443), 44445).await.unwrap();

    let mut root_store = rustls::RootCertStore::empty();
    root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    let mut config = rustls::ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();
    config.enable_early_data = false;
    config.time_provider = rustls::time_provider::TimeProvider::none();
    let config = Arc::new(config);
    let connector = sel4_async_network_rustls::TcpConnector::from(config);
    let mut conn = connector
        .connect(ServerName::DnsName("example.com".try_into().unwrap()), TcpSocketWrapper::new(socket))
        .unwrap()
        .await
        .unwrap();

    conn.send_all(b"GET / HTTP/1.1\r\n").await.unwrap();
    conn.send_all(b"Host: example.com\r\n").await.unwrap();
    conn.send_all(b"\r\n").await.unwrap();

    let mut buf = vec![0; 4096];
    loop {
        let n = conn.recv(&mut buf).await.unwrap();
        if n == 0 {
            break;
        }
        log::info!("{}", str::from_utf8(&buf[..n]).unwrap());
    }

    // ctx.close_async().await.unwrap();
    // ctx.take_io().unwrap().inner_mut().close().await.unwrap();
    // drop(ctx);

    log::info!("client test complete");
}
