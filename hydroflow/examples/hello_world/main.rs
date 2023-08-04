use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use hydroflow::hydroflow_syntax;

macro_rules! assert_contains_each_by_tick {
    ($results:expr, $tick:expr, &[]) => {{
        assert_eq!($results.borrow().get(&$tick), None);
    }};
    ($results:expr, $tick:expr, $input:expr) => {{
        for v in $input {
            assert!(
                $results.borrow()[&$tick].contains(v),
                "did not contain: {:?} in {:?}",
                v,
                $results.borrow()[&$tick]
            );
        }
    }};
}

pub fn main() {
    let results = Rc::new(RefCell::new(HashMap::<usize, Vec<_>>::new()));
    let results_inner = Rc::clone(&results);

    let mut df = hydroflow_syntax! {
        source_iter([(7, 1), (7, 2)])
            -> [0]my_join;

        source_iter([(7, 0)]) -> unioner;
        source_iter([(7, 1)]) -> defer_tick() -> unioner;
        source_iter([(7, 2)]) -> defer_tick() -> defer_tick() -> unioner;
        unioner = union()
            -> [1]my_join;

        my_join = join::<'static, 'static>()
            -> for_each(|x| results_inner.borrow_mut().entry(context.current_tick()).or_default().push(x));
    };
    // assert_graphvis_snapshots!(df);
    df.run_available();

    #[rustfmt::skip]
    {
        assert_contains_each_by_tick!(results, 0, &[(7, (1, 0)), (7, (2, 0))]);
        assert_contains_each_by_tick!(results, 1, &[(7, (1, 0)), (7, (2, 0)), (7, (1, 1)), (7, (2, 1))]);
        assert_contains_each_by_tick!(results, 2, &[(7, (1, 0)), (7, (2, 0)), (7, (1, 1)), (7, (2, 1)), (7, (1, 2)), (7, (2, 2))]);
        assert_contains_each_by_tick!(results, 3, &[]);
    };
}

#[test]
fn test() {
    main();
}
