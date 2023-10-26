use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::Hash;

use crate::test::cartesian_power;

trait Semiring<T>:
    Addition<T> + Multiply<T> + IsAdditiveIdentity<T> + IsMultiplicativeIdentity<T>
{
}

impl<T> Semiring<T> for T where
    T: Addition<T> + Multiply<T> + IsAdditiveIdentity<T> + IsMultiplicativeIdentity<T>
{
}

trait Addition<T> {
    fn add(&self, other: &T) -> Self;
}

trait IsAdditiveIdentity<T> {
    fn is_additive_identity(&self) -> bool;
}

trait Multiply<T> {
    fn multiply(&self, other: &T) -> Self;
}

trait IsMultiplicativeIdentity<T> {
    fn is_multiplicative_identity(&self) -> bool;
}

fn test_properties<T: Semiring<T> + PartialEq<T> + Debug>(items: &[T]) {
    // + should be commutative
    for [a, b] in cartesian_power(items) {
        assert_eq!(a.add(b), b.add(a), "`{:?}`, `{:?}`", a, b);
    }

    // associativity
    for [a, b, c] in cartesian_power(items) {
        // +
        assert_eq!(a.add(b).add(c), a.add(&b.add(c)), "`{:?}`, `{:?}`", a, b);

        // *
        assert_eq!(
            a.multiply(b).multiply(c),
            a.multiply(&b.multiply(c)),
            "`{:?}`, `{:?}`",
            a,
            b
        );
    }

    // identities
    {
        let additive_identity = items
            .iter()
            .find(|&x| IsAdditiveIdentity::is_additive_identity(x))
            .expect("Must specify an additive identity in the list of items");

        let multiplicative_identity = items
            .iter()
            .find(|&x| IsMultiplicativeIdentity::is_multiplicative_identity(x))
            .expect("Must specify a multiplicative identity in the list of items");

        for [a] in cartesian_power(items) {
            // additive identity
            assert_eq!(a.add(&additive_identity), *a, "`{:?}`", a);

            // multiplicative identity
            assert_eq!(a.multiply(&multiplicative_identity), *a, "`{:?}`", a);
            assert_eq!(multiplicative_identity.multiply(a), *a, "`{:?}`", a);
        }
    }

    // distribution
    {
        for [a, b, c] in cartesian_power(items) {
            assert_eq!(
                c.multiply(&a.add(b)),
                c.multiply(a).add(&c.multiply(b)),
                "`{:?}`, `{:?}`, `{:?}`",
                a,
                b,
                c
            );

            assert_eq!(
                a.add(b).multiply(c),
                a.multiply(c).add(&b.multiply(c)),
                "`{:?}`, `{:?}`, `{:?}`",
                a,
                b,
                c
            );
        }
    }

    // Absorption
    {
        let additive_identity = items
            .iter()
            .find(|&x| IsAdditiveIdentity::is_additive_identity(x))
            .expect("Must specify an additive identity in the list of items");

        for [a] in cartesian_power(items) {
            assert_eq!(
                a.multiply(additive_identity),
                *additive_identity,
                "`{:?}`",
                a,
            );

            assert_eq!(
                additive_identity.multiply(a),
                *additive_identity,
                "`{:?}`",
                a,
            );
        }
    }
}

#[derive(Debug)]
struct NaturalNumber(usize);

impl PartialEq for NaturalNumber {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Addition<NaturalNumber> for NaturalNumber {
    fn add(&self, other: &Self) -> Self {
        Self(self.0 + other.0)
    }
}

impl IsAdditiveIdentity<NaturalNumber> for NaturalNumber {
    fn is_additive_identity(&self) -> bool {
        self.0 == 0
    }
}

impl Multiply<NaturalNumber> for NaturalNumber {
    fn multiply(&self, other: &Self) -> Self {
        Self(self.0 * other.0)
    }
}

impl IsMultiplicativeIdentity<NaturalNumber> for NaturalNumber {
    fn is_multiplicative_identity(&self) -> bool {
        self.0 == 1
    }
}

#[derive(Debug)]
struct Booleans(bool);

impl PartialEq for Booleans {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Addition<Booleans> for Booleans {
    fn add(&self, other: &Self) -> Self {
        Self(self.0 || other.0)
    }
}

impl IsAdditiveIdentity<Booleans> for Booleans {
    fn is_additive_identity(&self) -> bool {
        self.0 == false
    }
}

impl Multiply<Booleans> for Booleans {
    fn multiply(&self, other: &Self) -> Self {
        Self(self.0 && other.0)
    }
}

impl IsMultiplicativeIdentity<Booleans> for Booleans {
    fn is_multiplicative_identity(&self) -> bool {
        self.0 == true
    }
}

#[derive(Debug)]
struct Subsets<T: Hash>(HashSet<T>);

impl<T: Hash + Eq> PartialEq for Subsets<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T: Hash + Eq + Clone> Addition<Subsets<T>> for Subsets<T> {
    fn add(&self, other: &Self) -> Self {
        Self(self.0.union(&other.0).cloned().collect())
    }
}

impl<T: Hash> IsAdditiveIdentity<Subsets<T>> for Subsets<T> {
    fn is_additive_identity(&self) -> bool {
        self.0.is_empty()
    }
}

impl<T: Hash + Eq + Clone> Multiply<Subsets<T>> for Subsets<T> {
    fn multiply(&self, other: &Self) -> Self {
        Self(self.0.intersection(&other.0).cloned().collect())
    }
}

impl<T: Hash> IsMultiplicativeIdentity<Subsets<T>> for Subsets<T> {
    fn is_multiplicative_identity(&self) -> bool {
        self.0 == true
    }
}

#[test]
fn test() {
    test_properties(&[
        NaturalNumber(0),
        NaturalNumber(1),
        NaturalNumber(2),
        NaturalNumber(3),
        NaturalNumber(4),
    ]);

    test_properties(&[Booleans(false), Booleans(true)]);
}
