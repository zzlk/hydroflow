use std::collections::HashSet;
use std::fmt::Debug;

use itertools::{EitherOrBoth, Itertools};

use crate::test::cartesian_power;

#[test]
fn test() {
    let items = &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13];

    associativity(items, u32::max);
    idempotency(items, u32::max);
    commutativity(items, u32::max);

    identity(items, u32::max, 0);
    inverse(items, u32::wrapping_add, 0, |x| 0u32.wrapping_sub(x));

    distributive(items, &u32::wrapping_add, &u32::wrapping_mul);

    absorbing_element(items, &u32::wrapping_mul, 0);

    ring(items, &u32::wrapping_add, &u32::wrapping_mul, 0, 1, &|x| {
        0u32.wrapping_sub(x)
    });

    semiring(items, &u32::wrapping_add, &u32::wrapping_mul, 0, 1);

    // semiring(
    //     &[
    //         HashSet::from([]),
    //         HashSet::from([0]),
    //         HashSet::from([1]),
    //         HashSet::from([0, 1]),
    //     ],
    //     &|x, y| x.union(&y).cloned().collect(),
    //     &|x, y| x.intersection(&y).cloned().collect(),
    //     HashSet::from([]),
    //     true,
    // );

    semiring(&[false, true], &|x, y| x | y, &|x, y| x & y, false, true);

    semiring(
        &[0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, f64::INFINITY],
        &|x, y| f64::min(x, y),
        &|x, y| x + y,
        f64::INFINITY,
        0.0,
    );

    semiring(
        &[0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, f64::NEG_INFINITY],
        &|x, y| f64::max(x, y),
        &|x, y| x + y,
        f64::NEG_INFINITY,
        0.0,
    );

    semiring(
        &[
            HashSet::from([]),
            HashSet::from(["".to_owned()]),
            HashSet::from(["a".to_owned()]),
            HashSet::from(["aa".to_owned(), "bb".to_owned()]),
            HashSet::from(["ab".to_owned(), "bb".to_owned(), "cc".to_owned()]),
            HashSet::from(["ba".to_owned()]),
            HashSet::from(["bb".to_owned()]),
        ],
        &|x, y| x.union(&y).cloned().collect(),
        &|x, y| {
            let mut new_set = HashSet::new();

            for a in x.iter() {
                for b in y.iter() {
                    new_set.insert(format!("{a}{b}"));
                }
            }

            new_set
        },
        HashSet::from([]),
        HashSet::from(["".to_owned()]),
    );

    // finish polynomial implementation
    // Make better cartesian product generator things so that it's easy to create "interesting" fuzzing sets.
    // what's going on with quasi groups? What is divisability?
    // wholistic aggregates?

    #[derive(Debug, Clone)]
    struct Polynomial {
        vec: Vec<usize>,
    }

    impl PartialEq for Polynomial {
        fn eq(&self, other: &Self) -> bool {
            self.vec.iter().zip_longest(&other.vec).all(|x| match x {
                EitherOrBoth::Both(a, b) => a == b,
                EitherOrBoth::Left(a) => *a == 0,
                EitherOrBoth::Right(b) => *b == 0,
            })
        }
    }

    impl Polynomial {
        fn new() -> Self {
            Self { vec: vec![] }
        }

        fn new_from(x: impl IntoIterator<Item = usize>) -> Self {
            Self {
                vec: x.into_iter().collect(),
            }
        }

        fn add(self, other: Self) -> Self {
            Polynomial {
                vec: self
                    .vec
                    .into_iter()
                    .zip_longest(other.vec.into_iter())
                    .map(|x| match x {
                        EitherOrBoth::Both(a, b) => a + b,
                        EitherOrBoth::Left(a) => a,
                        EitherOrBoth::Right(b) => b,
                    })
                    .collect(),
            }
        }

        fn mul(self, other: Self) -> Self {
            let mut vec = vec![0; self.vec.len() + other.vec.len()];

            for (index, i) in self.vec.iter().enumerate() {
                for (index2, j) in other.vec.iter().enumerate() {
                    vec[index + index2] += *i * *j;
                }
            }

            Polynomial { vec }
        }

        fn evaluate(&self, x: usize) -> usize {
            let mut ret = 0;

            for (index, c) in self.vec.iter().enumerate() {
                ret += c * x.pow(index as u32);
            }

            ret
        }
    }

    semiring(
        &[
            Polynomial::new_from([]),
            Polynomial::new_from([0]),
            Polynomial::new_from([1]),
            Polynomial::new_from([2]),
            Polynomial::new_from([0, 1]),
            Polynomial::new_from([1, 2]),
            Polynomial::new_from([0, 1, 2]),
            Polynomial::new_from([1, 2, 3]),
        ],
        &Polynomial::add,
        &Polynomial::mul,
        Polynomial::new(),
        Polynomial::new_from([1]),
    );
}

fn quasi_group<S: Debug + PartialEq + Clone, const N: usize>(
    items: &[S; N],
    f: &impl Fn(S, S) -> S,
    e: S,
    b: &impl Fn(S) -> S,
) {
    todo!();
}

fn divisability<S: Debug + PartialEq + Clone, const N: usize>(
    items: &[S; N],
    f: &impl Fn(S, S) -> S,
    b: &impl Fn(S) -> S,
) {
    todo!();
}

fn monoid<S: Debug + PartialEq + Clone, const N: usize>(
    items: &[S; N],
    f: &impl Fn(S, S) -> S,
    e: S,
) {
    semigroup(items, f);
    identity(items, f, e);
}

fn semigroup<S: Debug + PartialEq + Clone, const N: usize>(items: &[S; N], f: &impl Fn(S, S) -> S) {
    associativity(items, f);
}

fn semiring<S: Debug + PartialEq + Clone, const N: usize>(
    items: &[S; N],
    f: &impl Fn(S, S) -> S,
    g: &impl Fn(S, S) -> S,
    zero: S,
    one: S,
) {
    commutative_monoid(items, f, zero.clone());
    monoid(items, g, one.clone());

    absorbing_element(items, g, zero);

    distributive(items, f, g);
}

fn ring<S: Debug + PartialEq + Clone, const N: usize>(
    items: &[S; N],
    f: &impl Fn(S, S) -> S,
    g: &impl Fn(S, S) -> S,
    zero: S,
    one: S,
    b: &impl Fn(S) -> S,
) {
    semiring(items, f, g, zero.clone(), one);
    inverse(items, f, zero, b);
}

fn field<S: Debug + PartialEq + Clone, const N: usize>(
    items: &[S; N],
    f: &impl Fn(S, S) -> S,
    g: &impl Fn(S, S) -> S,
    zero: S,
    one: S,
    b: &impl Fn(S) -> S,
) {
    ring(items, f, g, zero.clone(), one.clone(), b);
    nonzero_inverse(items, f, one, zero, b);
}

fn commutative_monoid<S: Debug + PartialEq + Clone, const N: usize>(
    items: &[S; N],
    f: &impl Fn(S, S) -> S,
    zero: S,
) {
    monoid(items, f, zero);
    commutativity(items, f);
}

fn abelian_group<S: Debug + PartialEq + Clone, const N: usize>(
    items: &[S; N],
    f: &impl Fn(S, S) -> S,
    zero: S,
    b: &impl Fn(S) -> S,
) {
    group(items, f, zero, b);
    commutativity(items, f);
}

fn group<S: Debug + PartialEq + Clone, const N: usize>(
    items: &[S; N],
    f: &impl Fn(S, S) -> S,
    e: S,
    b: &impl Fn(S) -> S,
) {
    monoid(items, f, e.clone());
    inverse(items, f, e, b);
}

fn distributive<S: Debug + PartialEq + Clone, const N: usize>(
    items: &[S; N],
    f: &impl Fn(S, S) -> S,
    g: &impl Fn(S, S) -> S,
) {
    left_distributes(items, f, g);
    right_distributes(items, f, g);
}

fn left_distributes<S: Debug + PartialEq + Clone, const N: usize>(
    items: &[S; N],
    f: impl Fn(S, S) -> S,
    g: impl Fn(S, S) -> S,
) {
    for [a, b, c] in cartesian_power(items) {
        // a(b+c) = ab + ac
        assert_eq!(
            g(a.clone(), f(b.clone(), c.clone())),
            f(g(a.clone(), b.clone()), g(a.clone(), c.clone()))
        );
    }
}

fn right_distributes<S: Debug + PartialEq + Clone, const N: usize>(
    items: &[S; N],
    f: impl Fn(S, S) -> S,
    g: impl Fn(S, S) -> S,
) {
    for [a, b, c] in cartesian_power(items) {
        // (b+c)a = ba + ca
        assert_eq!(
            g(f(b.clone(), c.clone()), a.clone()),
            f(g(b.clone(), a.clone()), g(c.clone(), a.clone()))
        );
    }
}

fn absorbing_element<S: Debug + PartialEq + Clone, const N: usize>(
    items: &[S; N],
    f: impl Fn(S, S) -> S,
    z: S,
) {
    for [a] in cartesian_power(items) {
        // az = z
        assert_eq!(f(a.clone(), z.clone()), z.clone());

        // za = z
        assert_eq!(f(z.clone(), a.clone()), z.clone());
    }
}

fn inverse<S: Debug + PartialEq + Clone, const N: usize>(
    items: &[S; N],
    f: impl Fn(S, S) -> S,
    e: S,
    b: impl Fn(S) -> S,
) {
    // ∃b: ab = e, ba = e
    for [a] in cartesian_power(items) {
        assert_eq!(f(a.clone(), b(a.clone())), e);
        assert_eq!(f(b(a.clone()), a.clone()), e);
    }
}

fn nonzero_inverse<S: Debug + PartialEq + Clone, const N: usize>(
    items: &[S; N],
    f: impl Fn(S, S) -> S,
    e: S,
    zero: S,
    b: impl Fn(S) -> S,
) {
    // ∃b: ab = e, ba = e
    for [a] in cartesian_power(items) {
        if *a != zero {
            assert_eq!(f(a.clone(), b(a.clone())), e);
            assert_eq!(f(b(a.clone()), a.clone()), e);
        }
    }
}

fn identity<S: Debug + PartialEq + Clone, const N: usize>(
    items: &[S; N],
    f: impl Fn(S, S) -> S,
    e: S,
) {
    // ea = a, ae = a
    for [a] in cartesian_power(items) {
        assert_eq!(f(e.clone(), a.clone()), a.clone());
        assert_eq!(f(a.clone(), e.clone()), a.clone());
    }
}

fn associativity<S: Debug + PartialEq + Clone, const N: usize>(
    items: &[S; N],
    f: impl Fn(S, S) -> S,
) {
    // a(bc) = (ab)c
    for [a, b, c] in cartesian_power(items) {
        assert_eq!(
            f(a.clone(), f(b.clone(), c.clone())),
            f(f(a.clone(), b.clone()), c.clone())
        );
    }
}

fn commutativity<S: Debug + PartialEq + Clone, const N: usize>(
    items: &[S; N],
    f: impl Fn(S, S) -> S,
) {
    // xy = yx
    for [x, y] in cartesian_power(items) {
        assert_eq!(f(x.clone(), y.clone()), f(y.clone(), x.clone()));
    }
}

fn idempotency<S: Debug + PartialEq + Clone, const N: usize>(
    items: &[S; N],
    f: impl Fn(S, S) -> S,
) {
    // xx = x
    for [x] in cartesian_power(items) {
        assert_eq!(f(x.clone(), x.clone()), x.clone());
    }
}
