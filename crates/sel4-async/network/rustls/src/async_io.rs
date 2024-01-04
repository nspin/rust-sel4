use core::pin::Pin;
use core::task::Poll;
use core::future;
use core::{mem, task};

use alloc::{sync::Arc, vec::Vec};

use futures::Future;
use rustls::client::UnbufferedClientConnection;
use rustls::pki_types::ServerName;
use rustls::unbuffered::{
    AppDataRecord, ConnectionState, EncodeError, EncryptError, InsufficientSizeError,
    UnbufferedStatus,
};
use rustls::ClientConfig;
use rustls::Error as TlsError;

use sel4_async_network_mbedtls::mbedtls::ssl::async_io::AsyncIo;

#[derive(Debug)]
pub enum Error<E> {
    TransitError(E),
    ConnectionAborted,
    TlsError(TlsError),
    EncodeError(EncodeError),
    EncryptError(EncryptError),
}

impl<E> From<TlsError> for Error<E> {
    fn from(err: TlsError) -> Self {
        Self::TlsError(err)
    }
}

impl<E> From<EncodeError> for Error<E> {
    fn from(err: EncodeError) -> Self {
        Self::EncodeError(err)
    }
}

impl<E> From<EncryptError> for Error<E> {
    fn from(err: EncryptError) -> Self {
        Self::EncryptError(err)
    }
}

// // //

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

/// returns `true` if the operation would block
fn poll_read<IO>(
    io: &mut IO,
    incoming: &mut Buffer,
    cx: &mut task::Context,
) -> Result<bool, Error<IO::Error>>
where
    IO: AsyncIo + Unpin,
{
    if incoming.unfilled().is_empty() {
        // XXX should this be user configurable?
        // incoming.reserve(1024);
        incoming.reserve(1024 * 256);
    }

    let would_block = match Pin::new(io).poll_recv(cx, incoming.unfilled()) {
        Poll::Ready(res) => {
            let read = res.map_err(Error::TransitError)?;
            log::trace!("read {read}B from socket");
            incoming.advance(read);
            false
        }

        Poll::Pending => true,
    };

    Ok(would_block)
}

/// returns `true` if the operation would block
fn poll_write<IO>(
    io: &mut IO,
    outgoing: &mut Buffer,
    cx: &mut task::Context,
) -> Result<bool, Error<IO::Error>>
where
    IO: AsyncIo + Unpin,
{
    let pending = match Pin::new(io).poll_send(cx, outgoing.filled()) {
        Poll::Ready(res) => {
            let written = res.map_err(Error::TransitError)?;
            log::trace!("wrote {written}B into socket");
            outgoing.discard(written);
            log::trace!("{}B remain in the outgoing buffer", outgoing.len());
            false
        }

        Poll::Pending => true,
    };
    Ok(pending)
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
                        return Poll::Pending;
                    }
                }

                ConnectionState::Closed => break,

                state => unreachable!("{state:?}"),
            }

            incoming.discard(discard);
        }

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

    pub fn poll_close(mut self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Result<(), Error<IO::Error>>> {
        // XXX send out close_notify here?
        // Pin::new(&mut self.io).poll_close(cx)
        Poll::Ready(Ok(()))
    }
}

struct WriteCursor<'a> {
    buf: &'a mut [u8],
    used: usize,
}

impl<'a> WriteCursor<'a> {
    fn new(buf: &'a mut [u8]) -> Self {
        Self { buf, used: 0 }
    }

    // HACK
    fn used(&self) -> usize {
        self.used
    }

    fn into_used(self) -> usize {
        self.used
    }

    fn append<'b>(&mut self, data: &'b [u8]) -> &'b [u8] {
        let len = self.remaining_capacity().min(data.len());

        self.unfilled()[..len].copy_from_slice(&data[..len]);
        self.used += len;

        data.split_at(len).1
    }

    fn unfilled(&mut self) -> &mut [u8] {
        &mut self.buf[self.used..]
    }

    fn is_full(&self) -> bool {
        self.remaining_capacity() == 0
    }

    fn remaining_capacity(&self) -> usize {
        self.buf.len() - self.used
    }
}

#[derive(Default)]
struct Buffer {
    inner: Vec<u8>,
    used: usize,
}

impl Buffer {
    fn advance(&mut self, num_bytes: usize) {
        self.used += num_bytes;
    }

    fn discard(&mut self, num_bytes: usize) {
        if num_bytes == 0 {
            return;
        }

        debug_assert!(num_bytes <= self.used);

        self.inner.copy_within(num_bytes..self.used, 0);
        self.used -= num_bytes;

        log::trace!("discarded {num_bytes}B");
    }

    fn reserve(&mut self, additional_bytes: usize) {
        let new_len = self.used + additional_bytes;
        self.inner.resize(new_len, 0);
    }

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn len(&self) -> usize {
        self.filled().len()
    }

    fn filled(&self) -> &[u8] {
        &self.inner[..self.used]
    }

    fn filled_mut(&mut self) -> &mut [u8] {
        &mut self.inner[..self.used]
    }

    fn unfilled(&mut self) -> &mut [u8] {
        &mut self.inner[self.used..]
    }

    fn capacity(&self) -> usize {
        self.inner.len()
    }
}

fn try_or_resize_and_retry<E1, E2>(
    mut f: impl FnMut(&mut [u8]) -> Result<usize, E1>,
    map_err: impl FnOnce(E1) -> Result<InsufficientSizeError, Error<E2>>,
    outgoing: &mut Buffer,
) -> Result<usize, Error<E2>>
where
    Error<E2>: From<E1>,
{
    let written = match f(outgoing.unfilled()) {
        Ok(written) => written,

        Err(e) => {
            let InsufficientSizeError { required_size } = map_err(e)?;
            outgoing.reserve(required_size);
            log::trace!("resized `outgoing_tls` buffer to {}B", outgoing.capacity());

            f(outgoing.unfilled())?
        }
    };

    outgoing.advance(written);

    Ok(written)
}
