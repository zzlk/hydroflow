use hydroflow::{assert_graphvis_snapshots, hydroflow_syntax, util::collect_ready};
use multiplatform_test::multiplatform_test;

#[multiplatform_test]
pub fn test_persist_replay_join() {
    let (persist_input_send, persist_input) = hydroflow::util::unbounded_channel::<u32>();
    let (other_input_send, other_input) = hydroflow::util::unbounded_channel::<u32>();
    let (result_send, mut result_recv) = hydroflow::util::unbounded_channel::<(u32, u32)>();

    let mut hf = hydroflow_syntax! {
        source_stream(persist_input)
            -> persist()
            -> fold::<'tick>(0, |a, b| (a + b))
            -> [0] product_node;

        source_stream(other_input) -> [1] product_node;

        product_node = cross_join::<'tick, 'tick>() -> for_each(|x| result_send.send(x).unwrap());
    };
    // assert_graphvis_snapshots!(hf);

    persist_input_send.send(1).unwrap();
    other_input_send.send(2).unwrap();
    hf.run_tick();
    assert_eq!(&[(1, 2)], &*collect_ready::<Vec<_>, _>(&mut result_recv));

    persist_input_send.send(2).unwrap();
    other_input_send.send(2).unwrap();
    hf.run_tick();
    assert_eq!(&[(3, 2)], &*collect_ready::<Vec<_>, _>(&mut result_recv));

    other_input_send.send(3).unwrap();
    hf.run_tick();
    hf.run_tick();
    hf.run_tick();
    assert_eq!(&[(3, 3)], &*collect_ready::<Vec<_>, _>(&mut result_recv));
}

fn main() {}
