use std::collections::HashSet;
use std::error::Error;
use std::fmt::Display;

use hydroflow::hydroflow_syntax;
use hydroflow::util::unsync_channel;
use lattices::set_union::{SetUnionHashSet, SetUnionSingletonSet};
use lattices::{Bottom, Top};

#[derive(Debug)]
enum Request {
    Put {
        key: &'static str,
        val: &'static str,
    },
    Get {
        key: &'static str,
    },
    Delete {
        key: &'static str,
    },
}

#[derive(Debug)]
enum Response {
    Put {
        key: &'static str,
    },
    Get {
        key: &'static str,
        val: Option<HashSet<&'static str>>,
    },
    Delete {
        key: &'static str,
    },
}

#[derive(Debug, Default)]
struct KvsError;
impl Display for KvsError {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        todo!()
    }
}
impl Error for KvsError {}

pub type MyLastWriteWins = Top<Bottom<SetUnionHashSet<&'static str>>>;

#[hydroflow::main]
pub async fn main() {
    let (tx_ingress, rx_ingress) = unsync_channel::<Request>(None);
    let (tx_egress, mut rx_egress) = unsync_channel::<Response>(None);

    tokio::task::spawn_local(async move {
        let mut df = hydroflow_syntax! {
            incoming_requests = source_stream(rx_ingress)
                -> demux(|req, var_args!(gets, puts, deletes)| {
                    match req {
                        Request::Put { key, val } => puts.give((key, val)),
                        Request::Get { key } => gets.give(key),
                        Request::Delete { key } => deletes.give(key),
                    }
                });

            puts = incoming_requests[puts]
                -> map(|(key, val)| (key, Top::new(Bottom::new(SetUnionSingletonSet::new_from(val)))))
                -> tee();
            puts
                -> map(|(key, _)| Response::Put { key })
                -> next_tick() // Response can be sent out before put is actually in the lhs of the join.
                -> responses;
            puts
                -> lhs_join_input;

            deletes = incoming_requests[deletes]
                -> map(|key| (key, Top::default()))
                -> tee();
            deletes
                -> map(|(key, _)| Response::Delete { key })
                -> next_tick() // Response can be sent out before put is actually in the lhs of the join.
                -> responses;
            deletes
                -> lhs_join_input;

            lhs_join_input = union();
            lhs_join_input
                -> [0]join_state;

            incoming_requests[gets]
                -> enumerate()
                -> map(|(clock, key)| (key, SetUnionSingletonSet::new_from((clock, key))))
                -> [1]join_state;

            join_state = lattice_join::<'static, 'tick, MyLastWriteWins, SetUnionHashSet<(u128, &'static str)>>();

            join_state
                -> flat_map(|(key, (lhs, rhs))| rhs.0.into_iter().map(move |_| Response::Get {key, val: lhs.0.clone().map(|x| x.0.map(|x| x.0)).flatten() })) // TODO: this is implcitly copying the data so something is wrong here.
                -> responses;

            responses = union()
                -> dest_sink(tx_egress);
        };

        println!("{}", df.meta_graph().unwrap().to_mermaid());

        df.run_async().await
    });

    macro_rules! kvs_send {
        ($obj:expr) => {{
            tx_ingress.send($obj).await.unwrap();
        }};
    }

    macro_rules! kvs_put {
        ($key:expr, $val:expr) => {{
            kvs_send!(Request::Put {
                key: $key,
                val: $val,
            });
            match rx_egress.recv().await.unwrap() {
                Response::Put { key } => {
                    assert_eq!(key, $key);
                    key
                }
                _ => panic!(),
            }
        }};
    }

    macro_rules! kvs_get {
        ($key:expr) => {{
            kvs_send!(Request::Get { key: $key });
            match rx_egress.recv().await.unwrap() {
                Response::Get { key, val } => {
                    assert_eq!(key, $key);
                    val
                }
                _ => panic!(),
            }
        }};
    }

    macro_rules! kvs_delete {
        ($key:expr) => {{
            kvs_send!(Request::Delete { key: $key });
            match rx_egress.recv().await.unwrap() {
                Response::Delete { key } => assert_eq!(key, $key),
                _ => panic!(),
            }
        }};
    }

    kvs_put!("test", "v1");
    assert_eq!(kvs_get!("test"), Some(HashSet::from_iter(["v1"])));
    kvs_put!("test", "v2");
    assert_eq!(kvs_get!("test"), Some(HashSet::from_iter(["v1", "v2"])));
    kvs_delete!("test");
    assert_eq!(kvs_get!("test"), None);
}
