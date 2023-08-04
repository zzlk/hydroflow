#![feature(prelude_import)]
#[prelude_import]
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use hydroflow::hydroflow_syntax;
pub fn main() {
    let results = Rc::new(RefCell::new(HashMap::<usize, Vec<_>>::new()));
    let results_inner = Rc::clone(&results);
    let mut df = {
        #[allow(unused_qualifications)]
        {
            use ::hydroflow::{var_args, var_expr};
            let mut df = ::hydroflow::scheduled::graph::Hydroflow::new();
            df.__assign_meta_graph(
                "{\"nodes\":[{\"value\":null,\"version\":0},{\"value\":{\"Operator\":\"source_iter([(7, 1), (7, 2)])\"},\"version\":1},{\"value\":{\"Operator\":\"source_iter([(7, 0)])\"},\"version\":1},{\"value\":{\"Operator\":\"source_iter([(7, 1)])\"},\"version\":1},{\"value\":{\"Operator\":\"defer_tick()\"},\"version\":1},{\"value\":{\"Operator\":\"source_iter([(7, 2)])\"},\"version\":1},{\"value\":{\"Operator\":\"defer_tick()\"},\"version\":1},{\"value\":{\"Operator\":\"defer_tick()\"},\"version\":1},{\"value\":{\"Operator\":\"union()\"},\"version\":1},{\"value\":{\"Operator\":\"join :: < 'static, 'static > ()\"},\"version\":1},{\"value\":{\"Operator\":\"for_each(| x |\\nresults_inner.borrow_mut().entry(context.current_tick()).or_default().push(x))\"},\"version\":1},{\"value\":{\"Handoff\":{}},\"version\":1},{\"value\":{\"Handoff\":{}},\"version\":1},{\"value\":{\"Handoff\":{}},\"version\":1},{\"value\":{\"Operator\":\"identity()\"},\"version\":1},{\"value\":{\"Handoff\":{}},\"version\":1},{\"value\":{\"Operator\":\"identity()\"},\"version\":1},{\"value\":{\"Handoff\":{}},\"version\":1},{\"value\":{\"Operator\":\"identity()\"},\"version\":1},{\"value\":{\"Handoff\":{}},\"version\":1}],\"graph\":[{\"value\":null,\"version\":0},{\"value\":[{\"idx\":1,\"version\":1},{\"idx\":9,\"version\":1}],\"version\":1},{\"value\":[{\"idx\":2,\"version\":1},{\"idx\":8,\"version\":1}],\"version\":1},{\"value\":[{\"idx\":4,\"version\":1},{\"idx\":8,\"version\":1}],\"version\":1},{\"value\":[{\"idx\":3,\"version\":1},{\"idx\":11,\"version\":1}],\"version\":3},{\"value\":[{\"idx\":7,\"version\":1},{\"idx\":8,\"version\":1}],\"version\":1},{\"value\":[{\"idx\":6,\"version\":1},{\"idx\":12,\"version\":1}],\"version\":3},{\"value\":[{\"idx\":5,\"version\":1},{\"idx\":13,\"version\":1}],\"version\":3},{\"value\":[{\"idx\":8,\"version\":1},{\"idx\":9,\"version\":1}],\"version\":1},{\"value\":[{\"idx\":9,\"version\":1},{\"idx\":10,\"version\":1}],\"version\":1},{\"value\":[{\"idx\":11,\"version\":1},{\"idx\":14,\"version\":1}],\"version\":3},{\"value\":[{\"idx\":12,\"version\":1},{\"idx\":16,\"version\":1}],\"version\":3},{\"value\":[{\"idx\":13,\"version\":1},{\"idx\":18,\"version\":1}],\"version\":3},{\"value\":[{\"idx\":14,\"version\":1},{\"idx\":15,\"version\":1}],\"version\":3},{\"value\":[{\"idx\":15,\"version\":1},{\"idx\":4,\"version\":1}],\"version\":1},{\"value\":[{\"idx\":16,\"version\":1},{\"idx\":17,\"version\":1}],\"version\":3},{\"value\":[{\"idx\":17,\"version\":1},{\"idx\":7,\"version\":1}],\"version\":1},{\"value\":[{\"idx\":18,\"version\":1},{\"idx\":19,\"version\":1}],\"version\":3},{\"value\":[{\"idx\":19,\"version\":1},{\"idx\":6,\"version\":1}],\"version\":1}],\"ports\":[{\"value\":null,\"version\":0},{\"value\":[\"Elided\",{\"Int\":\"0\"}],\"version\":1},{\"value\":[\"Elided\",\"Elided\"],\"version\":1},{\"value\":[\"Elided\",\"Elided\"],\"version\":1},{\"value\":[\"Elided\",\"Elided\"],\"version\":3},{\"value\":[\"Elided\",\"Elided\"],\"version\":1},{\"value\":[\"Elided\",\"Elided\"],\"version\":3},{\"value\":[\"Elided\",\"Elided\"],\"version\":3},{\"value\":[\"Elided\",{\"Int\":\"1\"}],\"version\":1},{\"value\":[\"Elided\",\"Elided\"],\"version\":1},{\"value\":[\"Elided\",\"Elided\"],\"version\":3},{\"value\":[\"Elided\",\"Elided\"],\"version\":3},{\"value\":[\"Elided\",\"Elided\"],\"version\":3},{\"value\":[\"Elided\",\"Elided\"],\"version\":3},{\"value\":[\"Elided\",\"Elided\"],\"version\":1},{\"value\":[\"Elided\",\"Elided\"],\"version\":3},{\"value\":[\"Elided\",\"Elided\"],\"version\":1},{\"value\":[\"Elided\",\"Elided\"],\"version\":3},{\"value\":[\"Elided\",\"Elided\"],\"version\":1}],\"node_subgraph\":[{\"value\":null,\"version\":0},{\"value\":{\"idx\":4,\"version\":1},\"version\":1},{\"value\":{\"idx\":4,\"version\":1},\"version\":1},{\"value\":{\"idx\":1,\"version\":1},\"version\":1},{\"value\":{\"idx\":4,\"version\":1},\"version\":1},{\"value\":{\"idx\":2,\"version\":1},\"version\":1},{\"value\":{\"idx\":3,\"version\":1},\"version\":1},{\"value\":{\"idx\":4,\"version\":1},\"version\":1},{\"value\":{\"idx\":4,\"version\":1},\"version\":1},{\"value\":{\"idx\":4,\"version\":1},\"version\":1},{\"value\":{\"idx\":4,\"version\":1},\"version\":1},{\"value\":null,\"version\":0},{\"value\":null,\"version\":0},{\"value\":null,\"version\":0},{\"value\":{\"idx\":5,\"version\":1},\"version\":1},{\"value\":null,\"version\":0},{\"value\":{\"idx\":6,\"version\":1},\"version\":1},{\"value\":null,\"version\":0},{\"value\":{\"idx\":7,\"version\":1},\"version\":1}],\"subgraph_nodes\":[{\"value\":null,\"version\":0},{\"value\":[{\"idx\":3,\"version\":1}],\"version\":1},{\"value\":[{\"idx\":5,\"version\":1}],\"version\":1},{\"value\":[{\"idx\":6,\"version\":1}],\"version\":1},{\"value\":[{\"idx\":1,\"version\":1},{\"idx\":2,\"version\":1},{\"idx\":4,\"version\":1},{\"idx\":7,\"version\":1},{\"idx\":8,\"version\":1},{\"idx\":9,\"version\":1},{\"idx\":10,\"version\":1}],\"version\":1},{\"value\":[{\"idx\":14,\"version\":1}],\"version\":1},{\"value\":[{\"idx\":16,\"version\":1}],\"version\":1},{\"value\":[{\"idx\":18,\"version\":1}],\"version\":1}],\"subgraph_stratum\":[{\"value\":null,\"version\":0},{\"value\":0,\"version\":1},{\"value\":0,\"version\":1},{\"value\":0,\"version\":1},{\"value\":0,\"version\":1},{\"value\":1,\"version\":1},{\"value\":1,\"version\":1},{\"value\":1,\"version\":1}],\"node_varnames\":[{\"value\":null,\"version\":0},{\"value\":null,\"version\":0},{\"value\":null,\"version\":0},{\"value\":null,\"version\":0},{\"value\":null,\"version\":0},{\"value\":null,\"version\":0},{\"value\":null,\"version\":0},{\"value\":null,\"version\":0},{\"value\":\"unioner\",\"version\":1},{\"value\":\"my_join\",\"version\":1},{\"value\":\"my_join\",\"version\":1}]}",
            );
            df.__assign_diagnostics("[]");
            let (hoff_11v1_send, hoff_11v1_recv) = df
                .make_edge::<_, ::hydroflow::scheduled::handoff::VecHandoff<_>>(
                    "handoff GraphNodeId(11v1)",
                );
            let (hoff_12v1_send, hoff_12v1_recv) = df
                .make_edge::<_, ::hydroflow::scheduled::handoff::VecHandoff<_>>(
                    "handoff GraphNodeId(12v1)",
                );
            let (hoff_13v1_send, hoff_13v1_recv) = df
                .make_edge::<_, ::hydroflow::scheduled::handoff::VecHandoff<_>>(
                    "handoff GraphNodeId(13v1)",
                );
            let (hoff_15v1_send, hoff_15v1_recv) = df
                .make_edge::<_, ::hydroflow::scheduled::handoff::VecHandoff<_>>(
                    "handoff GraphNodeId(15v1)",
                );
            let (hoff_17v1_send, hoff_17v1_recv) = df
                .make_edge::<_, ::hydroflow::scheduled::handoff::VecHandoff<_>>(
                    "handoff GraphNodeId(17v1)",
                );
            let (hoff_19v1_send, hoff_19v1_recv) = df
                .make_edge::<_, ::hydroflow::scheduled::handoff::VecHandoff<_>>(
                    "handoff GraphNodeId(19v1)",
                );
            let mut sg_1v1_node_3v1_iter = {
                #[inline(always)]
                fn check_iter<IntoIter: ::std::iter::IntoIterator<Item = Item>, Item>(
                    into_iter: IntoIter,
                ) -> impl ::std::iter::Iterator<Item = Item> {
                    ::std::iter::IntoIterator::into_iter(into_iter)
                }
                check_iter([(7, 1)])
            };
            df.add_subgraph_stratified(
                "Subgraph GraphSubgraphId(1v1)",
                0,
                (),
                (hoff_11v1_send, ()),
                move |context, (), (hoff_11v1_send, ())| {
                    let hoff_11v1_send = ::hydroflow::pusherator::for_each::ForEach::new(|
                        v|
                    {
                        hoff_11v1_send.give(Some(v));
                    });
                    let op_3v1 = sg_1v1_node_3v1_iter.by_ref();
                    let op_3v1 = {
                        #[allow(non_snake_case)]
                        #[inline(always)]
                        pub fn op_3v1__source_iter__loc_unknown_start_0_0_end_0_0<
                            Item,
                            Input: ::std::iter::Iterator<Item = Item>,
                        >(input: Input) -> impl ::std::iter::Iterator<Item = Item> {
                            struct Pull<
                                Item,
                                Input: ::std::iter::Iterator<Item = Item>,
                            > {
                                inner: Input,
                            }
                            impl<
                                Item,
                                Input: ::std::iter::Iterator<Item = Item>,
                            > Iterator for Pull<Item, Input> {
                                type Item = Item;
                                #[inline(always)]
                                fn next(&mut self) -> Option<Self::Item> {
                                    self.inner.next()
                                }
                                #[inline(always)]
                                fn size_hint(&self) -> (usize, Option<usize>) {
                                    self.inner.size_hint()
                                }
                            }
                            Pull { inner: input }
                        }
                        op_3v1__source_iter__loc_unknown_start_0_0_end_0_0(op_3v1)
                    };
                    #[inline(always)]
                    fn check_pivot_run<
                        Pull: ::std::iter::Iterator<Item = Item>,
                        Push: ::hydroflow::pusherator::Pusherator<Item = Item>,
                        Item,
                    >(pull: Pull, push: Push) {
                        ::hydroflow::pusherator::pivot::Pivot::new(pull, push).run();
                    }
                    check_pivot_run(op_3v1, hoff_11v1_send);
                },
            );
            let mut sg_2v1_node_5v1_iter = {
                #[inline(always)]
                fn check_iter<IntoIter: ::std::iter::IntoIterator<Item = Item>, Item>(
                    into_iter: IntoIter,
                ) -> impl ::std::iter::Iterator<Item = Item> {
                    ::std::iter::IntoIterator::into_iter(into_iter)
                }
                check_iter([(7, 2)])
            };
            df.add_subgraph_stratified(
                "Subgraph GraphSubgraphId(2v1)",
                0,
                (),
                (hoff_13v1_send, ()),
                move |context, (), (hoff_13v1_send, ())| {
                    let hoff_13v1_send = ::hydroflow::pusherator::for_each::ForEach::new(|
                        v|
                    {
                        hoff_13v1_send.give(Some(v));
                    });
                    let op_5v1 = sg_2v1_node_5v1_iter.by_ref();
                    let op_5v1 = {
                        #[allow(non_snake_case)]
                        #[inline(always)]
                        pub fn op_5v1__source_iter__loc_unknown_start_0_0_end_0_0<
                            Item,
                            Input: ::std::iter::Iterator<Item = Item>,
                        >(input: Input) -> impl ::std::iter::Iterator<Item = Item> {
                            struct Pull<
                                Item,
                                Input: ::std::iter::Iterator<Item = Item>,
                            > {
                                inner: Input,
                            }
                            impl<
                                Item,
                                Input: ::std::iter::Iterator<Item = Item>,
                            > Iterator for Pull<Item, Input> {
                                type Item = Item;
                                #[inline(always)]
                                fn next(&mut self) -> Option<Self::Item> {
                                    self.inner.next()
                                }
                                #[inline(always)]
                                fn size_hint(&self) -> (usize, Option<usize>) {
                                    self.inner.size_hint()
                                }
                            }
                            Pull { inner: input }
                        }
                        op_5v1__source_iter__loc_unknown_start_0_0_end_0_0(op_5v1)
                    };
                    #[inline(always)]
                    fn check_pivot_run<
                        Pull: ::std::iter::Iterator<Item = Item>,
                        Push: ::hydroflow::pusherator::Pusherator<Item = Item>,
                        Item,
                    >(pull: Pull, push: Push) {
                        ::hydroflow::pusherator::pivot::Pivot::new(pull, push).run();
                    }
                    check_pivot_run(op_5v1, hoff_13v1_send);
                },
            );
            let mut sg_4v1_node_1v1_iter = {
                #[inline(always)]
                fn check_iter<IntoIter: ::std::iter::IntoIterator<Item = Item>, Item>(
                    into_iter: IntoIter,
                ) -> impl ::std::iter::Iterator<Item = Item> {
                    ::std::iter::IntoIterator::into_iter(into_iter)
                }
                check_iter([(7, 1), (7, 2)])
            };
            let mut sg_4v1_node_2v1_iter = {
                #[inline(always)]
                fn check_iter<IntoIter: ::std::iter::IntoIterator<Item = Item>, Item>(
                    into_iter: IntoIter,
                ) -> impl ::std::iter::Iterator<Item = Item> {
                    ::std::iter::IntoIterator::into_iter(into_iter)
                }
                check_iter([(7, 0)])
            };
            let sg_4v1_node_9v1_joindata_lhs = df.add_state(std::cell::RefCell::new(
                ::hydroflow::compiled::pull::HalfSetJoinState::default(),
            ));
            let sg_4v1_node_9v1_joindata_rhs = df.add_state(std::cell::RefCell::new(
                ::hydroflow::compiled::pull::HalfSetJoinState::default(),
            ));
            let sg_4v1_node_9v1_persisttick = df.add_state(std::cell::RefCell::new(0usize));
            df.add_subgraph_stratified(
                "Subgraph GraphSubgraphId(4v1)",
                0,
                (hoff_15v1_recv, (hoff_17v1_recv, ())),
                (),
                move |context, (hoff_15v1_recv, (hoff_17v1_recv, ())), ()| {
                    let mut hoff_15v1_recv = hoff_15v1_recv.borrow_mut_swap();
                    let hoff_15v1_recv = hoff_15v1_recv.drain(..);
                    let mut hoff_17v1_recv = hoff_17v1_recv.borrow_mut_swap();
                    let hoff_17v1_recv = hoff_17v1_recv.drain(..);
                    let op_1v1 = sg_4v1_node_1v1_iter.by_ref();
                    let op_1v1 = {
                        #[allow(non_snake_case)]
                        #[inline(always)]
                        pub fn op_1v1__source_iter__loc_unknown_start_0_0_end_0_0<
                            Item,
                            Input: ::std::iter::Iterator<Item = Item>,
                        >(input: Input) -> impl ::std::iter::Iterator<Item = Item> {
                            struct Pull<
                                Item,
                                Input: ::std::iter::Iterator<Item = Item>,
                            > {
                                inner: Input,
                            }
                            impl<
                                Item,
                                Input: ::std::iter::Iterator<Item = Item>,
                            > Iterator for Pull<Item, Input> {
                                type Item = Item;
                                #[inline(always)]
                                fn next(&mut self) -> Option<Self::Item> {
                                    self.inner.next()
                                }
                                #[inline(always)]
                                fn size_hint(&self) -> (usize, Option<usize>) {
                                    self.inner.size_hint()
                                }
                            }
                            Pull { inner: input }
                        }
                        op_1v1__source_iter__loc_unknown_start_0_0_end_0_0(op_1v1)
                    };
                    let op_2v1 = sg_4v1_node_2v1_iter.by_ref();
                    let op_2v1 = {
                        #[allow(non_snake_case)]
                        #[inline(always)]
                        pub fn op_2v1__source_iter__loc_unknown_start_0_0_end_0_0<
                            Item,
                            Input: ::std::iter::Iterator<Item = Item>,
                        >(input: Input) -> impl ::std::iter::Iterator<Item = Item> {
                            struct Pull<
                                Item,
                                Input: ::std::iter::Iterator<Item = Item>,
                            > {
                                inner: Input,
                            }
                            impl<
                                Item,
                                Input: ::std::iter::Iterator<Item = Item>,
                            > Iterator for Pull<Item, Input> {
                                type Item = Item;
                                #[inline(always)]
                                fn next(&mut self) -> Option<Self::Item> {
                                    self.inner.next()
                                }
                                #[inline(always)]
                                fn size_hint(&self) -> (usize, Option<usize>) {
                                    self.inner.size_hint()
                                }
                            }
                            Pull { inner: input }
                        }
                        op_2v1__source_iter__loc_unknown_start_0_0_end_0_0(op_2v1)
                    };
                    let op_4v1 = {
                        fn check_input<Iter: ::std::iter::Iterator<Item = Item>, Item>(
                            iter: Iter,
                        ) -> impl ::std::iter::Iterator<Item = Item> {
                            iter
                        }
                        check_input::<_, _>(hoff_15v1_recv)
                    };
                    let op_4v1 = {
                        #[allow(non_snake_case)]
                        #[inline(always)]
                        pub fn op_4v1__defer_tick__loc_unknown_start_0_0_end_0_0<
                            Item,
                            Input: ::std::iter::Iterator<Item = Item>,
                        >(input: Input) -> impl ::std::iter::Iterator<Item = Item> {
                            struct Pull<
                                Item,
                                Input: ::std::iter::Iterator<Item = Item>,
                            > {
                                inner: Input,
                            }
                            impl<
                                Item,
                                Input: ::std::iter::Iterator<Item = Item>,
                            > Iterator for Pull<Item, Input> {
                                type Item = Item;
                                #[inline(always)]
                                fn next(&mut self) -> Option<Self::Item> {
                                    self.inner.next()
                                }
                                #[inline(always)]
                                fn size_hint(&self) -> (usize, Option<usize>) {
                                    self.inner.size_hint()
                                }
                            }
                            Pull { inner: input }
                        }
                        op_4v1__defer_tick__loc_unknown_start_0_0_end_0_0(op_4v1)
                    };
                    let op_7v1 = {
                        fn check_input<Iter: ::std::iter::Iterator<Item = Item>, Item>(
                            iter: Iter,
                        ) -> impl ::std::iter::Iterator<Item = Item> {
                            iter
                        }
                        check_input::<_, _>(hoff_17v1_recv)
                    };
                    let op_7v1 = {
                        #[allow(non_snake_case)]
                        #[inline(always)]
                        pub fn op_7v1__defer_tick__loc_unknown_start_0_0_end_0_0<
                            Item,
                            Input: ::std::iter::Iterator<Item = Item>,
                        >(input: Input) -> impl ::std::iter::Iterator<Item = Item> {
                            struct Pull<
                                Item,
                                Input: ::std::iter::Iterator<Item = Item>,
                            > {
                                inner: Input,
                            }
                            impl<
                                Item,
                                Input: ::std::iter::Iterator<Item = Item>,
                            > Iterator for Pull<Item, Input> {
                                type Item = Item;
                                #[inline(always)]
                                fn next(&mut self) -> Option<Self::Item> {
                                    self.inner.next()
                                }
                                #[inline(always)]
                                fn size_hint(&self) -> (usize, Option<usize>) {
                                    self.inner.size_hint()
                                }
                            }
                            Pull { inner: input }
                        }
                        op_7v1__defer_tick__loc_unknown_start_0_0_end_0_0(op_7v1)
                    };
                    let op_8v1 = {
                        #[allow(unused)]
                        #[inline(always)]
                        fn check_inputs<
                            A: ::std::iter::Iterator<Item = Item>,
                            B: ::std::iter::Iterator<Item = Item>,
                            Item,
                        >(a: A, b: B) -> impl ::std::iter::Iterator<Item = Item> {
                            a.chain(b)
                        }
                        check_inputs(check_inputs(op_2v1, op_4v1), op_7v1)
                    };
                    let op_8v1 = {
                        #[allow(non_snake_case)]
                        #[inline(always)]
                        pub fn op_8v1__union__loc_unknown_start_0_0_end_0_0<
                            Item,
                            Input: ::std::iter::Iterator<Item = Item>,
                        >(input: Input) -> impl ::std::iter::Iterator<Item = Item> {
                            struct Pull<
                                Item,
                                Input: ::std::iter::Iterator<Item = Item>,
                            > {
                                inner: Input,
                            }
                            impl<
                                Item,
                                Input: ::std::iter::Iterator<Item = Item>,
                            > Iterator for Pull<Item, Input> {
                                type Item = Item;
                                #[inline(always)]
                                fn next(&mut self) -> Option<Self::Item> {
                                    self.inner.next()
                                }
                                #[inline(always)]
                                fn size_hint(&self) -> (usize, Option<usize>) {
                                    self.inner.size_hint()
                                }
                            }
                            Pull { inner: input }
                        }
                        op_8v1__union__loc_unknown_start_0_0_end_0_0(op_8v1)
                    };
                    let mut sg_4v1_node_9v1_joindata_lhs_borrow = || {
                        context.state_ref(sg_4v1_node_9v1_joindata_lhs).borrow_mut()
                    };
                    let mut sg_4v1_node_9v1_joindata_rhs_borrow = || {
                        context.state_ref(sg_4v1_node_9v1_joindata_rhs).borrow_mut()
                    };
                    let mut sg_4v1_node_9v1_persisttick_borrow = context
                        .state_ref(sg_4v1_node_9v1_persisttick)
                        .borrow_mut();
                    let op_9v1 = {
                        #[inline(always)]
                        fn check_inputs<'a, K, I1, V1, I2, V2>(
                            mut lhs: I1,
                            mut rhs: I2,
                            lhs_state_fn: impl 'a + Fn(
                            ) -> ::std::cell::RefMut<
                                    'a,
                                    ::hydroflow::compiled::pull::HalfSetJoinState<K, V1, V2>,
                                >,
                            rhs_state_fn: impl 'a + Fn(
                            ) -> ::std::cell::RefMut<
                                    'a,
                                    ::hydroflow::compiled::pull::HalfSetJoinState<K, V2, V1>,
                                >,
                            is_new_tick: bool,
                        ) -> impl 'a + Iterator<Item = (K, (V1, V2))>
                        where
                            K: 'a + Eq + std::hash::Hash + Clone,
                            V1: 'a + Clone + ::std::cmp::Eq,
                            V2: 'a + Clone + ::std::cmp::Eq,
                            I1: 'a + Iterator<Item = (K, V1)>,
                            I2: 'a + Iterator<Item = (K, V2)>,
                        {
                            use ::hydroflow::compiled::pull::HalfJoinState;
                            let mut sg_4v1_node_9v1_joindata_lhs_borrow = (lhs_state_fn)();
                            let mut lhs_keys = if is_new_tick {
                                Some(::std::cell::RefMut::map(
                                    sg_4v1_node_9v1_joindata_lhs_borrow,
                                    |x| (&*x).iter(),
                                ))
                            } else {
                                None
                            };
                            ::std::iter::from_fn(move || {
                                loop {
                                    let mut sg_4v1_node_9v1_joindata_lhs_borrow = (lhs_state_fn)();
                                    let mut sg_4v1_node_9v1_joindata_rhs_borrow = (rhs_state_fn)();
                                    let mut lhs_state = &mut *sg_4v1_node_9v1_joindata_lhs_borrow;
                                    let mut rhs_state = &mut *sg_4v1_node_9v1_joindata_rhs_borrow;
                                    if let ::std::option::Option::Some((k, v1, v2))
                                        = rhs_state.pop_match()
                                    {
                                        return ::std::option::Option::Some((k, (v1, v2)));
                                    }
                                    if let ::std::option::Option::Some(x) = lhs_keys {
                                        while let ::std::option::Option::Some((k, v1)) = x.next() {
                                            if let ::std::option::Option::Some((k, v1, v2))
                                                = rhs_state.probe(&k, &v1)
                                            {
                                                return ::std::option::Option::Some((k, (v1, v2)));
                                            }
                                            continue;
                                        }
                                    }
                                    lhs_keys = ::std::option::Option::None;
                                    if let ::std::option::Option::Some((k, v2, v1))
                                        = lhs_state.pop_match()
                                    {
                                        return ::std::option::Option::Some((k, (v1, v2)));
                                    }
                                    if let ::std::option::Option::Some((k, v1)) = lhs.next() {
                                        if lhs_state.build(k.clone(), &v1) {
                                            if let ::std::option::Option::Some((k, v1, v2))
                                                = rhs_state.probe(&k, &v1)
                                            {
                                                return ::std::option::Option::Some((k, (v1, v2)));
                                            }
                                        }
                                        continue;
                                    }
                                    if let ::std::option::Option::Some((k, v2)) = rhs.next() {
                                        if rhs_state.build(k.clone(), &v2) {
                                            if let ::std::option::Option::Some((k, v2, v1))
                                                = lhs_state.probe(&k, &v2)
                                            {
                                                return ::std::option::Option::Some((k, (v1, v2)));
                                            }
                                        }
                                        continue;
                                    }
                                    return None;
                                }
                            })
                        }
                        {
                            let __is_new_tick = if *sg_4v1_node_9v1_persisttick_borrow
                                < context.current_tick()
                            {
                                *sg_4v1_node_9v1_persisttick_borrow = context
                                    .current_tick();
                                true
                            } else {
                                false
                            };
                            check_inputs(
                                op_1v1,
                                op_8v1,
                                sg_4v1_node_9v1_joindata_lhs_borrow,
                                sg_4v1_node_9v1_joindata_rhs_borrow,
                                __is_new_tick,
                            )
                        }
                    };
                    let op_9v1 = {
                        #[allow(non_snake_case)]
                        #[inline(always)]
                        pub fn op_9v1__join__loc_unknown_start_0_0_end_0_0<
                            Item,
                            Input: ::std::iter::Iterator<Item = Item>,
                        >(input: Input) -> impl ::std::iter::Iterator<Item = Item> {
                            struct Pull<
                                Item,
                                Input: ::std::iter::Iterator<Item = Item>,
                            > {
                                inner: Input,
                            }
                            impl<
                                Item,
                                Input: ::std::iter::Iterator<Item = Item>,
                            > Iterator for Pull<Item, Input> {
                                type Item = Item;
                                #[inline(always)]
                                fn next(&mut self) -> Option<Self::Item> {
                                    self.inner.next()
                                }
                                #[inline(always)]
                                fn size_hint(&self) -> (usize, Option<usize>) {
                                    self.inner.size_hint()
                                }
                            }
                            Pull { inner: input }
                        }
                        op_9v1__join__loc_unknown_start_0_0_end_0_0(op_9v1)
                    };
                    let op_10v1 = ::hydroflow::pusherator::for_each::ForEach::new(|x| {
                        results_inner
                            .borrow_mut()
                            .entry(context.current_tick())
                            .or_default()
                            .push(x)
                    });
                    let op_10v1 = {
                        #[allow(non_snake_case)]
                        #[inline(always)]
                        pub fn op_10v1__for_each__loc_unknown_start_0_0_end_0_0<
                            Item,
                            Input: ::hydroflow::pusherator::Pusherator<Item = Item>,
                        >(
                            input: Input,
                        ) -> impl ::hydroflow::pusherator::Pusherator<Item = Item> {
                            struct Push<
                                Item,
                                Input: ::hydroflow::pusherator::Pusherator<Item = Item>,
                            > {
                                inner: Input,
                            }
                            impl<
                                Item,
                                Input: ::hydroflow::pusherator::Pusherator<Item = Item>,
                            > ::hydroflow::pusherator::Pusherator for Push<Item, Input> {
                                type Item = Item;
                                #[inline(always)]
                                fn give(&mut self, item: Self::Item) {
                                    self.inner.give(item)
                                }
                            }
                            Push { inner: input }
                        }
                        op_10v1__for_each__loc_unknown_start_0_0_end_0_0(op_10v1)
                    };
                    #[inline(always)]
                    fn check_pivot_run<
                        Pull: ::std::iter::Iterator<Item = Item>,
                        Push: ::hydroflow::pusherator::Pusherator<Item = Item>,
                        Item,
                    >(pull: Pull, push: Push) {
                        ::hydroflow::pusherator::pivot::Pivot::new(pull, push).run();
                    }
                    check_pivot_run(op_9v1, op_10v1);
                    context.schedule_subgraph(context.current_subgraph(), false);
                },
            );
            df.add_subgraph_stratified(
                "Subgraph GraphSubgraphId(3v1)",
                0,
                (hoff_19v1_recv, ()),
                (hoff_12v1_send, ()),
                move |context, (hoff_19v1_recv, ()), (hoff_12v1_send, ())| {
                    let mut hoff_19v1_recv = hoff_19v1_recv.borrow_mut_swap();
                    let hoff_19v1_recv = hoff_19v1_recv.drain(..);
                    let hoff_12v1_send = ::hydroflow::pusherator::for_each::ForEach::new(|
                        v|
                    {
                        hoff_12v1_send.give(Some(v));
                    });
                    let op_6v1 = {
                        fn check_input<Iter: ::std::iter::Iterator<Item = Item>, Item>(
                            iter: Iter,
                        ) -> impl ::std::iter::Iterator<Item = Item> {
                            iter
                        }
                        check_input::<_, _>(hoff_19v1_recv)
                    };
                    let op_6v1 = {
                        #[allow(non_snake_case)]
                        #[inline(always)]
                        pub fn op_6v1__defer_tick__loc_unknown_start_0_0_end_0_0<
                            Item,
                            Input: ::std::iter::Iterator<Item = Item>,
                        >(input: Input) -> impl ::std::iter::Iterator<Item = Item> {
                            struct Pull<
                                Item,
                                Input: ::std::iter::Iterator<Item = Item>,
                            > {
                                inner: Input,
                            }
                            impl<
                                Item,
                                Input: ::std::iter::Iterator<Item = Item>,
                            > Iterator for Pull<Item, Input> {
                                type Item = Item;
                                #[inline(always)]
                                fn next(&mut self) -> Option<Self::Item> {
                                    self.inner.next()
                                }
                                #[inline(always)]
                                fn size_hint(&self) -> (usize, Option<usize>) {
                                    self.inner.size_hint()
                                }
                            }
                            Pull { inner: input }
                        }
                        op_6v1__defer_tick__loc_unknown_start_0_0_end_0_0(op_6v1)
                    };
                    #[inline(always)]
                    fn check_pivot_run<
                        Pull: ::std::iter::Iterator<Item = Item>,
                        Push: ::hydroflow::pusherator::Pusherator<Item = Item>,
                        Item,
                    >(pull: Pull, push: Push) {
                        ::hydroflow::pusherator::pivot::Pivot::new(pull, push).run();
                    }
                    check_pivot_run(op_6v1, hoff_12v1_send);
                },
            );
            df.add_subgraph_stratified(
                "Subgraph GraphSubgraphId(5v1)",
                1,
                (hoff_11v1_recv, ()),
                (hoff_15v1_send, ()),
                move |context, (hoff_11v1_recv, ()), (hoff_15v1_send, ())| {
                    let mut hoff_11v1_recv = hoff_11v1_recv.borrow_mut_swap();
                    let hoff_11v1_recv = hoff_11v1_recv.drain(..);
                    let hoff_15v1_send = ::hydroflow::pusherator::for_each::ForEach::new(|
                        v|
                    {
                        hoff_15v1_send.give(Some(v));
                    });
                    let op_14v1 = {
                        fn check_input<Iter: ::std::iter::Iterator<Item = Item>, Item>(
                            iter: Iter,
                        ) -> impl ::std::iter::Iterator<Item = Item> {
                            iter
                        }
                        check_input::<_, _>(hoff_11v1_recv)
                    };
                    let op_14v1 = {
                        #[allow(non_snake_case)]
                        #[inline(always)]
                        pub fn op_14v1__identity__loc_unknown_start_0_0_end_0_0<
                            Item,
                            Input: ::std::iter::Iterator<Item = Item>,
                        >(input: Input) -> impl ::std::iter::Iterator<Item = Item> {
                            struct Pull<
                                Item,
                                Input: ::std::iter::Iterator<Item = Item>,
                            > {
                                inner: Input,
                            }
                            impl<
                                Item,
                                Input: ::std::iter::Iterator<Item = Item>,
                            > Iterator for Pull<Item, Input> {
                                type Item = Item;
                                #[inline(always)]
                                fn next(&mut self) -> Option<Self::Item> {
                                    self.inner.next()
                                }
                                #[inline(always)]
                                fn size_hint(&self) -> (usize, Option<usize>) {
                                    self.inner.size_hint()
                                }
                            }
                            Pull { inner: input }
                        }
                        op_14v1__identity__loc_unknown_start_0_0_end_0_0(op_14v1)
                    };
                    #[inline(always)]
                    fn check_pivot_run<
                        Pull: ::std::iter::Iterator<Item = Item>,
                        Push: ::hydroflow::pusherator::Pusherator<Item = Item>,
                        Item,
                    >(pull: Pull, push: Push) {
                        ::hydroflow::pusherator::pivot::Pivot::new(pull, push).run();
                    }
                    check_pivot_run(op_14v1, hoff_15v1_send);
                },
            );
            df.add_subgraph_stratified(
                "Subgraph GraphSubgraphId(6v1)",
                1,
                (hoff_12v1_recv, ()),
                (hoff_17v1_send, ()),
                move |context, (hoff_12v1_recv, ()), (hoff_17v1_send, ())| {
                    let mut hoff_12v1_recv = hoff_12v1_recv.borrow_mut_swap();
                    let hoff_12v1_recv = hoff_12v1_recv.drain(..);
                    let hoff_17v1_send = ::hydroflow::pusherator::for_each::ForEach::new(|
                        v|
                    {
                        hoff_17v1_send.give(Some(v));
                    });
                    let op_16v1 = {
                        fn check_input<Iter: ::std::iter::Iterator<Item = Item>, Item>(
                            iter: Iter,
                        ) -> impl ::std::iter::Iterator<Item = Item> {
                            iter
                        }
                        check_input::<_, _>(hoff_12v1_recv)
                    };
                    let op_16v1 = {
                        #[allow(non_snake_case)]
                        #[inline(always)]
                        pub fn op_16v1__identity__loc_unknown_start_0_0_end_0_0<
                            Item,
                            Input: ::std::iter::Iterator<Item = Item>,
                        >(input: Input) -> impl ::std::iter::Iterator<Item = Item> {
                            struct Pull<
                                Item,
                                Input: ::std::iter::Iterator<Item = Item>,
                            > {
                                inner: Input,
                            }
                            impl<
                                Item,
                                Input: ::std::iter::Iterator<Item = Item>,
                            > Iterator for Pull<Item, Input> {
                                type Item = Item;
                                #[inline(always)]
                                fn next(&mut self) -> Option<Self::Item> {
                                    self.inner.next()
                                }
                                #[inline(always)]
                                fn size_hint(&self) -> (usize, Option<usize>) {
                                    self.inner.size_hint()
                                }
                            }
                            Pull { inner: input }
                        }
                        op_16v1__identity__loc_unknown_start_0_0_end_0_0(op_16v1)
                    };
                    #[inline(always)]
                    fn check_pivot_run<
                        Pull: ::std::iter::Iterator<Item = Item>,
                        Push: ::hydroflow::pusherator::Pusherator<Item = Item>,
                        Item,
                    >(pull: Pull, push: Push) {
                        ::hydroflow::pusherator::pivot::Pivot::new(pull, push).run();
                    }
                    check_pivot_run(op_16v1, hoff_17v1_send);
                },
            );
            df.add_subgraph_stratified(
                "Subgraph GraphSubgraphId(7v1)",
                1,
                (hoff_13v1_recv, ()),
                (hoff_19v1_send, ()),
                move |context, (hoff_13v1_recv, ()), (hoff_19v1_send, ())| {
                    let mut hoff_13v1_recv = hoff_13v1_recv.borrow_mut_swap();
                    let hoff_13v1_recv = hoff_13v1_recv.drain(..);
                    let hoff_19v1_send = ::hydroflow::pusherator::for_each::ForEach::new(|
                        v|
                    {
                        hoff_19v1_send.give(Some(v));
                    });
                    let op_18v1 = {
                        fn check_input<Iter: ::std::iter::Iterator<Item = Item>, Item>(
                            iter: Iter,
                        ) -> impl ::std::iter::Iterator<Item = Item> {
                            iter
                        }
                        check_input::<_, _>(hoff_13v1_recv)
                    };
                    let op_18v1 = {
                        #[allow(non_snake_case)]
                        #[inline(always)]
                        pub fn op_18v1__identity__loc_unknown_start_0_0_end_0_0<
                            Item,
                            Input: ::std::iter::Iterator<Item = Item>,
                        >(input: Input) -> impl ::std::iter::Iterator<Item = Item> {
                            struct Pull<
                                Item,
                                Input: ::std::iter::Iterator<Item = Item>,
                            > {
                                inner: Input,
                            }
                            impl<
                                Item,
                                Input: ::std::iter::Iterator<Item = Item>,
                            > Iterator for Pull<Item, Input> {
                                type Item = Item;
                                #[inline(always)]
                                fn next(&mut self) -> Option<Self::Item> {
                                    self.inner.next()
                                }
                                #[inline(always)]
                                fn size_hint(&self) -> (usize, Option<usize>) {
                                    self.inner.size_hint()
                                }
                            }
                            Pull { inner: input }
                        }
                        op_18v1__identity__loc_unknown_start_0_0_end_0_0(op_18v1)
                    };
                    #[inline(always)]
                    fn check_pivot_run<
                        Pull: ::std::iter::Iterator<Item = Item>,
                        Push: ::hydroflow::pusherator::Pusherator<Item = Item>,
                        Item,
                    >(pull: Pull, push: Push) {
                        ::hydroflow::pusherator::pivot::Pivot::new(pull, push).run();
                    }
                    check_pivot_run(op_18v1, hoff_19v1_send);
                },
            );
            df
        }
    };
    df.run_available();
    #[rustfmt::skip]
    {
        {
            for v in &[(7, (1, 0)), (7, (2, 0))] {
                if !results.borrow()[&0].contains(v) {
                    {
                        ::core::panicking::panic_fmt(
                            format_args!(
                                "did not contain: {0:?} in {1:?}", v, results.borrow() [& 0]
                            ),
                        );
                    }
                }
            }
        };
        {
            for v in &[(7, (1, 0)), (7, (2, 0)), (7, (1, 1)), (7, (2, 1))] {
                if !results.borrow()[&1].contains(v) {
                    {
                        ::core::panicking::panic_fmt(
                            format_args!(
                                "did not contain: {0:?} in {1:?}", v, results.borrow() [& 1]
                            ),
                        );
                    }
                }
            }
        };
        {
            for v in &[
                (7, (1, 0)),
                (7, (2, 0)),
                (7, (1, 1)),
                (7, (2, 1)),
                (7, (1, 2)),
                (7, (2, 2)),
            ] {
                if !results.borrow()[&2].contains(v) {
                    {
                        ::core::panicking::panic_fmt(
                            format_args!(
                                "did not contain: {0:?} in {1:?}", v, results.borrow() [& 2]
                            ),
                        );
                    }
                }
            }
        };
        {
            match (&results.borrow().get(&3), &None) {
                (left_val, right_val) => {
                    if !(*left_val == *right_val) {
                        let kind = ::core::panicking::AssertKind::Eq;
                        ::core::panicking::assert_failed(
                            kind,
                            &*left_val,
                            &*right_val,
                            ::core::option::Option::None,
                        );
                    }
                }
            };
        };
    };
}
