//! TODO:

use std::collections::hash_map::Entry::*;
use std::collections::HashMap;
use std::fmt::Debug;
use std::net::SocketAddr;
use std::pin::pin;

use bytes::{Bytes, BytesMut};
use futures::{SinkExt, StreamExt};
use serde::de::DeserializeOwned;
use serde::Serialize;
use tokio::io::AsyncReadExt;
use tokio::net::TcpListener;
use tokio::task::spawn_local;
use tokio_util::codec::{Decoder, FramedRead, FramedWrite};

use super::unsync::mpsc::{Receiver, Sender};
use super::{deserialize_from_bytes, serialize_to_bytes, unsync_channel};

/// TODO:
pub async fn tcp(endpoint: &str) -> Receiver<Bytes> {
    let listener = TcpListener::bind(endpoint).await.unwrap();

    let (tx, rx) = unsync_channel(None);

    spawn_local(async move {
        loop {
            let mut socket = match listener.accept().await {
                Ok((socket, _src)) => socket,
                Err(_) => {
                    continue;
                }
            };

            let tx = tx.clone();
            spawn_local(async move {
                loop {
                    let mut buf = BytesMut::new();
                    if let Err(_) = socket.read_buf(&mut buf).await {
                        return;
                    }

                    tx.send(buf.freeze()).await.unwrap();
                }
            });
        }
    });

    rx
}

/// TODO:
pub async fn tcp_codec<Codec: 'static + Decoder + Clone>(
    endpoint: &str,
    codec: Codec,
) -> Receiver<(Codec::Item, SocketAddr)> {
    let listener = TcpListener::bind(endpoint).await.unwrap();

    let (tx, rx) = unsync_channel(None);

    spawn_local(async move {
        let codec = codec.clone();
        loop {
            let (socket, src) = match listener.accept().await {
                Ok((socket, src)) => (socket, src),
                Err(_) => {
                    continue;
                }
            };

            let mut tx = tx.clone();

            let codec = codec.clone();
            spawn_local(async move {
                let fr = FramedRead::new(socket, codec);
                let mut mapped =
                    pin!(fr.filter_map(|x| async { x.map(|x| Ok((x, src.clone()))).ok() }));

                let _ = tx.send_all(&mut mapped).await;
            });
        }
    });

    rx
}

/// TODO:
pub async fn tcp_serde<T: 'static + DeserializeOwned>(
    endpoint: SocketAddr,
) -> Receiver<Result<T, Box<bincode::ErrorKind>>> {
    let listener = TcpListener::bind(endpoint).await.unwrap();

    let (tx, rx) = unsync_channel(None);

    spawn_local(async move {
        loop {
            let socket = if let Ok((socket, _)) = listener.accept().await {
                socket
            } else {
                continue;
            };

            let mut tx = tx.clone();

            spawn_local(async move {
                let fr = FramedRead::new(socket, tokio_util::codec::LengthDelimitedCodec::new());
                let mapped = fr.filter_map(|x| async move {
                    x.map(|x| Ok(deserialize_from_bytes::<T>(&x))).ok()
                });

                let _ = tx.send_all(&mut pin!(mapped)).await;
            });
        }
    });

    rx
}

/// TODO:
// pub async fn udp_serde<T: 'static + DeserializeOwned>(
//     endpoint: SocketAddr,
// ) -> Result<Receiver<Result<T, Box<bincode::ErrorKind>>>, std::io::Error> {
//     let socket = tokio::net::UdpSocket::bind(endpoint).await?;

//     let (tx, rx) = unsync_channel(None);

//     spawn_local(async move {
//         let fr = FramedRead::new(socket, tokio_util::codec::LengthDelimitedCodec::new());
//         let mapped =
//             fr.filter_map(|x| async move { x.map(|x| Ok(deserialize_from_bytes::<T>(&x))).ok() });

//         let _ = tx.send_all(&mut pin!(mapped)).await;
//     });

//     Ok(rx)
// }

/// TODO:
pub async fn tcp_serde_route_sink<T: 'static + Serialize>() -> Sender<(T, SocketAddr)> {
    let (tx, mut rx) = unsync_channel(None);

    spawn_local(async move {
        let mut streams = HashMap::new();

        loop {
            while let Some((payload, addr)) = rx.recv().await {
                let bytes = serialize_to_bytes(payload);

                let stream = match streams.entry(addr) {
                    Occupied(entry) => entry.into_mut(),
                    Vacant(entry) => {
                        let socket = tokio::net::TcpSocket::new_v4().unwrap();
                        entry.insert(FramedWrite::new(
                            socket.connect(addr).await.unwrap(),
                            tokio_util::codec::LengthDelimitedCodec::new(),
                        ))
                    }
                };

                stream.send(bytes).await.unwrap();
            }
        }
    });

    tx
}

/// TODO:
pub async fn tcp_serde_responder<Req: 'static + DeserializeOwned, Resp: 'static + Serialize>(
    endpoint: SocketAddr,
) -> Receiver<Result<(Req, Sender<Resp>), Box<bincode::ErrorKind>>> {
    let listener = TcpListener::bind(endpoint).await.unwrap();

    let (tx, rx) = unsync_channel(None);

    spawn_local(async move {
        loop {
            let socket = if let Ok((socket, _)) = listener.accept().await {
                socket
            } else {
                continue;
            };

            let mut tx = tx.clone();

            spawn_local(async move {
                let (recv, send) = socket.into_split();
                let mut send =
                    FramedWrite::new(send, tokio_util::codec::LengthDelimitedCodec::new());
                let recv = FramedRead::new(recv, tokio_util::codec::LengthDelimitedCodec::new());

                let (tx2, mut rx2) = unsync_channel(None);

                spawn_local(async move {
                    while let Some(resp) = rx2.recv().await {
                        send.send(serialize_to_bytes(resp)).await.unwrap();
                    }
                });

                let mapped = recv.filter_map(|x| {
                    let tx2 = tx2.clone();

                    async move {
                        x.map(|x| Ok(deserialize_from_bytes::<Req>(&x).map(|x| (x, tx2.clone()))))
                            .ok()
                    }
                });

                let _ = tx.send_all(&mut pin!(mapped)).await;
            });
        }
    });

    rx
}

/// TODO:
pub async fn tcp_serde_req_route_resp<
    Req: 'static + Serialize,
    Resp: 'static + DeserializeOwned,
>() -> (
    Sender<(Req, SocketAddr)>,
    Receiver<Result<Resp, Box<bincode::ErrorKind>>>,
) {
    let (from_upstream_tx, mut from_upstream_rx) = unsync_channel(None);
    let (to_downstream_tx, to_downstream_rx) = unsync_channel(None);

    spawn_local(async move {
        let mut streams = HashMap::new();

        loop {
            while let Some((payload, addr)) = from_upstream_rx.recv().await {
                let bytes = serialize_to_bytes(payload);

                let stream = match streams.entry(addr) {
                    Occupied(entry) => entry.into_mut(),
                    Vacant(entry) => {
                        let socket = tokio::net::TcpSocket::new_v4().unwrap();
                        let stream = socket.connect(addr).await.unwrap();

                        let (recv, send) = stream.into_split();

                        let send =
                            FramedWrite::new(send, tokio_util::codec::LengthDelimitedCodec::new());
                        let recv =
                            FramedRead::new(recv, tokio_util::codec::LengthDelimitedCodec::new());

                        let mut to_downstream_tx = to_downstream_tx.clone();
                        spawn_local(async move {
                            let mapped = recv.filter_map(|x| async move {
                                x.map(|x| Ok(deserialize_from_bytes::<Resp>(&x))).ok()
                            });

                            let _ = to_downstream_tx.send_all(&mut pin!(mapped)).await;
                        });

                        entry.insert(send)
                    }
                };

                stream.send(bytes).await.unwrap();
            }
        }
    });

    (from_upstream_tx, to_downstream_rx)
}
