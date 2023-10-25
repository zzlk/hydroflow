use hydroflow::hydroflow_syntax;

pub fn main() {
    let mut df = hydroflow_syntax! {
        source_iter(["Hello World"])
            -> instrument("tp1")
            -> assert_eq(["meme"]);
    };
    df.run_available();
}

#[test]
fn test() {
    main();
}
