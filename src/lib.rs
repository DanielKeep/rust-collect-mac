/**
This macro provides a way to initialise any container for which there is a FromIterator implementation.  It allows for both sequence and map syntax to be used, as well as inline type ascription for the result.

For example:

```
# #[macro_use] extern crate collect_mac;
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
*/
#[macro_export]
macro_rules! collect {
    (@count_tts $($tts:tt)*) => {
        0usize $(+ collect!(@replace_expr $tts 1usize))*
    };

    (@replace_expr $_tt:tt $sub:expr) => {
        $sub
    };

    (@collect
        ty: $col_ty:ty,
        es: [$v0:expr, $($vs:expr),* $(,)*],
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
