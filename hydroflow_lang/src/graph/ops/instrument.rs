use quote::quote_spanned;

use super::{
    OperatorCategory, OperatorConstraints, OperatorInstance, OperatorWriteOutput, WriteContextArgs,
    RANGE_0, RANGE_1,
};

/// TODO:
pub const INSTRUMENT: OperatorConstraints = OperatorConstraints {
    name: "instrument",
    categories: &[OperatorCategory::Map],
    hard_range_inn: RANGE_1,
    soft_range_inn: RANGE_1,
    hard_range_out: RANGE_1,
    soft_range_out: RANGE_1,
    num_args: 1,
    persistence_args: RANGE_0,
    type_args: RANGE_0,
    is_external_input: false,
    ports_inn: None,
    ports_out: None,
    input_delaytype_fn: |_| None,
    flow_prop_fn: None,
    write_fn: |wc @ &WriteContextArgs {
                   hydroflow,
                   root,
                   op_span,
                   ident,
                   inputs,
                   outputs,
                   is_pull,
                   op_inst: OperatorInstance { arguments, .. },
                   ..
               },
               _| {
        let name: &syn::Expr = &arguments[0];
        let instrument_data = wc.make_ident("instrument_data");

        let write_prologue = quote_spanned! {op_span=>
            let #instrument_data = #hydroflow.add_state(
                ::std::cell::RefCell::new(0usize)
            );
        };

        let write_iterator = if is_pull {
            let input = &inputs[0];
            quote_spanned! {op_span=>
                let #ident = #input.inspect(|_x| {
                    eprintln!("pull {}: {}", #name, _x);
                });
            }
        } else {
            let output = &outputs[0];

            quote_spanned! {op_span=>
                let #ident = #root::pusherator::inspect::Inspect::new(|_x| {
                    eprintln!("push {}: {}", #name, _x);
                }, #output);
            }
        };

        Ok(OperatorWriteOutput {
            write_prologue,
            write_iterator,
            ..Default::default()
        })
    },
};
