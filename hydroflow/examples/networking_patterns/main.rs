use std::io::Error;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::pin::Pin;
use std::str::FromStr;
use std::time::Duration;

use bytes::{Bytes, BytesMut};
use futures::{Sink, Stream};
use hydroflow::hydroflow_syntax;
use hydroflow::util::cli::HydroCLI;
use hydroflow::util::unsync::mpsc::*;
use hydroflow::util::{
    adapters, bind_tcp_bytes, bind_tcp_lines, bind_udp_lines, connect_tcp_lines,
};
use hydroflow_cli_integration::{ConnectedBidi, ConnectedSink, ConnectedSource};
use serde::{Deserialize, Serialize};
use tokio::io::AsyncReadExt;

macro_rules! wait_1_sec {
    ($f1:expr) => {
        let _ = ::tokio::time::timeout(::std::time::Duration::from_secs(1), $f1).await;
    };
    ($f1:expr, $f2:expr) => {
        let _ = ::tokio::time::timeout(
            ::std::time::Duration::from_secs(1),
            ::futures::future::join($f1, $f2),
        )
        .await;
    };
}

/// This example creates and destroys multiple different servers and so in order to ensure that resources (like sockets) are cleaned up
/// each example is wrapped in it's own runtime.
fn run_in_new_runtime<F: std::future::Future>(f: F) -> F::Output {
    let rt = ::tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    rt.block_on(async { ::tokio::task::LocalSet::new().run_until(f).await })
}

#[derive(Serialize, Deserialize, Debug)]
struct MyRequest(usize);

#[derive(Serialize, Deserialize, Debug)]
struct MyResponse(usize);

const SERVER_ENDPOINT: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
const CLIENT_ENDPOINT: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8081);

pub fn main() {
    run_in_new_runtime(async {
        let (server_sink, server_stream, _) = bind_udp_lines(SERVER_ENDPOINT).await;
        let mut echo_server = hydroflow_syntax! {
            source_stream(server_stream)
                -> map(Result::unwrap)
                -> map(|(payload, src_addr)| (format!("server: {payload}"), src_addr))
                -> dest_sink(server_sink);
        };

        let (client_sink, client_stream, _) = bind_udp_lines(CLIENT_ENDPOINT).await;
        let mut echo_client = hydroflow_syntax! {
            source_iter([("hello".to_owned(), SERVER_ENDPOINT)])
                -> dest_sink(client_sink);

            source_stream(client_stream)
                -> map(Result::unwrap)
                -> for_each(|x| println!("client: {x:?}"));
        };

        wait_1_sec!(echo_server.run_async(), echo_client.run_async());
    });

    run_in_new_runtime(async {
        let (server_sink, server_stream) = bind_tcp_lines(SERVER_ENDPOINT).await;
        let mut echo_server = hydroflow_syntax! {
            source_stream(server_stream)
                -> map(Result::unwrap)
                -> map(|(payload, src_addr)| (format!("server: {payload}"), src_addr))
                -> dest_sink(server_sink);
        };

        let (client_sink, client_stream) = connect_tcp_lines();
        let mut echo_client = hydroflow_syntax! {
            source_iter([("hello".to_owned(), SERVER_ENDPOINT)])
                -> dest_sink(client_sink);

            source_stream(client_stream)
                -> map(Result::unwrap)
                -> for_each(|x| println!("client: {x:?}"));
        };

        wait_1_sec!(echo_server.run_async(), echo_client.run_async());
    });
}

// let mut ports = hydroflow::util::cli::init().await;

// let increment_requests = ports
//     .port("input")
//     .connect::<ConnectedBidi>()
//     .await
//     .into_source();

// let increment_requests: Pin<Box<dyn Sink<Bytes, Error = Error> + Send + Sync>> = ports
//     .port("input")
//     .connect::<ConnectedBidi>()
//     .await
//     .into_sink();

// source_port("input", HydroCLIBidiAdapter { hydro_cli: &mut ports })
//     -> map(|x| x.unwrap().freeze())
//     -> dest_port("output", HydroCLIBidiAdapter { hydro_cli: &mut ports });

// runtime_wrap2(async {
//     let mut echo_server = hydroflow_syntax! {
//         source(adapters::tcp_serde_responder("127.0.0.1:8080"))
//             -> map(Result::unwrap)
//             -> map(|(payload, resp)| (payload.unwrap(), resp))
//             -> for_each(|(req, resp): (String, Sender<String>)| resp.try_send(format!("received: {req:?}")).unwrap());
//     };

//     let mut echo_client = hydroflow_syntax! {
//         source_iter([("Hello".to_owned(), "127.0.0.1:8080".parse().unwrap())])
//             -> dest::<(String, SocketAddr)>(adapters::tcp_serde_route_sink());

//         source::<Result<String, _>>(adapters::tcp_serde("127.0.0.1:8081"))
//             -> map(Result::unwrap)
//             -> for_each(|x| println!("received: {x:?}"));
//     };

//     wait!(
//         Duration::from_secs(1),
//         echo_server.run_async(),
//         echo_client.run_async()
//     );
// });

// let mut server = hydroflow_syntax! {
//     source(adapters::tcp_serde_responder("127.0.0.1:8080"))
//         -> map(|x| x.unwrap())
//         -> for_each(|(req, resp): (String, Sender<String>)| resp.try_send(format!("received: {req:?}")).unwrap());
// };

// let mut client = hydroflow_syntax! {
//     source(adapter::std_io_lines())
//         -> map(|x| (x, "127.0.0.1:8080".parse().unwrap()))
//         -> dest_source(adapter::tcp_serde_req_route_resp())
//         -> map(|x| x.unwrap())
//         -> for_each(|x| println!("response from server: {x:?}"));
// };

// futures::join!(server.run_async(), client.run_async());

// df.run_async().await.unwrap();

// run_in_new_runtime(async {
//     #[derive(Default)]
//     struct Router<T> {
//         bindings: std::collections::HashMap<&'static str, Sender<T>>,
//     }

//     impl<T> Router<T> {
//         fn bind(&mut self, s: &str) -> Receiver<T> {
//             let (tx, rx) = hydroflow::util::unsync_channel(None);
//             self.bindings.insert(s, tx);
//             rx
//         }

//         fn connect(&mut self, s: &str) -> Sender<T> {
//             self.bindings.get(s).unwrap().clone()
//         }

//         fn route(&mut self) -> Routed<T> {

//         }
//     }

//     struct Routed<T> {

//     }

//     impl<T> Sink<(T, &'static str)> for Router<T> {
//         type Error = TrySendError<Option<T>>;

//         fn poll_ready(
//             self: Pin<&mut Self>,
//             cx: &mut std::task::Context<'_>,
//         ) -> std::task::Poll<Result<(), Self::Error>> {
//             todo!()
//         }

//         fn start_send(
//             self: Pin<&mut Self>,
//             (item, addr): (T, &'static str),
//         ) -> Result<(), Self::Error> {
//             let sender = std::pin::pin!(self.bindings.get_mut(addr).unwrap());
//             sender.start_send(item)
//         }

//         fn poll_flush(
//             self: Pin<&mut Self>,
//             cx: &mut std::task::Context<'_>,
//         ) -> std::task::Poll<Result<(), Self::Error>> {
//             todo!()
//         }

//         fn poll_close(
//             self: Pin<&mut Self>,
//             cx: &mut std::task::Context<'_>,
//         ) -> std::task::Poll<Result<(), Self::Error>> {
//             todo!()
//         }
//     }

//     let mut router = Router::default();

//     let server_stream = router.bind("inproc://S1");
//     let mut echo_server = hydroflow_syntax! {
//             source_stream(server_stream)
//                 -> map(Result::unwrap)
//                 -> map(|(payload, src_addr)| (format!("server: {payload}"), src_addr))
//                 -> dest_sink(server_sink);
//     };

//     let client_stream = router.bind("inproc://S2");
//     let mut echo_client = hydroflow_syntax! {
//             source_iter([("hello".to_owned(), "inproc://S1")])
//                 -> dest_sink(client_sink);

//             source_stream(client_stream)
//                 -> map(Result::unwrap)
//                 -> for_each(|x| println!("client: {x:?}"));
//     };

//     wait_1_sec!(echo_server.run_async(), echo_client.run_async());
// });

// tcp_request_response_static_endpoints();
// tcp_request_response_dynamic_client();

// Static addresses, everyone knows where everyone is.
// Server can't support multiple clients.
// Client can't correlate requests and responses (can use application level ID tags for this + idempotency = works well).

// Now CLIENT_ADDR does not appear anywhere because the server knows how to respond on the same connection that the request came in on
