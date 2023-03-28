mod broadcast_receiver_stream;
mod buffer_pool;
mod server;
mod util;

use crate::server::run_server;
use clap::command;
use clap::Parser;
use clap::Subcommand;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crdts::LWWReg;
use hydroflow::lang::lattice::LatticeRepr;
use hydroflow::lang::lattice::Merge;
use serde_big_array::BigArray;
use std::marker::PhantomData;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::sync::Arc;

// impl<
//         K: 'static,
//         SelfTag,
//         DeltaTag,
//         SelfLr: LatticeRepr<Lattice = L>,
//         DeltaLr: LatticeRepr<Lattice = L>,
//         L: Lattice,
//     > Merge<MapUnionRepr<DeltaTag, K, DeltaLr>> for MapUnionRepr<SelfTag, K, SelfLr>
// where
//     SelfTag: MapTag<K, SelfLr::Repr>,
//     DeltaTag: MapTag<K, DeltaLr::Repr>,
//     MapUnionRepr<SelfTag, K, SelfLr>: LatticeRepr<Lattice = MapUnion<K, L>>,
//     MapUnionRepr<DeltaTag, K, DeltaLr>: LatticeRepr<Lattice = MapUnion<K, L>>,
//     <MapUnionRepr<SelfTag, K, SelfLr> as LatticeRepr>::Repr:
//         Extend<(K, SelfLr::Repr)> + Collection<K, SelfLr::Repr>,
//     <MapUnionRepr<DeltaTag, K, DeltaLr> as LatticeRepr>::Repr:
//         IntoIterator<Item = (K, DeltaLr::Repr)>,
//     SelfLr: Merge<DeltaLr>,
//     DeltaLr: Convert<SelfLr>,
// {

struct Hashjoin<LR: LatticeRepr + Merge<DeltaLR>, DeltaLR: LatticeRepr> {
    val: LR::Repr,
    _x: PhantomData<*const DeltaLR>,
}

impl<LR: LatticeRepr + Merge<DeltaLR>, DeltaLR: LatticeRepr> Hashjoin<LR, DeltaLR> {
    pub fn meme(&mut self, v: DeltaLR::Repr) {
        <LR as Merge<DeltaLR>>::merge(&mut self.val, v);
    }
}

#[derive(PartialEq, Eq, Clone, Serialize, Deserialize, Debug, Ord, PartialOrd)]
pub struct ValueType {
    #[serde(with = "BigArray")]
    pub data: [u8; 1024],
}

impl Default for ValueType {
    fn default() -> Self {
        Self { data: [0; 1024] }
    }
}

type MyRegType = LWWReg<ValueType, u128>;

#[derive(PartialEq, Eq, Clone, Serialize, Deserialize, Debug)]
pub enum KVSRequest {
    Put { key: u64, value: ValueType },
    Get { key: u64 },
    Gossip { key: u64, reg: MyRegType },
}

#[derive(PartialEq, Eq, Clone, Serialize, Deserialize, Debug)]
pub enum KVSResponse {
    PutResponse { key: u64 },
    GetResponse { key: u64, reg: MyRegType },
}

#[derive(Debug, Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    #[command(arg_required_else_help = true)]
    Bench {
        #[clap(long)]
        threads: usize,

        #[clap(long)]
        dist: f64,
    },
}

fn main() {
    let ctx = tmq::Context::new();

    let throughput = Arc::new(AtomicUsize::new(0));

    match Cli::parse().command {
        Commands::Bench { threads, dist } => {
            let topology: Vec<_> = (0..threads).map(|x| x).collect();

            for addr in topology.iter() {
                run_server(
                    *addr,
                    topology.clone(),
                    dist,
                    ctx.clone(),
                    throughput.clone(),
                );
            }
        }
    }

    std::thread::sleep(Duration::from_millis(2000));

    throughput.store(0, Ordering::SeqCst);
    let start_time = std::time::Instant::now();

    std::thread::sleep(Duration::from_millis(12000));
    let puts = throughput.load(Ordering::SeqCst) as f64 / start_time.elapsed().as_secs_f64();
    println!("{puts}");

    // let start_time = std::time::Instant::now();
    // let mut time_since_last_report = std::time::Instant::now();
    // loop {
    //     if time_since_last_report.elapsed() >= Duration::from_secs(1) {
    //         time_since_last_report = Instant::now();
    //         println!("puts/s: {}", throughput.load(Ordering::SeqCst));
    //         throughput.store(0, Ordering::SeqCst);
    //     }

    //     std::thread::sleep(Duration::from_millis(32));
    // }
}
