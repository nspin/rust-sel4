use alloc::sync::Arc;
use alloc::vec;
use core::str;
use core::time::Duration;

use mbedtls::ssl::async_io::AsyncIoExt;
use rustls::pki_types::{ServerName, UnixTime};
use rustls::version::TLS12;
use rustls::ClientConfig;
use smoltcp::wire::DnsQueryType;

use sel4_async_network::ManagedInterface;
use sel4_async_network_mbedtls::TcpSocketWrapper;
use sel4_async_network_rustls::{GetCurrentTimeImpl, NoServerCertVerifier};
use sel4_async_time::{Instant, TimerManager};

const NOW: u64 = 1704284617;

const DOMAIN: &str = "example.com";
const PORT: u16 = 443;

// const DOMAIN: &str = "localhost";
// const PORT: u16 = 44330;

pub async fn run(
    now_fn: impl 'static + Send + Sync + Fn() -> Instant,
    network_ctx: ManagedInterface,
    timers_ctx: TimerManager,
) {
    timers_ctx
        .sleep_until((now_fn()) + Duration::from_secs(1))
        .await;

    let query = if DOMAIN != "localhost" {
        network_ctx
            .dns_query(DOMAIN, DnsQueryType::A)
            .await
            .unwrap()
    } else {
        vec![smoltcp::wire::IpAddress::v4(127, 0, 0, 1)]
    };

    let mut socket = network_ctx.new_tcp_socket();
    socket.connect((query[0], PORT), 44445).await.unwrap();

    let mut root_store = rustls::RootCertStore::empty();
    root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    let mut config = rustls::ClientConfig::builder_with_protocol_versions(&[&TLS12])
        .with_root_certificates(root_store)
        .with_no_client_auth();
    config.enable_early_data = false;
    config.time_provider = rustls::time_provider::TimeProvider::new(GetCurrentTimeImpl::new(
        UnixTime::since_unix_epoch(Duration::from_secs(NOW)),
        now_fn,
    ));

    let mut dangerous_config = ClientConfig::dangerous(&mut config);
    dangerous_config.set_certificate_verifier(Arc::new(NoServerCertVerifier));

    let config = Arc::new(config);
    let connector = sel4_async_network_rustls::async_io::client::TcpConnector::from(config);
    let mut conn = connector
        .connect(
            ServerName::DnsName(DOMAIN.try_into().unwrap()),
            TcpSocketWrapper::new(socket),
        )
        .unwrap()
        .await
        .unwrap();

    log::debug!("XXXXX handshake done");

    conn.send_all(b"GET / HTTP/1.1\r\n").await.unwrap();
    log::debug!("XXXXX a1");
    conn.send_all(b"Host: example.com\r\n").await.unwrap();
    log::debug!("XXXXX a2");
    conn.send_all(b"\r\n").await.unwrap();
    log::debug!("XXXXX a3");
    conn.flush().await.unwrap();
    log::debug!("XXXXX b");

    const BUF_SIZE: usize = 1024 * 64;
    // const BUF_SIZE: usize = 4096;

    let mut buf = vec![0; BUF_SIZE];
    loop {
        let n = conn.recv(&mut buf).await.unwrap();
        log::debug!("XXXXX c1 {}", n);
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
