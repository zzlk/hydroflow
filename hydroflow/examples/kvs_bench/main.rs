#![feature(core_intrinsics)]

mod buffer_pool;
mod protocol;
mod server;

use crate::server::run_server;

use clap::command;
use clap::Parser;
use clap::Subcommand;

use crate::buffer_pool::BufferPool;
use crate::protocol::BytesWrapper;
use crate::protocol::KvsRequest;
use crate::protocol::NodeId;
use bincode::options;
use bytes::BufMut;
use bytes::Bytes;
use bytes::BytesMut;
use futures::Stream;
use rand::Rng;
use rand::SeedableRng;
use serde::de::DeserializeSeed;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;
use tokio::sync::mpsc::Sender;
use tokio::sync::mpsc::UnboundedSender;
use tokio_stream::StreamExt;

#[derive(Debug, Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Bench {
        #[clap(long, default_value_t = 1)]
        threads: usize,

        #[clap(long, default_value_t = 4.0)]
        dist: f64,

        #[clap(long, default_value_t = 2)]
        warmup: u64,

        #[clap(long, default_value_t = 10)]
        duration: u64,

        #[clap(long, default_value_t = false)]
        report: bool,

        #[clap(long, default_value_t = false)]
        print_mermaid: bool,
    },
}

pub struct Topology<RX>
where
    RX: Stream<Item = (usize, Bytes)>,
{
    pub lookup: Vec<usize>,
    pub tx: Vec<UnboundedSender<Bytes>>,
    pub rx: Vec<RX>,
}

impl<RX> Default for Topology<RX>
where
    RX: Stream<Item = (usize, Bytes)> + StreamExt + Unpin,
{
    fn default() -> Self {
        Self {
            lookup: Default::default(),
            tx: Default::default(),
            rx: Default::default(),
        }
    }
}

fn main() {
    match Cli::parse().command {
        Commands::Bench {
            threads,
            dist,
            warmup,
            duration,
            report,
            mut print_mermaid,
        } => {
            let mut throughputs = Vec::new();
            let mut nodes: HashMap<NodeId, Topology<_>> = HashMap::default();
            let mut client_tx: HashMap<NodeId, Sender<Vec<Bytes>>> = HashMap::default();

            for n1 in 0..threads {
                throughputs.push(Arc::new(AtomicUsize::new(0)));

                nodes.entry(n1).or_default();

                for n2 in 0..threads {
                    if n2 == n1 {
                        continue;
                    }

                    let (tx, rx) = hydroflow::util::unbounded_channel::<Bytes>();

                    {
                        let entry = nodes.entry(n1).or_default();

                        entry.lookup.push(n2);
                        entry.tx.push(tx);
                    }

                    {
                        nodes
                            .entry(n2)
                            .or_default()
                            .rx
                            .push(rx.map(move |x| (n2, x)));
                    }
                }
            }

            for (node_id, topology) in nodes {
                let (tx, rx) = hydroflow::util::bounded_channel::<Vec<Bytes>>(20);

                client_tx.insert(node_id, tx);

                run_server(
                    node_id,
                    rx,
                    topology,
                    dist,
                    throughputs[node_id].clone(),
                    print_mermaid,
                );

                print_mermaid = false; // Only want one node to print the mermaid since it is the same for all of them.
            }

            for i in 0..1 {
                let client_tx = client_tx.clone();
                std::thread::spawn(move || {
                    let rt = tokio::runtime::Builder::new_current_thread()
                        .enable_all()
                        .build()
                        .unwrap();

                    rt.block_on(async {
                        let mut rng = rand::rngs::SmallRng::from_entropy();
                        let dist = rand_distr::Zipf::new(1_000_000, dist).unwrap();

                        let mut pre_gen_index = 0;
                        let pre_gen_random_numbers: Vec<u64> =
                            (0..(128 * 1024)).map(|_| rng.sample(dist) as u64).collect();

                        let mut buffer = BytesMut::with_capacity(1024);
                        buffer.resize(1024, 227);
                        let buffer = buffer.freeze();

                        let req = KvsRequest::Put {
                            key: rng.sample(dist) as u64,
                            value: BytesWrapper(buffer.clone()),
                        };

                        let mut serialized = BytesMut::with_capacity(1024 + 128);
                        bincode2::encode_into_std_write(
                            &bincode2::serde::BytesCompat(req),
                            &mut (&mut serialized).writer(),
                            bincode2::config::standard(),
                        )
                        .unwrap();

                        let serialized = serialized.freeze();

                        loop {
                            for (node_id, tx) in client_tx.iter() {
                                tx.send(vec![serialized.clone(); 1024]).await.unwrap();
                            }
                        }
                    });
                });
            }

            let get_reset_throughputs = || {
                let mut sum = 0;
                for x in throughputs.iter() {
                    sum += x.swap(0, Ordering::SeqCst);
                }

                sum
            };

            let mut total_writes_so_far = 0;

            std::thread::sleep(Duration::from_secs(warmup));

            get_reset_throughputs();
            let start_time = Instant::now();
            let mut time_last_interval = start_time;

            loop {
                if start_time.elapsed().as_secs_f64() >= duration as f64 {
                    break;
                }

                std::thread::sleep(Duration::from_secs(1));

                if report {
                    let writes_this_interval = get_reset_throughputs();
                    let puts =
                        writes_this_interval as f64 / time_last_interval.elapsed().as_secs_f64();
                    time_last_interval = Instant::now();
                    println!("{puts}");

                    total_writes_so_far += writes_this_interval;
                }
            }

            total_writes_so_far += get_reset_throughputs();
            let puts = total_writes_so_far as f64 / start_time.elapsed().as_secs_f64();

            println!("{puts}");
        }
    }
}

// {
//     use bincode::Options;
//     let mut buff = BytesMut::new();
//     buff.put_u8(7);
//     let serialization_options = options();
//     let req = KvsClientRequest::Put {
//         key: 5,
//         value: MySpecialBytesType(buff.freeze()),
//     };
//     let mut serialized =
//         BytesMut::with_capacity(serialization_options.serialized_size(&req).unwrap() as usize);

//     bincode2::serde::encode_into_std_write(
//         &req,
//         &mut (&mut serialized).writer(),
//         bincode2::config::standard(),
//     )
//     .unwrap();

//     let serialized = serialized.freeze();

//     let deserialized: KvsClientRequest =
//         bincode2::serde::decode_from_bytes(serialized.clone(), bincode2::config::standard())
//             .unwrap()
//             .0;

//     if let KvsClientRequest::Put { key, value } = deserialized {
//         let slice = serialized.slice_ref(&value.0);
//         assert_eq!(slice[0], 7);
//         dbg!(serialized, value.0);
//     } else {
//         panic!();
//     }
// }
