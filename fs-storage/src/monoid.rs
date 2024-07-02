// Currently, we have three structures: Tags (HashSet), Properties (HashSet),
// Score (int). In fact, HashSet already implements a union function,
// so only a special function for integers is needed.
// CRDTs can be considered later when we need to add structures that require
// more powerful combine semantics.

// Trait defining a Monoid, which represents a mathematical structure with an
// identity element and an associative binary operation.
pub trait Monoid<V> {
    // Returns the neutral element of the monoid.
    fn neutral() -> V;

    // Combines two elements of the monoid into a single element.
    fn combine(a: &V, b: &V) -> V;

    // Combines multiple elements of the monoid into a single element.
    // Default implementation uses `neutral()` as the initial accumulator and
    // `combine()` for folding.
    fn combine_all<I: IntoIterator<Item = V>>(values: I) -> V {
        values
            .into_iter()
            .fold(Self::neutral(), |acc, val| Self::combine(&acc, &val))
    }
}

impl Monoid<i32> for i32 {
    fn neutral() -> i32 {
        0
    }

    fn combine(a: &i32, b: &i32) -> i32 {
        if a > b {
            *a
        } else {
            *b
        }
    }
}

impl Monoid<String> for String {
    fn neutral() -> String {
        String::new()
    }

    fn combine(a: &String, b: &String) -> String {
        let mut result = a.clone();
        result.push_str(b);
        result
    }
}
