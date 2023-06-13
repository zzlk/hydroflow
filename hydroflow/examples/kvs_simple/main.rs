use hydroflow::hydroflow_syntax;
use lattices::map_union::MapUnionHashMap;
use lattices::set_union::{SetUnionHashSet, SetUnionSingletonSet};
use lattices::{Merge, Pair, Top, Unit};

#[hydroflow::main]
pub async fn main() {
    type TwoPhaseSet<K> = Pair<SetUnionHashSet<K>, SetUnionHashSet<K>>;
    type TwoPhaseSet2<K> = MapUnionHashMap<K, Top<Unit>>;
    type TwoPhaseMap<K, Lat> = MapUnionHashMap<K, Top<Lat>>;

    let mut df = hydroflow_syntax! {
        incoming_requests = source_iter([Pair::new(SetUnionSingletonSet::new_from("Hello"), SetUnionHashSet::<&'static str>::default())])
            // -> lattice_merge::<SetUnionHashSet<&'static str>>()
            -> fold(TwoPhaseSet::default(), lattices::Merge::merge_owned)
            -> inspect(|x| println!("{x:?}"))

            -> null();
    };

    println!("{}", df.meta_graph().unwrap().to_mermaid());

    df.run_available();
}
