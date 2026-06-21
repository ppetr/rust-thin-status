use std::cmp::Ordering;
use std::ffi::c_void;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::ptr::NonNull;
use triomphe::ThinArc;

/// Stores an `isize` as a tagged value inside a pointer. This means that one bit of `isize` isn't
/// available, and therefore inly numbers within `IsizeInPtr::MIN` and `IsizeIntPtr::MAX` are
/// convertible. This is expressed by `impl TryFrom<isize> for IsizeInPtr`.
#[derive(Eq, PartialEq, Ord, PartialOrd, Clone, Copy, Debug)]
pub struct IsizeInPtr {
    ptr: NonNull<c_void>,
}

impl IsizeInPtr {
    /// Maximal value that can be stored.
    pub const MAX: isize = isize::MAX >> 1;
    pub const MIN: isize = isize::MIN >> 1;
    const TAG_MASK: isize = 1;

    fn from_ptr(ptr: *const c_void) -> Option<Self> {
        if (ptr as isize & Self::TAG_MASK) == 0 {
            None
        } else {
            Some(unsafe { Self::new_unchecked(ptr) })
        }
    }

    fn from_isize_unchecked(value: isize) -> Self {
        let tagged = (value << 1) | Self::TAG_MASK;
        unsafe { Self::new_unchecked(tagged as *const c_void) }
    }

    unsafe fn new_unchecked(ptr: *const c_void) -> Self {
        IsizeInPtr {
            ptr: unsafe { NonNull::from_ref(&*ptr) },
        }
    }

    fn get(&self) -> isize {
        (self.ptr.as_ptr() as isize) >> 1
    }
}

impl Default for IsizeInPtr {
    fn default() -> Self {
        Self::from_isize_unchecked(0)
    }
}

impl From<IsizeInPtr> for isize {
    /// Convert `value` back to `isize`.
    fn from(value: IsizeInPtr) -> isize {
        value.get()
    }
}

impl TryFrom<isize> for IsizeInPtr {
    type Error = ();

    /// Wrap a `value` if it fits inside `IsizeInPtr`.
    fn try_from(value: isize) -> Result<IsizeInPtr, Self::Error> {
        if (value <= Self::MAX) && (value >= Self::MIN) {
            Ok(Self::from_isize_unchecked(value))
        } else {
            Err(())
        }
    }
}

macro_rules! impl_try_from_for_integral {
    ($($t:ty),*) => {
        $(
            impl TryFrom<$t> for IsizeInPtr {
                type Error = ();

                fn try_from(value: $t) -> Result<Self, Self::Error> {
                    isize::try_from(value)
                        .map_err(|_| ())
                        .and_then(|n: isize| IsizeInPtr::try_from(n))
                }
            }
        )*
    };
}
impl_try_from_for_integral!(i8, i16, i32, i64, i128);

/// A type representing either a signed pointer-sized integer (`isize`) or
/// a reference-counted pointer (`ThinArc<H, T>`).
///
/// Optimized using `NonNull` so that `Option<ThinArcOrInt<H, T>>` takes up exactly
/// the size of a single architecture pointer.
pub struct ThinArcOrInt<H, T> {
    raw: NonNull<c_void>,
    _marker: PhantomData<ThinArc<H, T>>,
}

unsafe impl<H, T> Send for ThinArcOrInt<H, T> where ThinArc<H, T>: Send {}
unsafe impl<H, T> Sync for ThinArcOrInt<H, T> where ThinArc<H, T>: Sync {}

impl<H, T> ThinArcOrInt<H, T> {
    /// Constructs an instance from a signed integer.
    pub fn from_isize(val: IsizeInPtr) -> Self {
        Self {
            raw: val.ptr,
            _marker: PhantomData,
        }
    }

    /// Constructs an instance from a ThinArc pointer.
    /// (Assumes the pointer is aligned and its LSB is 0.)
    pub fn from_arc(arc: ThinArc<H, T>) -> Self {
        let ptr = ThinArc::into_raw(arc);
        debug_assert!(
            IsizeInPtr::from_ptr(ptr).is_none(),
            "Pointer must be 2-aligned!"
        );
        // Safety: ThinArc allocations on the heap are never null.
        Self {
            raw: unsafe { NonNull::from_ref(&*ptr) },
            _marker: PhantomData,
        }
    }

    /// Tries to convert a `value` using `try_into()` to `IsizeInPtr`. If it succeeds, stores it as
    /// an integer inside the internal pointer. Otherwise it's stored in `ThinArc` as `H`.
    pub fn from_convertible<U: TryInto<IsizeInPtr, Error = H>>(value: U) -> Self {
        match value.try_into() {
            Ok(i) => Self::from_isize(i),
            Err(h) => Self::from_arc(ThinArc::from_header_and_iter(h, std::iter::empty())),
        }
    }

    pub fn has_number(&self) -> bool {
        IsizeInPtr::from_ptr(self.raw.as_ptr()).is_some()
    }

    pub fn has_ref(&self) -> bool {
        !self.has_number()
    }

    /// Returns the integer value if present, or `None` otherwise.
    pub fn as_isize(&self) -> Option<isize> {
        IsizeInPtr::from_ptr(self.raw.as_ptr()).map(|i| i.into())
    }

    /// Returns a shared reference to the `ThinArc` if present, or `None` otherwise.
    pub fn as_arc(&self) -> Option<&ThinArc<H, T>> {
        if self.has_ref() {
            unsafe { Some(self.as_arc_internal()) }
        } else {
            None
        }
    }

    unsafe fn as_arc_internal(&self) -> &ThinArc<H, T> {
        let ptr = &self.raw as *const NonNull<c_void> as *const ThinArc<H, T>;
        &*ptr
    }

    unsafe fn take_arc_internal(&mut self) -> ThinArc<H, T> {
        ThinArc::from_raw(self.raw.as_ptr() as *const c_void)
    }
}

impl<H, T> Default for ThinArcOrInt<H, T> {
    fn default() -> Self {
        Self::from_isize(Default::default())
    }
}

impl<H, T> Drop for ThinArcOrInt<H, T> {
    fn drop(&mut self) {
        if self.has_ref() {
            let _arc = unsafe { self.take_arc_internal() };
        }
    }
}

impl<H, T> Clone for ThinArcOrInt<H, T> {
    fn clone(&self) -> Self {
        if self.has_number() {
            Self {
                raw: self.raw,
                _marker: PhantomData,
            }
        } else {
            let arc = unsafe { self.as_arc_internal() };
            let cloned_arc = arc.clone();
            Self::from_arc(cloned_arc)
        }
    }
}

impl<H, T> PartialEq for ThinArcOrInt<H, T>
where
    H: PartialEq,
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        match (self.as_isize(), other.as_isize()) {
            (Some(s), Some(o)) => s == o,
            (None, None) => unsafe { self.as_arc_internal() == other.as_arc_internal() },
            _ => false,
        }
    }
}

impl<H, T> Eq for ThinArcOrInt<H, T>
where
    H: Eq,
    T: Eq,
{
}

impl<H, T> PartialOrd for ThinArcOrInt<H, T>
where
    H: PartialOrd,
    T: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self.as_isize(), other.as_isize()) {
            (Some(s), Some(o)) => s.partial_cmp(&o),
            (None, None) => unsafe { self.as_arc_internal().partial_cmp(other.as_arc_internal()) },
            (Some(_), None) => Some(Ordering::Less),
            (None, Some(_)) => Some(Ordering::Greater),
        }
    }
}

impl<H, T> Ord for ThinArcOrInt<H, T>
where
    H: Ord,
    T: Ord,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other)
            .expect("ThinArc::partial_cmp returned `None`")
    }
}

impl<H, T> Hash for ThinArcOrInt<H, T>
where
    H: Hash,
    T: Hash,
{
    fn hash<S: Hasher>(&self, state: &mut S) {
        if let Some(num) = self.as_isize() {
            0.hash(state);
            num.hash(state);
        } else {
            1.hash(state);
            unsafe { self.as_arc_internal().hash(state) };
        }
    }
}

impl<H: fmt::Debug, T: fmt::Debug> fmt::Debug for ThinArcOrInt<H, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(num) = self.as_isize() {
            f.debug_tuple("ThinArcOrInt::Number").field(&num).finish()
        } else {
            f.debug_tuple("ThinArcOrInt::ThinArc")
                .field(self.as_arc().unwrap())
                .finish()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn from_isize(value: isize) -> ThinArcOrInt<(), ()> {
        ThinArcOrInt::<(), ()>::from_isize(
            IsizeInPtr::try_from(value).expect("Out of allowed bounds"),
        )
    }

    #[test]
    fn test_size() {
        assert_eq!(
            std::mem::size_of::<ThinArcOrInt<(), String>>(),
            std::mem::size_of::<usize>()
        );
    }

    #[test]
    fn test_clone_and_drop() {
        let arc = ThinArc::from_header_and_slice((), "Shared data".as_bytes());
        let val1 = ThinArcOrInt::from_arc(arc);

        let val2 = val1.clone();

        assert_eq!(&val1.as_arc().unwrap().slice, "Shared data".as_bytes());
        assert!(std::ptr::eq(
            &val1.as_arc().unwrap().slice,
            &val2.as_arc().unwrap().slice
        ));
    }

    #[test]
    fn test_option_size_optimization() {
        assert_eq!(
            std::mem::size_of::<ThinArcOrInt<(), String>>(),
            std::mem::size_of::<usize>()
        );
        assert_eq!(
            std::mem::size_of::<Option<ThinArcOrInt<(), String>>>(),
            std::mem::size_of::<usize>()
        );
        assert_ne!(None, Some(from_isize(0)));
    }

    #[test]
    fn test_negative_numbers() {
        let negative = from_isize(-42);
        assert!(negative.has_number());
        assert_eq!(negative.as_isize(), Some(-42));
    }

    #[test]
    fn test_max_number() {
        let negative = from_isize(IsizeInPtr::MAX);
        assert!(negative.has_number());
        assert_eq!(negative.as_isize(), Some(IsizeInPtr::MAX));
    }

    #[test]
    fn test_min_number() {
        let negative = from_isize(IsizeInPtr::MIN);
        assert!(negative.has_number());
        assert_eq!(negative.as_isize(), Some(IsizeInPtr::MIN));
    }

    #[test]
    fn test_thin_arc() {
        let arc = ThinArc::from_header_and_slice((), b"Hello Rust");
        let val = ThinArcOrInt::from_arc(arc);

        assert!(val.has_ref());
        assert!(!val.has_number());
        assert_eq!(
            &val.as_arc().expect("Must be Arc, not isize").slice,
            b"Hello Rust"
        );
    }
}
