use quote::{quote_spanned, ToTokens};
use syn::parse_quote;

use super::{
    FlowProperties, FlowPropertyVal, OperatorCategory, OperatorConstraints, OperatorWriteOutput,
    Persistence, WriteContextArgs, RANGE_1,
};
use crate::diagnostic::{Diagnostic, Level};
use crate::graph::{OpInstGenerics, OperatorInstance};

/// > 2 input streams of type <(K, V1)> and <(K, V2)>, 1 output stream of type <(K, (V1, V2))>
///
/// Forms the equijoin of the tuples in the input streams by their first (key) attribute. Note that the result nests the 2nd input field (values) into a tuple in the 2nd output field.
///
/// ```hydroflow
/// source_iter(vec![("hello", "world"), ("stay", "gold"), ("hello", "world")]) -> [0]my_join;
/// source_iter(vec![("hello", "cleveland")]) -> [1]my_join;
/// my_join = join()
///     -> assert_eq([("hello", ("world", "cleveland"))]);
/// ```
///
/// `join` can also be provided with one or two generic lifetime persistence arguments, either
/// `'tick` or `'static`, to specify how join data persists. With `'tick`, pairs will only be
/// joined with corresponding pairs within the same tick. With `'static`, pairs will be remembered
/// across ticks and will be joined with pairs arriving in later ticks. When not explicitly
/// specified persistence defaults to `tick.
///
/// When two persistence arguments are supplied the first maps to port `0` and the second maps to
/// port `1`.
/// When a single persistence argument is supplied, it is applied to both input ports.
/// When no persistence arguments are applied it defaults to `'tick` for both.
///
/// The syntax is as follows:
/// ```hydroflow,ignore
/// join(); // Or
/// join::<'static>();
///
/// join::<'tick>();
///
/// join::<'static, 'tick>();
///
/// join::<'tick, 'static>();
/// // etc.
/// ```
///
/// `join` is defined to treat its inputs as *sets*, meaning that it
/// eliminates duplicated values in its inputs. If you do not want
/// duplicates eliminated, use the [`join_multiset`](#join_multiset) operator.
///
/// ### Examples
///
/// ```rustbook
/// let (input_send, input_recv) = hydroflow::util::unbounded_channel::<(&str, &str)>();
/// let mut flow = hydroflow::hydroflow_syntax! {
///     source_iter([("hello", "world")]) -> [0]my_join;
///     source_stream(input_recv) -> [1]my_join;
///     my_join = join::<'tick>() -> for_each(|(k, (v1, v2))| println!("({}, ({}, {}))", k, v1, v2));
/// };
/// input_send.send(("hello", "oakland")).unwrap();
/// flow.run_tick();
/// input_send.send(("hello", "san francisco")).unwrap();
/// flow.run_tick();
/// ```
/// Prints out `"(hello, (world, oakland))"` since `source_iter([("hello", "world")])` is only
/// included in the first tick, then forgotten.
///
/// ---
///
/// ```rustbook
/// let (input_send, input_recv) = hydroflow::util::unbounded_channel::<(&str, &str)>();
/// let mut flow = hydroflow::hydroflow_syntax! {
///     source_iter([("hello", "world")]) -> [0]my_join;
///     source_stream(input_recv) -> [1]my_join;
///     my_join = join::<'static>() -> for_each(|(k, (v1, v2))| println!("({}, ({}, {}))", k, v1, v2));
/// };
/// input_send.send(("hello", "oakland")).unwrap();
/// flow.run_tick();
/// input_send.send(("hello", "san francisco")).unwrap();
/// flow.run_tick();
/// ```
/// Prints out `"(hello, (world, oakland))"` and `"(hello, (world, san francisco))"` since the
/// inputs are peristed across ticks.
pub const JOIN: OperatorConstraints = OperatorConstraints {
    name: "join",
    categories: &[OperatorCategory::MultiIn],
    hard_range_inn: &(2..=2),
    soft_range_inn: &(2..=2),
    hard_range_out: RANGE_1,
    soft_range_out: RANGE_1,
    num_args: 0,
    persistence_args: &(0..=2),
    type_args: &(0..=1),
    is_external_input: false,
    ports_inn: Some(|| super::PortListSpec::Fixed(parse_quote! { 0, 1 })),
    ports_out: None,
    properties: FlowProperties {
        deterministic: FlowPropertyVal::Preserve,
        monotonic: FlowPropertyVal::Preserve,
        inconsistency_tainted: false,
    },
    input_delaytype_fn: |_| None,
    write_fn: |wc @ &WriteContextArgs {
                   root,
                   context,
                   hydroflow,
                   op_span,
                   ident,
                   inputs,
                   op_inst:
                       OperatorInstance {
                           generics:
                               OpInstGenerics {
                                   persistence_args,
                                   type_args,
                                   ..
                               },
                           ..
                       },
                   ..
               },
               diagnostics| {
        let join_type =
            type_args
                .get(0)
                .map(ToTokens::to_token_stream)
                .unwrap_or(quote_spanned!(op_span=>
                    #root::compiled::pull::HalfSetJoinState
                ));

        // TODO: This is really bad.
        // This will break if the user aliases HalfSetJoinState to something else. Temporary hacky solution.
        // Note that cross_join() depends on the implementation here as well.
        // Need to decide on what to do about multisetjoin.
        // Should it be a separate operator (multisetjoin() and multisetcrossjoin())?
        // Should the default be multiset join? And setjoin requires the use of lattice_join() with SetUnion lattice?
        let additional_trait_bounds = if join_type.to_string().contains("HalfSetJoinState") {
            quote_spanned!(op_span=>
                + ::std::cmp::Eq
            )
        } else {
            quote_spanned!(op_span=>)
        };

        let mut make_joindata = |persistence, side| {
            let joindata_ident = wc.make_ident(format!("joindata_{}", side));
            let borrow_ident = wc.make_ident(format!("joindata_{}_borrow", side));
            let (init, borrow) = match persistence {
                Persistence::Tick => (
                    quote_spanned! {op_span=>
                        #root::util::monotonic_map::MonotonicMap::new_init(
                            #join_type::default()
                        )
                    },
                    quote_spanned! {op_span=>
                        &mut *#borrow_ident.get_mut_clear(#context.current_tick())
                    },
                ),
                Persistence::Static => (
                    quote_spanned! {op_span=>
                        #join_type::default()
                    },
                    quote_spanned! {op_span=>
                        &mut *#borrow_ident
                    },
                ),
                Persistence::Mutable => {
                    diagnostics.push(Diagnostic::spanned(
                        op_span,
                        Level::Error,
                        "An implementation of 'mutable does not exist",
                    ));
                    return Err(());
                }
            };
            Ok((joindata_ident, borrow_ident, init, borrow))
        };

        let persistences = match persistence_args[..] {
            [] => [Persistence::Tick, Persistence::Tick],
            [a] => [a, a],
            [a, b] => [a, b],
            _ => unreachable!(),
        };

        let (lhs_joindata_ident, lhs_borrow_ident, lhs_init, lhs_borrow) =
            make_joindata(persistences[0], "lhs")?;
        let (rhs_joindata_ident, rhs_borrow_ident, rhs_init, rhs_borrow) =
            make_joindata(persistences[1], "rhs")?;

        let tick_ident = wc.make_ident("persisttick");
        let tick_borrow_ident = wc.make_ident("persisttick_borrow");

        let write_prologue = quote_spanned! {op_span=>
            let #lhs_joindata_ident = #hydroflow.add_state(std::cell::RefCell::new(
                #lhs_init
            ));
            let #rhs_joindata_ident = #hydroflow.add_state(std::cell::RefCell::new(
                #rhs_init
            ));
            let #tick_ident = #hydroflow.add_state(std::cell::RefCell::new(
                0usize
            ));
        };

        let lhs = &inputs[0];
        let rhs = &inputs[1];
        let write_iterator = quote_spanned! {op_span=>
            let mut #lhs_borrow_ident = || #context.state_ref(#lhs_joindata_ident).borrow_mut();
            let mut #rhs_borrow_ident = || #context.state_ref(#rhs_joindata_ident).borrow_mut();
            let mut #tick_borrow_ident = #context.state_ref(#tick_ident).borrow_mut();

            let #ident = {
                // Limit error propagation by bounding locally, erasing output iterator type.
                #[inline(always)]
                fn check_inputs<'a, K, I1, V1, I2, V2>(
                    mut lhs: I1,
                    mut rhs: I2,
                    lhs_state_fn: impl 'a + Fn() -> ::std::cell::RefMut<'a, #join_type<K, V1, V2>>,
                    rhs_state_fn: impl 'a + Fn() -> ::std::cell::RefMut<'a, #join_type<K, V2, V1>>,
                    is_new_tick: bool,
                ) -> impl 'a + Iterator<Item = (K, (V1, V2))>
                where
                    K: 'a + Eq + std::hash::Hash + Clone,
                    V1: 'a + Clone #additional_trait_bounds,
                    V2: 'a + Clone #additional_trait_bounds,
                    I1: 'a + Iterator<Item = (K, V1)>,
                    I2: 'a + Iterator<Item = (K, V2)>,
                {
                    use #root::compiled::pull::HalfJoinState;

                    // enum Either<ILhs, IRhs, K, V1, V2>
                    // where
                    //     ILhs: Iterator<Item = (K, V1)>,
                    //     IRhs: Iterator<Item = (K, V2)>,
                    // {
                    //     Lhs(ILhs),
                    //     Rhs(IRhs),
                    //     None,
                    // }

                    // fn extend_lifetime<'a, 'b, T>(x: &'a mut T) -> &'b mut T
                    // where
                    //     'b: 'a
                    // {
                    //     unsafe {
                    //         &mut *(x as *mut T)
                    //     }
                    // }

                    // let mut keys = if is_new_tick {
                    //     if lhs_state.len() < rhs_state.len() {
                    //         Either::Lhs(extend_lifetime(lhs_state).iter())
                    //     } else {
                    //         Either::Rhs(extend_lifetime(rhs_state).iter())
                    //     }
                    // } else {
                    //     Either::None
                    // };

                    let mut #lhs_borrow_ident = (lhs_state_fn)();
                    // let mut lhs_iter_state = #lhs_borrow;
                    let mut lhs_keys = is_new_tick.then(|| ::std::cell::RefMut::map(#lhs_borrow_ident, |x| (&*x).iter()));

                    ::std::iter::from_fn(move || loop {
                        let mut #lhs_borrow_ident = (lhs_state_fn)();
                        let mut #rhs_borrow_ident = (rhs_state_fn)();

                        let mut lhs_state = #lhs_borrow;
                        let mut rhs_state = #rhs_borrow;

                        // Emit matches
                        // STAGE 1 AND 2
                        if let ::std::option::Option::Some((k, v1, v2)) = rhs_state.pop_match() {
                            return ::std::option::Option::Some((k, (v1, v2)));
                        }
                        // STAGE 1
                        if let ::std::option::Option::Some(ref mut x) = lhs_keys {
                            while let ::std::option::Option::Some((k, v1)) = x.next() {
                                if let ::std::option::Option::Some((k, v1, v2)) = rhs_state.probe(&k, &v1) {
                                    return ::std::option::Option::Some((k, (v1, v2)));
                                }
                                continue;
                            }
                        }

                        // STAGE 2
                        lhs_keys = ::std::option::Option::None;
                        if let ::std::option::Option::Some((k, v2, v1)) = lhs_state.pop_match() {
                            return ::std::option::Option::Some((k, (v1, v2)));
                        }

                        if let ::std::option::Option::Some((k, v1)) = lhs.next() {
                            if lhs_state.build(k.clone(), &v1) {
                                if let ::std::option::Option::Some((k, v1, v2)) = rhs_state.probe(&k, &v1) {
                                    return ::std::option::Option::Some((k, (v1, v2)));
                                }
                            }
                            continue;
                        }

                        if let ::std::option::Option::Some((k, v2)) = rhs.next() {
                            if rhs_state.build(k.clone(), &v2) {
                                if let ::std::option::Option::Some((k, v2, v1)) = lhs_state.probe(&k, &v2) {
                                    return ::std::option::Option::Some((k, (v1, v2)));
                                }
                            }
                            continue;
                        }

                        return None;
                    })
                }

                {
                    let __is_new_tick = if *#tick_borrow_ident < #context.current_tick() {
                        *#tick_borrow_ident = #context.current_tick();
                        true
                    } else {
                        false
                    };

                    check_inputs(#lhs, #rhs, #lhs_borrow_ident, #rhs_borrow_ident, __is_new_tick)
                }
            };
        };

        let write_iterator_after =
            if persistences[0] == Persistence::Static && persistences[1] == Persistence::Static {
                quote_spanned! {op_span=>
                    // TODO: Probably only need to schedule if #*_borrow.len() > 0?
                    #context.schedule_subgraph(#context.current_subgraph(), false);
                }
            } else {
                quote_spanned! {op_span=>}
            };

        Ok(OperatorWriteOutput {
            write_prologue,
            write_iterator,
            write_iterator_after,
        })
    },
};
