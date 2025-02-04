use crate::diagnostic::{Diagnostic, Level};

use super::{
    parse_persistence_lifetimes, DelayType, OperatorConstraints, OperatorWriteOutput, Persistence,
    WriteContextArgs, WriteIteratorArgs, RANGE_1,
};

use quote::quote_spanned;
use syn::spanned::Spanned;

/// Takes a stream as input and produces a sorted version of the stream as output.
///
/// ```hydroflow
/// // should print 1, 2, 3 (in order)
/// source_iter(vec![2, 3, 1])
///     -> sort()
///     -> for_each(|x| println!("{}", x));
/// ```
///
/// `sort` can also be provided with one generic lifetime persistence argument, either
/// `'tick` or `'static`, to specify how data persists. The default is `'tick`. With `'tick` only
/// the values will only be collected within a single tick will be sorted and emitted. With
/// `'static`, values will be remembered across ticks and will be repeatedly emitted each tick (in
/// order).
///
/// ```rustbook
/// let (input_send, input_recv) = hydroflow::util::unbounded_channel::<usize>();
/// let mut flow = hydroflow::hydroflow_syntax! {
///     source_stream(input_recv)
///         -> sort::<'static>()
///         -> for_each(|n| println!("{}", n));
/// };
///
/// input_send.send(6).unwrap();
/// input_send.send(3).unwrap();
/// input_send.send(4).unwrap();
/// flow.run_available();
/// // 3, 4, 6
///
/// input_send.send(1).unwrap();
/// input_send.send(7).unwrap();
/// flow.run_available();
/// // 1, 3, 4, 6, 7
/// ```
#[hydroflow_internalmacro::operator_docgen]
pub const SORT: OperatorConstraints = OperatorConstraints {
    name: "sort",
    hard_range_inn: RANGE_1,
    soft_range_inn: RANGE_1,
    hard_range_out: RANGE_1,
    soft_range_out: RANGE_1,
    ports_inn: None,
    ports_out: None,
    num_args: 0,
    input_delaytype_fn: &|_| Some(DelayType::Stratum),
    write_fn: &(|wc @ &WriteContextArgs { op_span, .. },
                 wi @ &WriteIteratorArgs {
                     ident,
                     inputs,
                     is_pull,
                     generic_args,
                     op_name,
                     ..
                 },
                 diagnostics| {
        assert!(is_pull);

        let generics_span = generic_args.map(Spanned::span).unwrap_or(op_span);

        let persistence = parse_persistence_lifetimes(wi, diagnostics);
        let persistence = match *persistence {
            [] => Persistence::Tick,
            [a] => a,
            _ => {
                diagnostics.push(Diagnostic::spanned(
                    generics_span,
                    Level::Error,
                    format!(
                        "Operator `{}` expects zero or one persistence lifetime generic arguments",
                        op_name
                    ),
                ));
                Persistence::Static
            }
        };

        let input = &inputs[0];
        match persistence {
            Persistence::Tick => {
                let write_iterator = quote_spanned! {op_span=>
                    // TODO(mingwei): unneccesary extra into_iter() then collect()
                    let #ident = {
                        let mut v = #input.collect::<::std::vec::Vec<_>>();
                        v.sort_unstable();
                        v.into_iter()
                    };
                };
                Ok(OperatorWriteOutput {
                    write_iterator,
                    ..Default::default()
                })
            }
            Persistence::Static => {
                let sortdata_ident = wc.make_ident("sortdata");

                let write_prologue = quote_spanned! {op_span=>
                    let #sortdata_ident = df.add_state(::std::cell::RefCell::new(::std::vec::Vec::new()));
                };
                let write_iterator = quote_spanned! {op_span=>
                    // TODO(mingwei): Better data structure for this?
                    let #ident = {
                        let mut v = context.state_ref(#sortdata_ident).borrow_mut();
                        v.extend(#input);
                        v.sort_unstable();
                        v.clone().into_iter()
                    };
                };

                Ok(OperatorWriteOutput {
                    write_prologue,
                    write_iterator,
                    ..Default::default()
                })
            }
        }
    }),
};
