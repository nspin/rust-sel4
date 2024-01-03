use alloc::sync::Arc;
use alloc::vec;
use core::fmt;
use core::str;
use core::time::Duration;

use smoltcp::wire::DnsQueryType;

use mbedtls::ssl::async_io::AsyncIoExt;

use sel4_async_network::{ManagedInterface, TcpSocketError};
use sel4_async_network_mbedtls::{
    get_mozilla_ca_list, insecure_dummy_rng, DbgCallbackBuilder, TcpSocketWrapper,
};
use sel4_async_time::Instant;
use sel4_async_time::TimerManager;

use rustls::version::{TLS12, TLS13};
use rustls::{
    pki_types::{ServerName, UnixTime},
    time_provider::GetCurrentTime,
    AppDataRecord, ClientConfig, ConnectionState, EncodeError, EncryptError, InsufficientSizeError,
    RootCertStore, UnbufferedStatus,
};

const NOW: u64 = 1704284617;

pub async fn run(
    now_fn: impl 'static + Send + Sync + Fn() -> Instant,
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
    let mut config = rustls::ClientConfig::builder_with_protocol_versions(&[&TLS12])
        .with_root_certificates(root_store)
        .with_no_client_auth();
    config.enable_early_data = false;
    config.time_provider = rustls::time_provider::TimeProvider::new(GetCurrentTimeImpl::new(
        UnixTime::since_unix_epoch(Duration::from_secs(NOW)),
        now_fn,
    ));
    let config = Arc::new(config);
    let connector = sel4_async_network_rustls::TcpConnector::from(config);
    let mut conn = connector
        .connect(
            ServerName::DnsName("example.com".try_into().unwrap()),
            TcpSocketWrapper::new(socket),
        )
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

struct GetCurrentTimeImpl<F> {
    start_global: UnixTime,
    start_local: Instant,
    now_fn: F,
}

impl<F> fmt::Debug for GetCurrentTimeImpl<F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GetCurrentTimeImpl").finish()
    }
}

impl<F: Send + Sync + Fn() -> Instant> GetCurrentTimeImpl<F> {
    fn new(now_global: UnixTime, now_fn: F) -> Self {
        let start_local = (now_fn)();
        Self {
            start_global: now_global,
            start_local,
            now_fn,
        }
    }
}

impl<F: Send + Sync + Fn() -> Instant> GetCurrentTime for GetCurrentTimeImpl<F> {
    fn get_current_time(&self) -> Option<UnixTime> {
        Some(UnixTime::since_unix_epoch(
            Duration::from_secs(self.start_global.as_secs()) + ((self.now_fn)() - self.start_local),
        ))
    }
}
