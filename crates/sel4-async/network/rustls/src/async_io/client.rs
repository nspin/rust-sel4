use core::future;
use core::mem;
use core::pin::Pin;
use core::task::{self, Poll};

use alloc::sync::Arc;

use futures::Future;
use rustls::client::UnbufferedClientConnection;
use rustls::pki_types::ServerName;
use rustls::unbuffered::{
    AppDataRecord, ConnectionState, EncodeError, EncryptError, UnbufferedStatus,
};
use rustls::ClientConfig;

use sel4_async_network_mbedtls::mbedtls::ssl::async_io::AsyncIo;

use super::{
    utils::{poll_read, poll_write, try_or_resize_and_retry, Buffer, WriteCursor},
    Error,
};

pub struct TcpConnector {
    config: Arc<ClientConfig>,
}

impl TcpConnector {
    pub fn connect<IO>(
        &self,
        domain: ServerName<'static>,
        stream: IO,
        // FIXME should not return an error but instead hoist it into a `Connect` variant
    ) -> Result<Connect<IO>, Error<IO::Error>>
    where
        IO: AsyncIo,
    {
        let conn = UnbufferedClientConnection::new(self.config.clone(), domain)?;

        Ok(Connect::new(conn, stream))
    }
}

impl From<Arc<ClientConfig>> for TcpConnector {
    fn from(config: Arc<ClientConfig>) -> Self {
        Self { config }
    }
}

pub struct Connect<IO> {
    inner: Option<ConnectInner<IO>>,
}

impl<IO> Connect<IO> {
    fn new(conn: UnbufferedClientConnection, io: IO) -> Self {
        Self {
            inner: Some(ConnectInner::new(conn, io)),
        }
    }
}

struct ConnectInner<IO> {
    conn: UnbufferedClientConnection,
    incoming: Buffer,
    io: IO,
    outgoing: Buffer,
}

impl<IO> ConnectInner<IO> {
    fn new(conn: UnbufferedClientConnection, io: IO) -> Self {
        Self {
            conn,
            incoming: Buffer::default(),
            io,
            outgoing: Buffer::default(),
        }
    }
}

impl<IO> Future for Connect<IO>
where
    IO: Unpin + AsyncIo,
{
    type Output = Result<TlsStream<IO>, Error<IO::Error>>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Self::Output> {
        let mut inner = self.inner.take().expect("polled after completion");

        let mut updates = Updates::default();
        let poll = loop {
            let action = inner.advance(&mut updates)?;

            match action {
                Action::Continue => continue,

                Action::Write => {
                    let mut outgoing = mem::take(&mut inner.outgoing);
                    let would_block = poll_write(&mut inner.io, &mut outgoing, cx)?;

                    updates.transmit_complete = outgoing.is_empty();
                    inner.outgoing = outgoing;

                    if would_block {
                        break Poll::Pending;
                    }
                }

                Action::Read => {
                    let mut incoming = mem::take(&mut inner.incoming);
                    let would_block = poll_read(&mut inner.io, &mut incoming, cx)?;

                    inner.incoming = incoming;

                    if would_block {
                        break Poll::Pending;
                    }
                }

                Action::Break => {
                    // XXX should we yield earlier when it's already possible to encrypt
                    // application data? that would reduce the number of round-trips
                    let ConnectInner {
                        conn,
                        incoming,
                        io,
                        outgoing,
                    } = inner;

                    return Poll::Ready(Ok(TlsStream {
                        conn,
                        incoming,
                        io,
                        outgoing,
                    }));
                }
            }
        };

        self.inner = Some(inner);

        poll
    }
}

#[derive(Default)]
struct Updates {
    transmit_complete: bool,
}

impl<IO: AsyncIo> ConnectInner<IO> {
    fn advance(&mut self, updates: &mut Updates) -> Result<Action, Error<IO::Error>> {
        log::trace!("incoming buffer has {}B of data", self.incoming.len());

        let UnbufferedStatus { discard, state } =
            self.conn.process_tls_records(self.incoming.filled_mut());

        log::trace!("state: {state:?}");
        let next = match state? {
            ConnectionState::EncodeTlsData(mut state) => {
                try_or_resize_and_retry(
                    |out_buffer| state.encode(out_buffer),
                    |e| {
                        if let EncodeError::InsufficientSize(is) = &e {
                            Ok(*is)
                        } else {
                            Err(e.into())
                        }
                    },
                    &mut self.outgoing,
                )?;

                Action::Continue
            }

            ConnectionState::TransmitTlsData(state) => {
                if updates.transmit_complete {
                    updates.transmit_complete = false;
                    state.done();
                    Action::Continue
                } else {
                    Action::Write
                }
            }

            ConnectionState::BlockedHandshake { .. } => Action::Read,

            ConnectionState::WriteTraffic(_) => Action::Break,

            state => unreachable!("{state:?}"), // due to type state
        };

        self.incoming.discard(discard);

        Ok(next)
    }
}

enum Action {
    Break,
    Continue,
    Read,
    Write,
}

pub struct TlsStream<IO> {
    conn: UnbufferedClientConnection,
    incoming: Buffer,
    io: IO,
    outgoing: Buffer,
}

impl<IO> AsyncIo for TlsStream<IO>
where
    IO: AsyncIo + Unpin,
{
    type Error = Error<IO::Error>;

    fn poll_send(
        &mut self,
        cx: &mut task::Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, Self::Error>> {
        let mut outgoing = mem::take(&mut self.outgoing);

        // no IO here; just in-memory writes
        match self.conn.process_tls_records(&mut []).state? {
            ConnectionState::WriteTraffic(mut state) => {
                try_or_resize_and_retry(
                    |out_buffer| state.encrypt(buf, out_buffer),
                    |e| {
                        if let EncryptError::InsufficientSize(is) = &e {
                            Ok(*is)
                        } else {
                            Err(e.into())
                        }
                    },
                    &mut outgoing,
                )?;
            }

            ConnectionState::Closed => {
                return Poll::Ready(Err(Error::ConnectionAborted));
            }

            state => unreachable!("{state:?}"),
        }

        // opportunistically try to write data into the socket
        // XXX should this be a loop?
        while !outgoing.is_empty() {
            let would_block = poll_write(&mut self.io, &mut outgoing, cx)?;
            if would_block {
                break;
            }
        }

        self.outgoing = outgoing;

        Poll::Ready(Ok(buf.len()))
    }

    fn poll_recv(
        &mut self,
        cx: &mut task::Context<'_>,
        buf: &mut [u8],
    ) -> Poll<Result<usize, Self::Error>> {
        let mut incoming = mem::take(&mut self.incoming);
        let mut cursor = WriteCursor::new(buf);

        log::debug!("XXX d loop enter");
        while !cursor.is_full() {
            log::trace!("incoming buffer has {}B of data", incoming.len());

            let UnbufferedStatus { mut discard, state } =
                self.conn.process_tls_records(incoming.filled_mut());

            match state? {
                ConnectionState::ReadTraffic(mut state) => {
                    log::debug!("XXX d ReadTraffic");
                    while let Some(res) = state.next_record() {
                        let AppDataRecord {
                            discard: new_discard,
                            payload,
                        } = res?;
                        // log::debug!("payload: {:x?}", payload);
                        log::debug!("payload: {}", core::str::from_utf8(payload).unwrap());
                        discard += new_discard;

                        let remainder = cursor.append(payload);

                        if !remainder.is_empty() {
                            // stash
                            todo!()
                        }
                    }
                }

                ConnectionState::WriteTraffic(_) => {
                    // panic!("XXX");
                    log::debug!("XXX d WriteTraffic");
                    let would_block = poll_read(&mut self.io, &mut incoming, cx)?;

                    if would_block {
                        self.incoming = incoming;
                        // ?
                        if cursor.used() != 0 {
                            break;
                        }
                        log::debug!("XXX d Pending");
                        return Poll::Pending;
                    }
                }

                ConnectionState::Closed => break,

                state => unreachable!("{state:?}"),
            }

            incoming.discard(discard);
        }

        log::debug!("XXX d Read");
        Poll::Ready(Ok(cursor.into_used()))
    }
}

impl<IO> TlsStream<IO>
where
    IO: AsyncIo + Unpin,
{
    pub async fn flush(&mut self) -> Result<(), Error<IO::Error>> {
        future::poll_fn(|cx| self.poll_flush(cx)).await
    }

    pub fn poll_flush(&mut self, cx: &mut task::Context<'_>) -> Poll<Result<(), Error<IO::Error>>> {
        let mut outgoing = mem::take(&mut self.outgoing);

        // write buffered TLS data into socket
        while !outgoing.is_empty() {
            let would_block = poll_write(&mut self.io, &mut outgoing, cx)?;

            if would_block {
                self.outgoing = outgoing;
                return Poll::Pending;
            }
        }

        self.outgoing = outgoing;

        // Pin::new(&mut self.io).poll_flush(cx)
        Poll::Ready(Ok(()))
    }

    #[allow(unused_mut)]
    pub fn poll_close(
        mut self: Pin<&mut Self>,
        _cx: &mut task::Context<'_>,
    ) -> Poll<Result<(), Error<IO::Error>>> {
        // XXX send out close_notify here?
        // Pin::new(&mut self.io).poll_close(cx)
        Poll::Ready(Ok(()))
    }
}
