/*
Copyright ⓒ 2015 Daniel Keep.

Licensed under the MIT license (see LICENSE or <http://opensource.org
/licenses/MIT>) or the Apache License, Version 2.0 (see LICENSE of
<http://www.apache.org/licenses/LICENSE-2.0>), at your option. All
files in the project carrying such notice may not be copied, modified,
or distributed except according to those terms.
*/
/*!
This test makes sure that `collect!` works as advertised for the various standard library collection types.  Where applicable, this also includes ensuring that only a single allocation is made.
*/

#[macro_use] extern crate collect_mac;

use std::any::Any;
use std::collections::{
    BinaryHeap,
    BTreeMap, BTreeSet,
    HashMap, HashSet,
    LinkedList,
    VecDeque,
};

/**
Check that two collections are equal by popping from them.
*/
macro_rules! assert_pop_eq {
    ($lhs:expr, $rhs:expr) => {
        {
            let mut i = 0;
            let mut lhs = $lhs;
            let mut rhs = $rhs;
            while let (Some(a), Some(b)) = (lhs.pop(), rhs.pop()) {
                assert_eq!((i, a), (i, b));
                i += 1;
            }
            assert_eq!(lhs.len(), 0);
            assert_eq!(rhs.len(), 0);
        }
    };
}

/**
Tries to ensure that the constructed collection allocates storage *exactly once*.
*/
macro_rules! check_growth {
    (
        ty: $col_ty:ty,
        es: $es:tt,
        eq: $eq:expr,
    ) => {
        check_growth!(
            ty: $col_ty,
            es: $es,
            eq: $eq,
            cmp: assert_eq,
        )
    };

    (
        #pop_eq
        ty: $col_ty:ty,
        es: $es:tt,
        eq: $eq:expr,
    ) => {
        check_growth!(
            ty: $col_ty,
            es: $es,
            eq: $eq,
            cmp: assert_pop_eq,
        )
    };

    (
        ty: $col_ty:ty,
        es: $es:tt,
        eq: $eq:expr,
        cmp: $cmp:ident,
    ) => {
        {
            // Construct the collection while checking the capacity at each step.
            let mut caps = vec![];
            let col = collect!(
                @collect
                ty: $col_ty,
                es: $es,
                cb: (col) { caps.push(col.capacity()); },
            );

            // Ensure that the collection is correct *and* the capacity goes: `[init_cap, final_cap, ...]`.
            let init_cap = <$col_ty>::new().capacity();
            let final_cap = col.capacity();

            assert_eq!(("caps[0]", caps[0]), ("caps[0]", init_cap));
            assert_eq!(
                ("caps[1..]", &caps[1..]),
                ("caps[1..]", &*vec![final_cap; col.len()])
            );

            $cmp!(col, $eq);
        }
    };
}

/**
Does a runtime type check, to avoid giving the type checker any additional hints (this is used to ensure that type hints provided to `collect!` work correctly).
*/
macro_rules! check_is {
    ($t:ty: $e:expr) => {
        {
            let v = $e;
            check_is::<$t, _>(&v);
            v
        }
    };
}

macro_rules! coerce {
    ($t:ty: $e:expr) => { { let v: $t = $e; v } };
}

#[test]
fn test_collect() {
    // Initialise an empty collection.
    let a: Vec<i32> = collect![];
    drop(a);
    let b: HashMap<String, bool> = collect![];
    drop(b);

    // Initialise a sequence.
    let c: String = collect!['a', 'b', 'c'];
    drop(c);

    // Initialise a sequence with a type constraint.
    let d = collect![as HashSet<_>: 0, 1, 2];
    drop(d);

    // Initialise a map collection.
    let e: HashMap<i32, &str> = collect![
        1 => "one",
        2 => "two",
        3 => "many",
        4 => "lots",
    ];
    drop(e);

    // Initialise a map with a type constraint.
    let f: HashMap<_, u8> = collect![as HashMap<i32, _>: 42 => 0, -11 => 2];
    drop(f);
}

#[test]
fn test_binary_heap() {
    let _: BinaryHeap<i32> = collect![];
    check_is!(BinaryHeap<i32>: collect![as BinaryHeap<i32>]);
    check_is!(BinaryHeap<i32>: collect![as BinaryHeap<i32>:]);

    macro_rules! mkcol {
        ($($tts:tt)*) => { vec![$($tts)*].into_iter().collect::<BinaryHeap<_>>() };
    }

    assert_pop_eq!(coerce!(BinaryHeap<_>: collect![0]), mkcol![0]);
    assert_pop_eq!(check_is!(BinaryHeap<i32>: collect![as BinaryHeap<_>: 0, 1]), mkcol![0, 1]);
    assert_pop_eq!(coerce!(BinaryHeap<_>: collect![0, 1, 2,]), mkcol![0, 1, 2]);

    check_growth!(
        #pop_eq
        ty: BinaryHeap<i32>,
        es: [1, 2, 3, 4, 5],
        eq: mkcol![1, 2, 3, 4, 5],
    );
}

#[test]
fn test_b_tree_map() {
    type Sstr = &'static str;
    let _: BTreeMap<Sstr, i32> = collect![];
    check_is!(BTreeMap<Sstr, i32>: collect![as BTreeMap<Sstr, i32>]);
    check_is!(BTreeMap<Sstr, i32>: collect![as BTreeMap<Sstr, i32>:]);

    macro_rules! mkcol {
        ($($tts:tt)*) => { vec![$($tts)*].into_iter().collect::<BTreeMap<_, _>>() };
    }

    assert_eq!(coerce!(BTreeMap<_, _>: collect!["hi" => 2]), mkcol![("hi", 2)]);
    assert_eq!(
        check_is!(BTreeMap<Sstr, i32>: collect![as BTreeMap<_, _>: "hi" => 2]),
        mkcol![("hi", 2)]
    );

    // Growth check does not apply.
}

#[test]
fn test_b_tree_set() {
    let _: BTreeSet<i32> = collect![];
    check_is!(BTreeSet<i32>: collect![as BTreeSet<i32>]);
    check_is!(BTreeSet<i32>: collect![as BTreeSet<i32>:]);

    macro_rules! mkcol {
        ($($tts:tt)*) => { vec![$($tts)*].into_iter().collect::<BTreeSet<_>>() };
    }

    assert_eq!(coerce!(BTreeSet<_>: collect![0]), mkcol![0]);
    assert_eq!(check_is!(BTreeSet<i32>: collect![as BTreeSet<_>: 0, 1]), mkcol![0, 1]);
    assert_eq!(coerce!(BTreeSet<_>: collect![0, 1, 2,]), mkcol![0, 1, 2]);

    // Growth check does not apply.
}

#[test]
fn test_hash_map() {
    type Sstr = &'static str;
    let _: HashMap<Sstr, i32> = collect![];
    check_is!(HashMap<Sstr, i32>: collect![as HashMap<Sstr, i32>]);
    check_is!(HashMap<Sstr, i32>: collect![as HashMap<Sstr, i32>:]);

    macro_rules! mkcol {
        ($($tts:tt)*) => { vec![$($tts)*].into_iter().collect::<HashMap<_, _>>() };
    }

    assert_eq!(coerce!(HashMap<_, _>: collect!["hi" => 2]), mkcol![("hi", 2)]);
    assert_eq!(
        check_is!(HashMap<Sstr, i32>: collect![as HashMap<_, _>: "hi" => 2]),
        mkcol![("hi", 2)]
    );

    check_growth!(
        ty: HashMap<Sstr, i32>,
        es: [("a", 1), ("b", 2), ("c", 3), ("d", 4), ("e", 5)],
        eq: mkcol![("a", 1), ("b", 2), ("c", 3), ("d", 4), ("e", 5)],
    );
}

#[test]
fn test_hash_set() {
    let _: HashSet<i32> = collect![];
    check_is!(HashSet<i32>: collect![as HashSet<i32>]);
    check_is!(HashSet<i32>: collect![as HashSet<i32>:]);

    macro_rules! mkcol {
        ($($tts:tt)*) => { vec![$($tts)*].into_iter().collect::<HashSet<_>>() };
    }

    assert_eq!(coerce!(HashSet<_>: collect![0]), mkcol![0]);
    assert_eq!(check_is!(HashSet<i32>: collect![as HashSet<_>: 0, 1]), mkcol![0, 1]);
    assert_eq!(coerce!(HashSet<_>: collect![0, 1, 2,]), mkcol![0, 1, 2]);

    check_growth!(
        ty: HashSet<i32>,
        es: [1, 2, 3, 4, 5],
        eq: mkcol![1, 2, 3, 4, 5],
    );
}

#[test]
fn test_linked_list() {
    let _: LinkedList<i32> = collect![];
    check_is!(LinkedList<i32>: collect![as LinkedList<i32>]);
    check_is!(LinkedList<i32>: collect![as LinkedList<i32>:]);

    macro_rules! mkcol {
        ($($tts:tt)*) => { vec![$($tts)*].into_iter().collect::<LinkedList<_>>() };
    }

    assert_eq!(coerce!(LinkedList<_>: collect![0]), mkcol![0]);
    assert_eq!(check_is!(LinkedList<i32>: collect![as LinkedList<_>: 0, 1]), mkcol![0, 1]);
    assert_eq!(coerce!(LinkedList<_>: collect![0, 1, 2,]), mkcol![0, 1, 2]);

    // Growth check does not apply.
}

#[test]
fn test_string() {
    let _: String = collect![];
    check_is!(String: collect![as String]);
    check_is!(String: collect![as String:]);

    assert_eq!(coerce!(String: collect!['x']), String::from("x"));
    assert_eq!(check_is!(String: collect![as String: 'x']), String::from("x"));
    assert_eq!(coerce!(String: collect!["one", "two"]), String::from("onetwo"));

    check_growth!(
        ty: String,
        es: ['1', '2', '3', '4', '5'],
        eq: String::from("12345"),
    );
}

#[test]
fn test_vec() {
    let _: Vec<i32> = collect![];
    check_is!(Vec<i32>: collect![as Vec<i32>]);
    check_is!(Vec<i32>: collect![as Vec<i32>:]);

    assert_eq!(coerce!(Vec<_>: collect![0]), vec![0]);
    assert_eq!(check_is!(Vec<i32>: collect![as Vec<_>: 0, 1]), vec![0, 1]);
    assert_eq!(coerce!(Vec<_>: collect![0, 1, 2,]), vec![0, 1, 2]);

    check_growth!(
        ty: Vec<i32>,
        es: [1, 2, 3, 4, 5],
        eq: vec![1, 2, 3, 4, 5],
    );
}

#[test]
fn test_vec_deque() {
    let _: VecDeque<i32> = collect![];
    check_is!(VecDeque<i32>: collect![as VecDeque<i32>]);
    check_is!(VecDeque<i32>: collect![as VecDeque<i32>:]);

    macro_rules! mkcol {
        ($($tts:tt)*) => { vec![$($tts)*].into_iter().collect::<VecDeque<_>>() };
    }

    assert_eq!(coerce!(VecDeque<_>: collect![0]), mkcol![0]);
    assert_eq!(check_is!(VecDeque<i32>: collect![as VecDeque<_>: 0, 1]), mkcol![0, 1]);
    assert_eq!(coerce!(VecDeque<_>: collect![0, 1, 2,]), mkcol![0, 1, 2]);

    check_growth!(
        ty: VecDeque<i32>,
        es: [1, 2, 3, 4, 5],
        eq: mkcol![1, 2, 3, 4, 5],
    );
}

fn check_is<T: Any, U: Any>(v: &U) {
    assert!(Any::is::<T>(v));
}
