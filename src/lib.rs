/*!
This crate provides the `collect!` macro, which can be used to easily construct arbitrary collections, including `Vec`, `String`, and `HashMap`.  It also endeavours to construct the collection with a single allocation, where possible.

## Example

```
// In the crate root module:
#[macro_use] extern crate collect_mac;

# use std::collections::{HashMap, HashSet, BTreeMap};
# fn main() {
// Initialise an empty collection.
let a: Vec<i32> = collect![];
let b: HashMap<String, bool> = collect![];

// Initialise a sequence.
let c: String = collect!['a', 'b', 'c'];

// Initialise a sequence with a type constraint.
let d = collect![as HashSet<_>: 0, 1, 2];

// Initialise a map collection.
let e: BTreeMap<i32, &str> = collect![
    1 => "one",
    2 => "two",
    3 => "many",
    4 => "lots",
];

// Initialise a map with a type constraint.
let f: HashMap<_, u8> = collect![as HashMap<i32, _>: 42 => 0, -11 => 2];
# }
```

## Details

The macro supports any collection which implements both the [`Default`][Default] and [`Extend`][Extend] traits.  Specifically, it creates a new, empty collection using `Default`, then calls `Extend` once for each element.

Single-allocation construction is tested and guaranteed for the following standard containers:

* [`HashMap`](http://doc.rust-lang.org/std/collections/struct.HashMap.html)
* [`HashSet`](http://doc.rust-lang.org/std/collections/struct.HashSet.html)
* [`String`](http://doc.rust-lang.org/std/string/struct.String.html)
* [`Vec`](http://doc.rust-lang.org/std/vec/struct.Vec.html)
* [`VecDeque`](http://doc.rust-lang.org/std/collections/struct.VecDeque.html)

In general, single-allocation construction is done by providing the number of elements through the [`Iterator::size_hint`][Iterator::size_hint] of the *first* call to `Extend`.  The expectation is that the collection will, if possible, pre-allocate enough space for all the elements when it goes to insert the first.

As an example, here is a simplified version of the `Extend` implementation for `Vec`:

```ignore
impl<T> Extend<T> for Vec<T> {
    #[inline]
    fn extend<I: IntoIterator<Item=T>>(&mut self, iterable: I) {
        let mut iterator = iterable.into_iter();
        while let Some(element) = iterator.next() {
            let len = self.len();
            if len == self.capacity() {
                let (lower, _) = iterator.size_hint();
                self.reserve(lower.saturating_add(1));
            }
            self.push(element);
        }
    }
}
```

[Default]: http://doc.rust-lang.org/std/default/trait.Default.html
[Extend]: http://doc.rust-lang.org/std/iter/trait.Extend.html
[Iterator::size_hint]: http://doc.rust-lang.org/std/iter/trait.Iterator.html#method.size_hint
*/

/**
This macro can be used to easily construct arbitrary collections, including `Vec`, `String`, and `HashMap`.  It also endeavours to construct the collection with a single allocation, where possible.

For more details, see [the crate documentation](./index.html).
*/
#[macro_export]
macro_rules! collect {
    /*
    Internal rules.
    */

    (@count_tts $($tts:tt)*) => {
        0usize $(+ collect!(@replace_expr $tts 1usize))*
    };

    (@replace_expr $_tt:tt $sub:expr) => {
        $sub
    };

    (@collect
        ty: $col_ty:ty,
        es: [$v0:expr, $($vs:expr),* $(,)*],
        // `cb` is an expression that is inserted after each "step" in constructing the collection.  It largely exists for testing purposes.
        cb: ($col:ident) $cb:expr,
    ) => {
        {
            const NUM_ELEMS: usize = collect!(@count_tts ($v0) $(($vs))*);

            let mut $col: $col_ty = ::std::default::Default::default();

            $cb;

            let hint = $crate::SizeHintIter {
                item: Some($v0),
                count: NUM_ELEMS
            };
            ::std::iter::Extend::extend(&mut $col, hint);

            $cb;

            $(
                ::std::iter::Extend::extend(&mut $col, Some($vs).into_iter());
                $cb;
            )*

            $col
        }
    };

    /*
    Public rules.
    */

    // Short-hands for initialising an empty collection.
    [] => {
        collect![as _:]
    };

    [as $col_ty:ty] => {
        collect![as $col_ty:]
    };

    [as $col_ty:ty:] => {
        {
            let col: $col_ty = ::std::default::Default::default();
            col
        }
    };

    // Initialise a sequence with a constrained container type.
    [as $col_ty:ty: $v0:expr] => { collect![as $col_ty: $v0,] };

    [as $col_ty:ty: $v0:expr, $($vs:expr),* $(,)*] => {
        collect!(
            @collect
            ty: $col_ty,
            es: [$v0, $($vs),*],
            cb: (col) (),
        )
    };

    // Initialise a map with a constrained container type.
    [as $col_ty:ty: $($ks:expr => $vs:expr),+ $(,)*] => {
        // Maps implement FromIterator by taking tuples, so we just need to rewrite each `a:b` as `(a,b)`.
        collect![as $col_ty: $(($ks, $vs)),+]
    };

    // Initialise a sequence with a fully inferred contained type.
    [$($vs:expr),+ $(,)*] => {
        collect![as _: $($vs),+]
    };

    // Initialise a map with a fully inferred contained type.
    [$($ks:expr => $vs:expr),+ $(,)*] => {
        collect![as _: $($ks => $vs),+]
    };
}

/**
This iterator's whole purpose in life is to lie whenever it's asked how many items it has.

This is necessary because of how `Extend` is implemented for `Vec`: specifically, it asks for the first element *before* it checks `size_hint`.  As a result, the old trick (of having an empty iterator that reported a false size hint) doesn't work.
*/
#[doc(hidden)]
pub struct SizeHintIter<T> {
    pub item: Option<T>,
    pub count: usize,
}

impl<T> Iterator for SizeHintIter<T> {
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<T> {
        match self.item.take() {
            Some(v) => {
                self.count -= 1;
                Some(v)
            },
            None => None
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.count, Some(self.count))
    }
}
