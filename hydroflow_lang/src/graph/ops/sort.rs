use quote::quote_spanned;

use super::{
    DelayType, FlowProperties, FlowPropertyVal, OperatorConstraints, OperatorWriteOutput,
    WriteContextArgs, RANGE_0, RANGE_1,
};

/// Takes a stream as input and produces a sorted version of the stream as output.
///
/// ```hydroflow
/// // should print 1, 2, 3 (in order)
/// source_iter(vec![2, 3, 1])
///     -> sort()
///     -> for_each(|x| println!("{}", x));
/// ```
///
/// `sort` is partially blocking. Only the values collected within a single tick will be sorted and
/// emitted.
pub const SORT: OperatorConstraints = OperatorConstraints {
    name: "sort",
    hard_range_inn: RANGE_1,
    soft_range_inn: RANGE_1,
    hard_range_out: RANGE_1,
    soft_range_out: RANGE_1,
    num_args: 0,
    persistence_args: RANGE_0,
    type_args: RANGE_0,
    is_external_input: false,
    ports_inn: None,
    ports_out: None,
    properties: FlowProperties {
        deterministic: FlowPropertyVal::Preserve,
        monotonic: FlowPropertyVal::No,
        inconsistency_tainted: false,
    },
    input_delaytype_fn: |_| Some(DelayType::Stratum),
    write_fn: |&WriteContextArgs {
                   op_span,
                   ident,
                   inputs,
                   is_pull,
                   ..
               },
               _| {
        assert!(is_pull);

        let input = &inputs[0];
        let write_iterator = quote_spanned! {op_span=>
            // TODO(mingwei): unneccesary extra handoff into_iter() then collect().
            // Fix requires handoff specialization.
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
    },
};
