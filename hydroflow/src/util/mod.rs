//! Helper utilities for the Hydroflow surface syntax.

mod udp;
pub use udp::*;
mod tcp;
pub use tcp::*;

use std::net::SocketAddr;
use std::task::{Context, Poll};

use bincode;
use futures::Stream;
use serde::{Deserialize, Serialize};

pub fn unbounded_channel<T>() -> (
    tokio::sync::mpsc::UnboundedSender<T>,
    tokio_stream::wrappers::UnboundedReceiverStream<T>,
) {
    let (send, recv) = tokio::sync::mpsc::unbounded_channel();
    let recv = tokio_stream::wrappers::UnboundedReceiverStream::new(recv);
    (send, recv)
}

/// Collects the immediately available items from the `Stream` into a `FromIterator` collection.
///
/// This consumes the stream, use [`futures::StreamExt::by_ref()`] (or just `&mut ...`) if you want
/// to retain ownership of your stream.
pub fn collect_ready<C, S>(stream: S) -> C
where
    C: FromIterator<S::Item>,
    S: Stream,
{
    let mut stream = Box::pin(stream);
    std::iter::from_fn(|| {
        match stream
            .as_mut()
            .poll_next(&mut Context::from_waker(futures::task::noop_waker_ref()))
        {
            Poll::Ready(opt) => opt,
            Poll::Pending => None,
        }
    })
    .collect()
}

pub fn serialize_to_bytes<T>(msg: T) -> bytes::Bytes
where
    T: Serialize + for<'a> Deserialize<'a> + Clone,
{
    bytes::Bytes::from(bincode::serialize(&msg).unwrap())
}

pub fn deserialize_from_bytes<T>(msg: bytes::BytesMut) -> T
where
    T: Serialize + for<'a> Deserialize<'a> + Clone,
{
    bincode::deserialize(&msg).unwrap()
}

pub fn ipv4_resolve(addr: &str) -> Result<SocketAddr, std::io::Error> {
    use std::net::ToSocketAddrs;
    let mut addrs = addr.to_socket_addrs()?;
    let result = addrs.find(|addr| addr.is_ipv4());
    match result {
        Some(addr) => Ok(addr),
        None => Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Unable to resolve IPv4 address",
        )),
    }
}

pub async fn bind_udp_bytes(addr: SocketAddr) -> (UdpSink, UdpStream, SocketAddr) {
    let socket = tokio::net::UdpSocket::bind(addr).await.unwrap();
    udp_bytes(socket)
}

pub async fn bind_udp_lines(addr: SocketAddr) -> (UdpLinesSink, UdpLinesStream, SocketAddr) {
    let socket = tokio::net::UdpSocket::bind(addr).await.unwrap();
    udp_lines(socket)
}

// from addendum in
// https://stackoverflow.com/questions/56105305/how-to-sort-a-vec-of-structs-by-a-string-field
pub fn sort_unstable_by_key_hrtb<T, F, K>(slice: &mut [T], f: F)
where
    F: for<'a> Fn(&'a T) -> &'a K,
    K: Ord,
{
    slice.sort_unstable_by(|a, b| f(a).cmp(f(b)))
}
