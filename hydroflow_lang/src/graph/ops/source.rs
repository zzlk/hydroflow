use quote::{quote_spanned, ToTokens};

use super::{
    FlowProperties, FlowPropertyVal, OperatorConstraints, OperatorWriteOutput, WriteContextArgs,
    RANGE_0, RANGE_1,
};
use crate::graph::{OpInstGenerics, OperatorInstance};

/// > 0 input streams, 1 output stream of `bytes::Bytes`
///
/// > Arguments: The logical name of an source port
///
/// The logical port name can be externally accessed outside of the hydroflow program and connected to a data source.
///
/// ```rustbook
/// async fn example() {
///     let mut df = hydroflow::hydroflow_syntax! {
///         source_port("source") -> for_each(|x| println!("{:?}", x));
///     };
///     let input = df.take_port_senders().remove("source").unwrap();
///     input.send(bytes::Bytes::from_static(b"hello")).await.unwrap();
///     df.run_available_async();
/// }
/// ```
#[hydroflow_internalmacro::operator_docgen]
pub const SOURCE: OperatorConstraints = OperatorConstraints {
    name: "source",
    hard_range_inn: RANGE_0,
    soft_range_inn: RANGE_0,
    hard_range_out: RANGE_1,
    soft_range_out: RANGE_1,
    num_args: 1,
    persistence_args: RANGE_0,
    type_args: &(0..=1),
    is_external_input: true,
    ports_inn: None,
    ports_out: None,
    properties: FlowProperties {
        deterministic: FlowPropertyVal::DependsOnArgs,
        monotonic: FlowPropertyVal::DependsOnArgs,
        inconsistency_tainted: false,
    },
    input_delaytype_fn: |_| None,
    write_fn: |wc @ &WriteContextArgs {
                   root,
                   context,
                   op_span,
                   ident,
                   hydroflow,
                   op_inst:
                       op_inst @ OperatorInstance {
                           arguments,
                           generics: OpInstGenerics { type_args, .. },
                           ..
                       },
                   ..
               },
               diagnostics| {
        let func = &arguments[0];

        let port_type = type_args
            .get(0)
            .map(ToTokens::to_token_stream)
            .unwrap_or(quote_spanned!(op_span=> _));

        let stream_ident = wc.make_ident("stream");
        let prologue = quote_spanned! {op_span=>
            let #stream_ident = {
                fn check_types<T>(port_getter: impl Stream<Item = T> + ::std::marker::Unpin) -> impl Stream<Item = T> + ::std::marker::Unpin {
                    port_getter
                }
                let stream = check_types::<#port_type>((#func).await);

                stream
            };
        };

        let mut punct = syn::punctuated::Punctuated::new();
        punct.push(syn::Expr::Verbatim(
            quote_spanned!(op_span=> #stream_ident ),
        ));

        let OperatorWriteOutput {
            write_prologue,
            write_iterator,
            write_iterator_after,
        } = (super::source_stream::SOURCE_STREAM.write_fn)(
            &WriteContextArgs {
                op_inst: &OperatorInstance {
                    arguments: punct,
                    ..op_inst.clone()
                },
                ..*wc
            },
            diagnostics,
        )?;

        let write_prologue = quote_spanned!(op_span=>
            #prologue
            #write_prologue
        );

        Ok(OperatorWriteOutput {
            write_prologue,
            write_iterator,
            write_iterator_after,
        })
    },
};
