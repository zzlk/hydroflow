use quote::{quote_spanned, ToTokens};

use super::{
    FlowProperties, FlowPropertyVal, OperatorCategory, OperatorConstraints, OperatorWriteOutput,
    Persistence, WriteContextArgs, RANGE_0, RANGE_1,
};
use crate::diagnostic::{Diagnostic, Level};
use crate::graph::{OpInstGenerics, OperatorInstance};

/// Stores each item as it passes through, and replays all item every tick.
///
/// ```hydroflow
/// // Normally `source_iter(...)` only emits once, but with `persist()` will replay the `"hello"`
/// // on every tick.
/// source_iter(["hello"])
///     -> persist()
///     -> for_each(|item| println!("{}: {}", context.current_tick(), item));
/// ```
///
/// `persist()` can be used to introduce statefulness into stateless pipelines. For example this
/// join only stores data for single `'tick`. The `persist()` operator introduces statefulness
/// across ticks. This can be useful for optimization transformations within the hydroflow
/// compiler.
/// ```rustbook
/// let (input_send, input_recv) = hydroflow::util::unbounded_channel::<(&str, &str)>();
/// let mut flow = hydroflow::hydroflow_syntax! {
///     source_iter([("hello", "world")]) -> persist() -> [0]my_join;
///     source_stream(input_recv) -> persist() -> [1]my_join;
///     my_join = join::<'tick>() -> for_each(|(k, (v1, v2))| println!("({}, ({}, {}))", k, v1, v2));
/// };
/// input_send.send(("hello", "oakland")).unwrap();
/// flow.run_tick();
/// input_send.send(("hello", "san francisco")).unwrap();
/// flow.run_tick();
/// // (hello, (world, oakland))
/// // (hello, (world, oakland))
/// // (hello, (world, san francisco))
/// ```
pub const PERSIST_MUT_KEYED: OperatorConstraints = OperatorConstraints {
    name: "persist_mut_keyed",
    categories: &[OperatorCategory::Persistence],
    hard_range_inn: RANGE_1,
    soft_range_inn: RANGE_1,
    hard_range_out: RANGE_1,
    soft_range_out: RANGE_1,
    num_args: 0,
    persistence_args: &(0..=1),
    type_args: &(0..=1),
    is_external_input: false,
    ports_inn: None,
    ports_out: None,
    properties: FlowProperties {
        deterministic: FlowPropertyVal::Preserve,
        monotonic: FlowPropertyVal::Yes,
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
                   outputs,
                   is_pull,
                   op_name,
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
        if matches!(persistence_args[..], [Persistence::Tick]) {
            diagnostics.push(Diagnostic::spanned(
                op_span,
                Level::Error,
                format!("`{}()` can only have `'static` persistence.", op_name),
            ));
            return Err(());
        };

        let generic_type_arg = type_args
            .get(0)
            .map(ToTokens::to_token_stream)
            .unwrap_or(quote_spanned!(op_span=> #root::util::Bag<_>));

        let persistdata_ident = wc.make_ident("persistdata");
        let vec_ident = wc.make_ident("persistvec");
        let tick_ident = wc.make_ident("persisttick");
        let write_prologue = quote_spanned! {op_span=>
            let #persistdata_ident = #hydroflow.add_state(::std::cell::RefCell::new((
                0_usize, // tick
                <#generic_type_arg as ::std::default::Default>::default(),
            )));
        };

        let write_iterator = if is_pull {
            let input = &inputs[0];

            quote_spanned! {op_span=>
                fn check_input<K, V>(
                    iter: impl Iterator<Item = ((K, V), bool)>,
                ) -> impl Iterator<Item = ((K, V), bool)> {
                    iter
                }

                let mut #vec_ident = #context.state_ref(#persistdata_ident).borrow_mut();
                let (ref mut #tick_ident, ref mut #vec_ident) = &mut *#vec_ident;
                let #ident = {
                    if *#tick_ident <= #context.current_tick() {
                        *#tick_ident = 1 + #context.current_tick();
                        for ((k, v), unpersist) in check_input(#input) {
                            if unpersist {
                                PersistenceMutKeyed::unpersist(#vec_ident, k);
                            } else {
                                PersistenceKeyed::persist(#vec_ident, k, v);
                            }
                        }
                        PersistenceKeyed::iter(#vec_ident)
                    } else {
                        for ((k, v), unpersist) in check_input(#input) {
                            if unpersist {
                                PersistenceMutKeyed::unpersist(#vec_ident, k);
                            } else {
                                PersistenceKeyed::persist(#vec_ident, k, v);
                            }
                        }
                        // TODO: how to handle the semi-persistence stuff
                        // let len = #vec_ident.len();
                        // #vec_ident[len..].iter().cloned()
                        PersistenceKeyed::iter(#vec_ident)
                    }
                };
            }
        } else {
            let output = &outputs[0];
            quote_spanned! {op_span=>
                let mut #vec_ident = #context.state_ref(#persistdata_ident).borrow_mut();
                let (ref mut #tick_ident, ref mut #vec_ident) = &mut *#vec_ident;
                let #ident = {
                    fn constrain_types<'ctx, Push, Item>(curr_tick: usize, tick: &'ctx mut usize, vec: &'ctx mut Vec<Item>, mut output: Push) -> impl 'ctx + #root::pusherator::Pusherator<Item = Item>
                    where
                        Push: 'ctx + #root::pusherator::Pusherator<Item = Item>,
                        Item: ::std::clone::Clone,
                    {
                        if *tick <= curr_tick {
                            *tick = 1 + curr_tick;
                            vec.iter().cloned().for_each(|item| {
                                #root::pusherator::Pusherator::give(&mut output, item);
                            });
                        }
                        #root::pusherator::map::Map::new(|item| {
                            PersistenceKeyed::persist(vec, item);
                            vec.last().unwrap().clone()
                        }, output)
                    }
                    constrain_types(#context.current_tick(), #tick_ident, #vec_ident, #output)
                };
            }
        };

        let write_iterator_after = quote_spanned! {op_span=>
            #context.schedule_subgraph(#context.current_subgraph(), false);
        };

        Ok(OperatorWriteOutput {
            write_prologue,
            write_iterator,
            write_iterator_after,
        })
    },
};
