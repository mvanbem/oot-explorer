//! Provides a scoped owner that allows mixing borrowed data with freshly produced data.

use std::any::Any;
use std::cell::RefCell;

/// A scoped owner for arbitrary values.
///
/// `ScopedOwner` allows deeply nested functions that operate on borrowed data to ensure newly
/// produced data lives as long as an enclosing scope, permitting free mixing of initially borrowed
/// data with borrowed data that happened to be produced more recently.
pub struct ScopedOwner {
    // TODO: Remove one set of runtime checks by swapping in an UnsafeCell.
    values: RefCell<Vec<Box<dyn Any>>>,
}
impl ScopedOwner {
    pub fn with_scope<F, R>(f: F) -> R
    where
        F: FnOnce(&ScopedOwner) -> R,
    {
        // Safety: This ScopedOwner must retain all added values until after f() returns.
        let scope = ScopedOwner {
            values: RefCell::new(vec![]),
        };
        f(&scope)
    }

    pub fn add<T>(&self, value: T) -> &T
    where
        // TODO: Can this constraint be relaxed? Maybe if ScopedOwner took a lifetime parameter that
        // all added values had to outlive?
        T: 'static,
    {
        let mut values = self.values.borrow_mut();
        values.push(Box::new(value));
        let ptr_value: *const T = values.last().unwrap().downcast_ref::<T>().unwrap();

        // Safety: The value stored in that box in self.values must outlive the elided lifetime from
        // this function's declaration. That is ensured through two things:
        //
        // 1. &ScopedOwner is only produced in the body of with_scope(), where the borrow outlives
        //    the call to f().
        //
        // 2. Items are never removed from self.values, thus they live past the end of f().
        unsafe { std::mem::transmute(ptr_value) }
    }
}

#[cfg(test)]
mod tests {
    use super::ScopedOwner;

    #[test]
    fn usage() {
        ScopedOwner::with_scope(|scope| {
            process(scope, &[0, 1, 2, 3]);
        });
    }

    fn process(scope: &ScopedOwner, data: &[u8]) {
        let data = get_maybe_derived(scope, data, false);
        assert_eq!(data, &[0, 1, 2, 3]);
        assert_eq!(ThirdByte::new(data).get_third_byte(), 2);

        let derived = get_maybe_derived(scope, data, true);
        assert_eq!(derived, &[0x80, 0x81, 0x82, 0x83]);
        assert_eq!(ThirdByte::new(derived).get_third_byte(), 0x82);
    }

    // This is the fancy function enabled by this module. Allocate a
    // new Vec<u8>, but safely return &'a [u8]!
    fn get_maybe_derived<'a>(
        scope: &'a ScopedOwner,
        data: &'a [u8],
        get_derived: bool,
    ) -> &'a [u8] {
        if get_derived {
            let mut derived = vec![];
            for byte in data {
                derived.push(byte.wrapping_add(0x80));
            }
            scope.add(derived).as_slice()
        } else {
            data
        }
    }

    #[derive(Clone, Copy)]
    struct ThirdByte<'a> {
        data: &'a [u8],
    }
    impl<'a> ThirdByte<'a> {
        fn new(data: &'a [u8]) -> ThirdByte<'a> {
            ThirdByte { data }
        }
        fn get_third_byte(self) -> u8 {
            self.data[2]
        }
    }
}
