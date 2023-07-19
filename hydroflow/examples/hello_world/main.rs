use hydroflow::hydroflow_syntax;

pub fn main() {
    const NEIGHBORS: [&str; 3] = ["a", "b", "c"];

    fn merge(x: &mut usize, y: usize) {
        *x += y;
    }

    let mut df = hydroflow_syntax! {
        src = union() -> tee();

        source_iter([("a", 1), ("b", 2), ("a", 4)]) -> src;
        source_iter([("a", 8), ("b", 16)]) -> defer_tick() -> src;

        src
            -> persist()
            -> reduce_keyed(merge)
            // -> inspect(|x| println!("LHS: {x:?}"))
            -> [0]J;

        src
            -> enumerate::<'static>()
            -> flat_map(|(req_id, (from, _data))| NEIGHBORS.into_iter().map(move |to| (from, to, req_id)))
            -> filter_map(|(from, to, req_id)| if to != from { Some((to, req_id)) } else { None })
            -> flat_map(|(to, req_id)| NEIGHBORS.into_iter().map(move |node_id| (node_id, (req_id, to))))
            -> filter(|(node_id, (_, to))| node_id != to)
            // -> inspect(|x| println!("RHS: {x:?}"))
            -> [1]J;

        J = join_multiset()
            // -> inspect(|x| println!("J: {x:?}"))
            -> map(|(_node_id, (data, (req_id, to)))| ((req_id, to), data))
            -> reduce_keyed(merge)
            -> sort_by_key(|((req_id, _to), _data)| req_id)
            -> for_each(|((req_id, to), data)| println!("req_id: {req_id}, To: {to}, Data: {data}"));
    };
    df.run_tick();
    println!("<Tick Boundary>");
    df.run_tick();
}

#[test]
fn test() {
    main();
}

//     let current_node_id = 1; // therefore 'local state' is just 1
//     let neighbors = [1, 2, 3];
//     [(2, f([1, 3])), (3, f([1, 2]))];
//     [(2, 17), (3, 119)]; // This is the end goal result. The evaluated version of above.

//     send list = neighbors WHERE id != node_id;

//     let to_send_list = filter(neighbors, |id| id != incoming_node_id); // if it's coming from 3, then we get [1, 2]

//     (3, f([1, 2]))
//     for node in to_send_list{
//         return (node, f(filter(neighbors, |id| id != node)))
//     }

//     SRC = source_iter([(src_node_id, data)]) -> tee();
//     SRC -> keyed_fold::<'static>(f) -> [0]J;
//     Q = SRC -> flat_map(|(src_node_id, data)| neighbors.filter(|x| x != src_node_id)) -> map(|x| (x, neighbors.filter(|y| x != y))) -> keyed_fold();

//     1 -> [2, 3] -> [(2, [1, 3]), (3, [1, 2])] -> [(2, f([1, 3]), (3, f([1, 2]))];
//     [1, 2, 3]

//     [1,2,3,4,5]
//     [(1, [1,2,3,4,5]), (2, [1,2,3,4,5]), (3, [1,2,3,4,5]), ]
//     neighbor_map map<node_id:neighbors>
//     neighbor_map |key, list| -> |key, list.where(|x| x != key)|

//     J = join();

//     Q -> [1]J;

//     J -> (src_node_id, (data, ??)) -> fold::<'tick>(fold_just_the_data) -> map(|x| (node, x));

//     SELECT SUM(data) FROM neighbors WHERE neighbor_id != node_id GROUP BY node_id

//     neighbors_list = [1,2,3,4];

//     neighbors_list = (|x| -> |(1,x)|)

// // TODOS:
//     // 1. represent this loop as a join.
//     // 2. initialize the neighbor list when we launch.
//     // 3. modify hydro deploy to use the new mux setup.

//    f(local_data, neighbors) // bad right
//     curr_neighbors = flatten(filter(neighbors, |id| id != current_node_id));
//     // want the list of distinct nodes with each one removed.

//     let to_send_list = cross_join(neighbors, neighbors); // [(1, 1), (1, 2), (1, 3), (2, 1), (2, 2), (2, 3), (3, 1), (3, 2), (3, 3)]
//     let to_send_list = filter(to_send_list, |(a, b)| a == this_node_id); //

//     #[allow(clippy::type_complexity)]
//     let df = hydroflow_syntax! {
//         from_neighbor = source_stream(from_neighbor)
//             -> map(|(node_id, x)| deserialize_from_bytes::<Vec<(u64, TimestampedValue<i32>)>>(x.unwrap()).unwrap())
//             -> fold::<'static>(
//                 //1. update current node_id state via max for incoming x
//                 // to_send_list = neighbor_list EXCEPT node_id
//                 // for each this_node_id in to_send_list
//                     // send(this_node_id, SUM(neighbor_list EXCEPT this_node_id))
//                 (HashMap::<u64, TimestampedValue<i32>>::new(), HashSet::new(), 0),
//                 |(prev, modified_tweets, prev_tick): &mut (HashMap<_, TimestampedValue<i32>>, HashSet<_>, _), req: Vec<(u64, TimestampedValue<i32>)>| {
//                     if *prev_tick != context.current_tick() {
//                         modified_tweets.clear();
//                     }

//                     for (k, v) in req {
//                         let updated = if let Some(e) = prev.get_mut(&k) {
//                             e.merge_from(v)
//                         } else {
//                             prev.insert(k, v);
//                             true
//                         };

//                         if updated {
//                             modified_tweets.insert(k);
//                         }
//                     }

//                     *prev_tick = context.current_tick();
//                 }
//             )
//             -> filter(|(_, _, tick)| *tick == context.current_tick())
//             -> flat_map(|(state, modified_tweets, _)| modified_tweets.iter().map(|t| (*t, *state.get(t).unwrap())).collect::<Vec<_>>())
//             -> tee();

//         from_local = source_stream(increment_requests)
//             -> map(|x| deserialize_from_bytes::<IncrementRequest>(&x.unwrap()).unwrap())
//             -> map(|x| (x.tweet_id, x.likes))
//             -> fold::<'static>(
//                 (HashMap::<u64, TimestampedValue<i32>>::new(), HashSet::new(), 0),
//                 |(prev, modified_tweets, prev_tick): &mut (HashMap<_, TimestampedValue<i32>>, HashSet<_>, usize), req: UpdateType| {
//                     if *prev_tick != context.current_tick() {
//                         modified_tweets.clear();
//                     }

//                     prev.entry(req.0).or_default().update(|v| v + req.1);
//                     modified_tweets.insert(req.0);
//                     *prev_tick = context.current_tick();
//                 }
//             )
//             -> filter(|(_, _, tick)| *tick == context.current_tick())
//             -> flat_map(|(state, modified_tweets, _)| modified_tweets.iter().map(|t| (*t, *state.get(t).unwrap())).collect::<Vec<_>>())
//             -> tee();

//         from_parent -> map(|v| (0, v)) -> to_right;
//         from_left -> map(|v| (1, v)) -> to_right;
//         from_local -> map(|v| (2, v)) -> to_right;

//         to_parent = union();

//         //TODO later optimization of materializing aggs of subsets e.g. in a tree so that you don't recompute from scratch for each neighbor

//         from_right -> map(|v| (0, v)) -> to_parent;
//         from_left -> map(|v| (1, v)) -> to_parent;
//         from_local -> map(|v| (2, v)) -> to_parent;

//         to_parent
//             -> fold::<'static>(
//                 (vec![HashMap::<u64, TimestampedValue<i32>>::new(); 3], HashMap::<u64, TimestampedValue<i32>>::new(), HashSet::new(), 0),
//                 |(each_source, acc_source, modified_tweets, prev_tick): &mut (Vec<HashMap<u64, TimestampedValue<i32>>>, HashMap<_, TimestampedValue<i32>>, HashSet<_>, usize), (source_i, (key, v)): (usize, _)| {
//                     if *prev_tick != context.current_tick() {
//                         modified_tweets.clear();
//                     }

//                     let updated = each_source[source_i].entry(key).or_default().merge_from(v);

//                     if updated {
//                         acc_source.entry(key).or_default().update(|_| each_source.iter().map(|s| s.get(&key).map(|t| t.value).unwrap_or_default()).sum());
//                         modified_tweets.insert(key);
//                     }

//                     *prev_tick = context.current_tick();
//                 }
//             )
//             -> filter(|(_, _, _, tick)| *tick == context.current_tick())
//             -> map(|(_, state, modified_tweets, _)| modified_tweets.iter().map(|t| (*t, *state.get(t).unwrap())).collect())
//             -> map(serialize_to_bytes::<Vec<(u64, TimestampedValue<i32>)>>)
//             -> dest_sink(to_parent);

//         to_query = union();

//         from_parent -> map(|v| (0, v)) -> to_query;
//         from_left -> map(|v| (1, v)) -> to_query;
//         from_right -> map(|v| (2, v)) -> to_query;
//         from_local -> map(|v| (3, v)) -> to_query;

//         to_query
//             -> fold::<'static>(
//                 (vec![HashMap::<u64, TimestampedValue<i32>>::new(); 4], HashMap::<u64, TimestampedValue<i32>>::new(), HashSet::new(), 0),
//                 |(each_source, acc_source, modified_tweets, prev_tick): &mut (Vec<HashMap<u64, TimestampedValue<i32>>>, HashMap<_, TimestampedValue<i32>>, HashSet<_>, usize), (source_i, (key, v)): (usize, _)| {
//                     if *prev_tick != context.current_tick() {
//                         modified_tweets.clear();
//                     }

//                     let updated = each_source[source_i].entry(key).or_default().merge_from(v);

//                     if updated {
//                         acc_source.entry(key).or_default().update(|_| each_source.iter().map(|s| s.get(&key).map(|t| t.value).unwrap_or_default()).sum());
//                         modified_tweets.insert(key);
//                     }

//                     *prev_tick = context.current_tick();
//                 }
//             )
//             -> filter(|(_, _, _, tick)| *tick == context.current_tick())
//             -> flat_map(|(_, state, modified_tweets, _)| modified_tweets.iter().map(|t| (*t, state.get(t).unwrap().value)).collect::<Vec<_>>())
//             -> map(serialize_to_bytes::<(u64, i32)>)
//             -> dest_sink(query_responses);
//     };

//     // initial memory
//     #[cfg(target_os = "linux")]
//     {
//         let x = procinfo::pid::stat_self().unwrap();
//         let bytes = x.rss * 1024 * 4;
//         println!("memory,{}", bytes);
//     }

//     let f1_handle = tokio::spawn(f1);
//     hydroflow::util::cli::launch_flow(df).await;
//     f1_handle.abort();
// }
