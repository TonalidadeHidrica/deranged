#![cfg_attr(docs_rs, feature(doc_auto_cfg))]
#![cfg_attr(not(feature = "std"), no_std)]
#![deny(
    anonymous_parameters,
    clippy::all,
    clippy::missing_safety_doc,
    clippy::missing_safety_doc,
    clippy::undocumented_unsafe_blocks,
    const_err,
    illegal_floating_point_literal_pattern,
    late_bound_lifetime_arguments,
    path_statements,
    patterns_in_fns_without_body,
    rust_2018_idioms,
    trivial_casts,
    trivial_numeric_casts,
    unreachable_pub,
    unsafe_op_in_unsafe_fn,
    unused_extern_crates
)]
#![warn(
    clippy::dbg_macro,
    clippy::decimal_literal_representation,
    clippy::get_unwrap,
    clippy::nursery,
    clippy::pedantic,
    clippy::todo,
    clippy::unimplemented,
    clippy::unwrap_used,
    clippy::use_debug,
    missing_copy_implementations,
    missing_debug_implementations,
    unused_qualifications,
    variant_size_differences
)]
#![allow(
    clippy::inline_always,
    clippy::missing_errors_doc,
    clippy::must_use_candidate,
    clippy::redundant_pub_crate
)]
#![doc(test(attr(deny(warnings))))]

use core::borrow::Borrow;
use core::cmp::Ordering;
use core::fmt;
use core::num::IntErrorKind;
use core::str::FromStr;
#[cfg(feature = "std")]
use std::error::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TryFromIntError;

impl fmt::Display for TryFromIntError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("out of range integral type conversion attempted")
    }
}
#[cfg(feature = "std")]
impl Error for TryFromIntError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseIntError {
    kind: IntErrorKind,
}

impl ParseIntError {
    /// Outputs the detailed cause of parsing an integer failing.
    // This function is not const because the counterpart of stdlib isn't
    #[allow(clippy::missing_const_for_fn)]
    #[inline(always)]
    pub fn kind(&self) -> &IntErrorKind {
        &self.kind
    }
}

impl fmt::Display for ParseIntError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            IntErrorKind::Empty => "cannot parse integer from empty string",
            IntErrorKind::InvalidDigit => "invalid digit found in string",
            IntErrorKind::PosOverflow => "number too large to fit in target type",
            IntErrorKind::NegOverflow => "number too small to fit in target type",
            IntErrorKind::Zero => "number would be zero for non-zero type",
            _ => "Unknown Int error kind",
        }
        .fmt(f)
    }
}

#[cfg(feature = "std")]
impl Error for ParseIntError {}

macro_rules! const_try_opt {
    ($e:expr) => {
        match $e {
            Some(value) => value,
            None => return None,
        }
    };
}

macro_rules! if_signed {
    (true $($x:tt)*) => { $($x)*};
    (false $($x:tt)*) => {};
}

macro_rules! article {
    (true) => {
        "An"
    };
    (false) => {
        "A"
    };
}

macro_rules! unsafe_unwrap_unchecked {
    ($e:expr) => {{
        let opt = $e;
        debug_assert!(opt.is_some());
        match $e {
            Some(value) => value,
            None => core::hint::unreachable_unchecked(),
        }
    }};
}

macro_rules! assume {
    ($e:expr) => {{
        let val = $e;
        debug_assert!(val);
        if !val {
            core::hint::unreachable_unchecked()
        }
    }};
}

macro_rules! impl_ranged {
    ($(
        $type:ident {
            mod_name: $mod_name:ident
            old_type: $old_type:ident
            internal: $internal:ident
            signed: $is_signed:ident
            into: [$($into:ident),* $(,)?]
            try_into: [$($try_into:ident),* $(,)?]
            try_from: [$($try_from:ident),* $(,)?]
        }
    )*) => {$(
        #[deprecated = concat!("renamed to `", stringify!($type), "`")]
        pub type $old_type<const MIN: $internal, const MAX: $internal> = $type<MIN, MAX>;

        pub use $mod_name::$type;

        // Introduce the type in a module. This ensures that all accesses and mutations of the field
        // have the necessary checks.
        mod $mod_name {
            #[doc = concat!(
                article!($is_signed),
                " `",
                stringify!($internal),
                "` that is known to be in the range `MIN..=MAX`.",
            )]
            #[repr(transparent)]
            #[derive(Clone, Copy, Eq, Ord, Hash)]
            pub struct $type<const MIN: $internal, const MAX: $internal>(
                $internal,
            );

            impl<const MIN: $internal, const MAX: $internal> $type<MIN, MAX> {
                /// Creates a ranged integer without checking the value.
                ///
                /// # Safety
                ///
                /// The value must be within the range `MIN..=MAX`.
                #[inline(always)]
                pub const unsafe fn new_unchecked(value: $internal) -> Self {
                    // Safety: The caller must ensure that the value is in range.
                    unsafe { assume!(MIN <= value && value <= MAX) };
                    Self(value)
                }

                /// Creates a ranged integer if the given value is in the range
                /// `MIN..=MAX`.
                #[inline(always)]
                pub const fn new(value: $internal) -> Option<Self> {
                    if value < MIN || value > MAX {
                        None
                    } else {
                        Some(Self(value))
                    }
                }

                /// Returns the value as a primitive type.
                #[inline(always)]
                pub const fn get(self) -> $internal {
                    // Safety: A stored value is always in range.
                    unsafe { assume!(MIN <= self.0 && self.0 <= MAX) };
                    self.0
                }

                #[inline(always)]
                pub(crate) const fn get_ref(&self) -> &$internal {
                    // Safety: A stored value is always in range.
                    unsafe { assume!(MIN <= self.0 && self.0 <= MAX) };
                    &self.0
                }
            }
        }

        impl<const MIN: $internal, const MAX: $internal> $type<MIN, MAX> {
            /// The smallest value that can be represented by this type.
            // Safety: `MIN` is in range by definition.
            pub const MIN: Self = unsafe { Self::new_unchecked(MIN) };

            /// The largest value that can be represented by this type.
            // Safety: `MAX` is in range by definition.
            pub const MAX: Self = unsafe { Self::new_unchecked(MAX) };

            #[inline]
            const fn new_saturating(value: $internal) -> Self {
                // Safety: The value is clamped to the range.
                unsafe {
                    Self::new_unchecked(if value < MIN {
                        MIN
                    } else if value > MAX {
                        MAX
                    } else {
                        value
                    })
                }
            }

            /// Checked integer addition. Computes `self + rhs`, returning `None` if the resulting
            /// value is out of range.
            #[must_use = "this returns the result of the operation, without modifying the original"]
            #[inline]
            pub const fn checked_add(self, rhs: $internal) -> Option<Self> {
                Self::new(const_try_opt!(self.get().checked_add(rhs)))
            }

            /// Unchecked integer addition. Computes `self + rhs`, assuming that the result is in
            /// range.
            ///
            /// # Safety
            ///
            /// The result of `self + rhs` must be in the range `MIN..=MAX`.
            #[must_use = "this returns the result of the operation, without modifying the original"]
            #[inline(always)]
            pub const unsafe fn unchecked_add(self, rhs: $internal) -> Self {
                // Safety: The caller must ensure that the result is in range.
                unsafe {
                    Self::new_unchecked(unsafe_unwrap_unchecked!(self.get().checked_add(rhs)))
                }
            }

            /// Checked integer addition. Computes `self - rhs`, returning `None` if the resulting
            /// value is out of range.
            #[must_use = "this returns the result of the operation, without modifying the original"]
            #[inline]
            pub const fn checked_sub(self, rhs: $internal) -> Option<Self> {
                Self::new(const_try_opt!(self.get().checked_sub(rhs)))
            }

            /// Unchecked integer subtraction. Computes `self - rhs`, assuming that the result is in
            /// range.
            ///
            /// # Safety
            ///
            /// The result of `self - rhs` must be in the range `MIN..=MAX`.
            #[must_use = "this returns the result of the operation, without modifying the original"]
            #[inline(always)]
            pub const unsafe fn unchecked_sub(self, rhs: $internal) -> Self {
                // Safety: The caller must ensure that the result is in range.
                unsafe {
                    Self::new_unchecked(unsafe_unwrap_unchecked!(self.get().checked_sub(rhs)))
                }
            }

            /// Checked integer addition. Computes `self * rhs`, returning `None` if the resulting
            /// value is out of range.
            #[must_use = "this returns the result of the operation, without modifying the original"]
            #[inline]
            pub const fn checked_mul(self, rhs: $internal) -> Option<Self> {
                Self::new(const_try_opt!(self.get().checked_mul(rhs)))
            }

            /// Unchecked integer multiplication. Computes `self * rhs`, assuming that the result is
            /// in range.
            ///
            /// # Safety
            ///
            /// The result of `self * rhs` must be in the range `MIN..=MAX`.
            #[must_use = "this returns the result of the operation, without modifying the original"]
            #[inline(always)]
            pub const unsafe fn unchecked_mul(self, rhs: $internal) -> Self {
                // Safety: The caller must ensure that the result is in range.
                unsafe {
                    Self::new_unchecked(unsafe_unwrap_unchecked!(self.get().checked_mul(rhs)))
                }
            }

            /// Checked integer addition. Computes `self / rhs`, returning `None` if `rhs == 0` or
            /// if the resulting value is out of range.
            #[must_use = "this returns the result of the operation, without modifying the original"]
            #[inline]
            pub const fn checked_div(self, rhs: $internal) -> Option<Self> {
                Self::new(const_try_opt!(self.get().checked_div(rhs)))
            }

            /// Unchecked integer division. Computes `self / rhs`, assuming that `rhs != 0` and that
            /// the result is in range.
            ///
            /// # Safety
            ///
            /// `self` must not be zero and the result of `self / rhs` must be in the range
            /// `MIN..=MAX`.
            #[must_use = "this returns the result of the operation, without modifying the original"]
            #[inline(always)]
            pub const unsafe fn unchecked_div(self, rhs: $internal) -> Self {
                // Safety: The caller must ensure that the result is in range and that `rhs` is not
                // zero.
                unsafe {
                    Self::new_unchecked(unsafe_unwrap_unchecked!(self.get().checked_div(rhs)))
                }
            }

            /// Checked Euclidean division. Computes `self.div_euclid(rhs)`, returning `None` if
            /// `rhs == 0` or if the resulting value is out of range.
            #[must_use = "this returns the result of the operation, without modifying the original"]
            #[inline]
            pub const fn checked_div_euclid(self, rhs: $internal) -> Option<Self> {
                Self::new(const_try_opt!(self.get().checked_div_euclid(rhs)))
            }

            /// Unchecked Euclidean division. Computes `self.div_euclid(rhs)`, assuming that
            /// `rhs != 0` and that the result is in range.
            ///
            /// # Safety
            ///
            /// `self` must not be zero and the result of `self.div_euclid(rhs)` must be in the
            /// range `MIN..=MAX`.
            #[must_use = "this returns the result of the operation, without modifying the original"]
            #[inline(always)]
            pub const unsafe fn unchecked_div_euclid(self, rhs: $internal) -> Self {
                // Safety: The caller must ensure that the result is in range and that `rhs` is not
                // zero.
                unsafe {
                    Self::new_unchecked(
                        unsafe_unwrap_unchecked!(self.get().checked_div_euclid(rhs))
                    )
                }
            }

            /// Checked integer remainder. Computes `self % rhs`, returning `None` if `rhs == 0` or
            /// if the resulting value is out of range.
            #[must_use = "this returns the result of the operation, without modifying the original"]
            #[inline]
            pub const fn checked_rem(self, rhs: $internal) -> Option<Self> {
                Self::new(const_try_opt!(self.get().checked_rem(rhs)))
            }

            /// Unchecked remainder. Computes `self % rhs`, assuming that `rhs != 0` and that the
            /// result is in range.
            ///
            /// # Safety
            ///
            /// `self` must not be zero and the result of `self % rhs` must be in the range
            /// `MIN..=MAX`.
            #[must_use = "this returns the result of the operation, without modifying the original"]
            #[inline(always)]
            pub const unsafe fn unchecked_rem(self, rhs: $internal) -> Self {
                // Safety: The caller must ensure that the result is in range and that `rhs` is not
                // zero.
                unsafe {
                    Self::new_unchecked(unsafe_unwrap_unchecked!(self.get().checked_rem(rhs)))
                }
            }

            /// Checked Euclidean remainder. Computes `self.rem_euclid(rhs)`, returning `None` if
            /// `rhs == 0` or if the resulting value is out of range.
            #[must_use = "this returns the result of the operation, without modifying the original"]
            #[inline]
            pub const fn checked_rem_euclid(self, rhs: $internal) -> Option<Self> {
                Self::new(const_try_opt!(self.get().checked_rem_euclid(rhs)))
            }

            /// Unchecked Euclidean remainder. Computes `self.rem_euclid(rhs)`, assuming that
            /// `rhs != 0` and that the result is in range.
            ///
            /// # Safety
            ///
            /// `self` must not be zero and the result of `self.rem_euclid(rhs)` must be in the
            /// range `MIN..=MAX`.
            #[must_use = "this returns the result of the operation, without modifying the original"]
            #[inline(always)]
            pub const unsafe fn unchecked_rem_euclid(self, rhs: $internal) -> Self {
                // Safety: The caller must ensure that the result is in range and that `rhs` is not
                // zero.
                unsafe {
                    Self::new_unchecked(
                        unsafe_unwrap_unchecked!(self.get().checked_rem_euclid(rhs))
                    )
                }
            }

            /// Checked negation. Computes `-self`, returning `None` if the resulting value is out
            /// of range.
            #[must_use = "this returns the result of the operation, without modifying the original"]
            #[inline]
            pub const fn checked_neg(self) -> Option<Self> {
                Self::new(const_try_opt!(self.get().checked_neg()))
            }

            /// Unchecked negation. Computes `-self`, assuming that `-self` is in range.
            ///
            /// # Safety
            ///
            /// The result of `-self` must be in the range `MIN..=MAX`.
            #[must_use = "this returns the result of the operation, without modifying the original"]
            #[inline(always)]
            pub const unsafe fn unchecked_neg(self) -> Self {
                // Safety: The caller must ensure that the result is in range.
                unsafe { Self::new_unchecked(unsafe_unwrap_unchecked!(self.get().checked_neg())) }
            }

            /// Checked shift left. Computes `self << rhs`, returning `None` if the resulting value
            /// is out of range.
            #[must_use = "this returns the result of the operation, without modifying the original"]
            #[inline]
            pub const fn checked_shl(self, rhs: u32) -> Option<Self> {
                Self::new(const_try_opt!(self.get().checked_shl(rhs)))
            }

            /// Unchecked shift left. Computes `self << rhs`, assuming that the result is in range.
            ///
            /// # Safety
            ///
            /// The result of `self << rhs` must be in the range `MIN..=MAX`.
            #[must_use = "this returns the result of the operation, without modifying the original"]
            #[inline(always)]
            pub const unsafe fn unchecked_shl(self, rhs: u32) -> Self {
                // Safety: The caller must ensure that the result is in range.
                unsafe {
                    Self::new_unchecked(unsafe_unwrap_unchecked!(self.get().checked_shl(rhs)))
                }
            }

            /// Checked shift right. Computes `self >> rhs`, returning `None` if
            /// the resulting value is out of range.
            #[must_use = "this returns the result of the operation, without modifying the original"]
            #[inline]
            pub const fn checked_shr(self, rhs: u32) -> Option<Self> {
                Self::new(const_try_opt!(self.get().checked_shr(rhs)))
            }

            /// Unchecked shift right. Computes `self >> rhs`, assuming that the result is in range.
            ///
            /// # Safety
            ///
            /// The result of `self >> rhs` must be in the range `MIN..=MAX`.
            #[must_use = "this returns the result of the operation, without modifying the original"]
            #[inline(always)]
            pub const unsafe fn unchecked_shr(self, rhs: u32) -> Self {
                // Safety: The caller must ensure that the result is in range.
                unsafe {
                    Self::new_unchecked(unsafe_unwrap_unchecked!(self.get().checked_shr(rhs)))
                }
            }

            if_signed!($is_signed
            /// Checked absolute value. Computes `self.abs()`, returning `None` if the resulting
            /// value is out of range.
            #[must_use = "this returns the result of the operation, without modifying the original"]
            #[inline]
            pub const fn checked_abs(self) -> Option<Self> {
                Self::new(const_try_opt!(self.get().checked_abs()))
            }

            /// Unchecked absolute value. Computes `self.abs()`, assuming that the result is in
            /// range.
            ///
            /// # Safety
            ///
            /// The result of `self.abs()` must be in the range `MIN..=MAX`.
            #[must_use = "this returns the result of the operation, without modifying the original"]
            #[inline(always)]
            pub const unsafe fn unchecked_abs(self) -> Self {
                // Safety: The caller must ensure that the result is in range.
                unsafe { Self::new_unchecked(unsafe_unwrap_unchecked!(self.get().checked_abs())) }
            });

            /// Checked exponentiation. Computes `self.pow(exp)`, returning `None` if the resulting
            /// value is out of range.
            #[must_use = "this returns the result of the operation, without modifying the original"]
            #[inline]
            pub const fn checked_pow(self, exp: u32) -> Option<Self> {
                Self::new(const_try_opt!(self.get().checked_pow(exp)))
            }

            /// Unchecked exponentiation. Computes `self.pow(exp)`, assuming that the result is in
            /// range.
            ///
            /// # Safety
            ///
            /// The result of `self.pow(exp)` must be in the range `MIN..=MAX`.
            #[must_use = "this returns the result of the operation, without modifying the original"]
            #[inline(always)]
            pub const unsafe fn unchecked_pow(self, exp: u32) -> Self {
                // Safety: The caller must ensure that the result is in range.
                unsafe {
                    Self::new_unchecked(unsafe_unwrap_unchecked!(self.get().checked_pow(exp)))
                }
            }

            /// Saturating integer addition. Computes `self + rhs`, saturating at the numeric
            /// bounds.
            #[must_use = "this returns the result of the operation, without modifying the original"]
            #[inline]
            pub const fn saturating_add(self, rhs: $internal) -> Self {
                Self::new_saturating(self.get().saturating_add(rhs))
            }

            /// Saturating integer subtraction. Computes `self - rhs`, saturating at the numeric
            /// bounds.
            #[must_use = "this returns the result of the operation, without modifying the original"]
            #[inline]
            pub const fn saturating_sub(self, rhs: $internal) -> Self {
                Self::new_saturating(self.get().saturating_sub(rhs))
            }

            if_signed!($is_signed
            /// Saturating integer negation. Computes `self - rhs`, saturating at the numeric
            /// bounds.
            #[must_use = "this returns the result of the operation, without modifying the original"]
            #[inline]
            pub const fn saturating_neg(self) -> Self {
                Self::new_saturating(self.get().saturating_neg())
            });

            if_signed!($is_signed
            /// Saturating absolute value. Computes `self.abs()`, saturating at the numeric bounds.
            #[must_use = "this returns the result of the operation, without modifying the original"]
            #[inline]
            pub const fn saturating_abs(self) -> Self {
                Self::new_saturating(self.get().saturating_abs())
            });

            /// Saturating integer multiplication. Computes `self * rhs`, saturating at the numeric
            /// bounds.
            #[must_use = "this returns the result of the operation, without modifying the original"]
            #[inline]
            pub const fn saturating_mul(self, rhs: $internal) -> Self {
                Self::new_saturating(self.get().saturating_mul(rhs))
            }

            /// Saturating integer exponentiation. Computes `self.pow(exp)`, saturating at the
            /// numeric bounds.
            #[must_use = "this returns the result of the operation, without modifying the original"]
            #[inline]
            pub const fn saturating_pow(self, exp: u32) -> Self {
                Self::new_saturating(self.get().saturating_pow(exp))
            }
        }

        impl<const MIN: $internal, const MAX: $internal> fmt::Debug for $type<MIN, MAX> {
            #[inline(always)]
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.get().fmt(f)
            }
        }

        impl<const MIN: $internal, const MAX: $internal> fmt::Display for $type<MIN, MAX> {
            #[inline(always)]
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.get().fmt(f)
            }
        }

        impl<const MIN: $internal, const MAX: $internal> AsRef<$internal> for $type<MIN, MAX> {
            #[inline(always)]
            fn as_ref(&self) -> &$internal {
                &self.get_ref()
            }
        }

        impl<const MIN: $internal, const MAX: $internal> Borrow<$internal> for $type<MIN, MAX> {
            #[inline(always)]
            fn borrow(&self) -> &$internal {
                &self.get_ref()
            }
        }

        impl<
            const MIN_A: $internal,
            const MAX_A: $internal,
            const MIN_B: $internal,
            const MAX_B: $internal,
        > PartialEq<$type<MIN_B, MAX_B>> for $type<MIN_A, MAX_A> {
            #[inline(always)]
            fn eq(&self, other: &$type<MIN_B, MAX_B>) -> bool {
                self.get() == other.get()
            }
        }

        impl<const MIN: $internal, const MAX: $internal> PartialEq<$internal> for $type<MIN, MAX> {
            #[inline(always)]
            fn eq(&self, other: &$internal) -> bool {
                self.get() == *other
            }
        }

        impl<const MIN: $internal, const MAX: $internal> PartialEq<$type<MIN, MAX>> for $internal {
            #[inline(always)]
            fn eq(&self, other: &$type<MIN, MAX>) -> bool {
                *self == other.get()
            }
        }

        impl<
            const MIN_A: $internal,
            const MAX_A: $internal,
            const MIN_B: $internal,
            const MAX_B: $internal,
        > PartialOrd<$type<MIN_B, MAX_B>> for $type<MIN_A, MAX_A> {
            #[inline(always)]
            fn partial_cmp(&self, other: &$type<MIN_B, MAX_B>) -> Option<Ordering> {
                self.get().partial_cmp(&other.get())
            }
        }

        impl<const MIN: $internal, const MAX: $internal> PartialOrd<$internal> for $type<MIN, MAX> {
            #[inline(always)]
            fn partial_cmp(&self, other: &$internal) -> Option<Ordering> {
                self.get().partial_cmp(other)
            }
        }

        impl<const MIN: $internal, const MAX: $internal> PartialOrd<$type<MIN, MAX>> for $internal {
            #[inline(always)]
            fn partial_cmp(&self, other: &$type<MIN, MAX>) -> Option<Ordering> {
                self.partial_cmp(&other.get())
            }
        }

        impl<const MIN: $internal, const MAX: $internal> fmt::Binary for $type<MIN, MAX> {
            #[inline(always)]
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.get().fmt(f)
            }
        }

        impl<const MIN: $internal, const MAX: $internal> fmt::LowerHex for $type<MIN, MAX> {
            #[inline(always)]
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.get().fmt(f)
            }
        }

        impl<const MIN: $internal, const MAX: $internal> fmt::UpperHex for $type<MIN, MAX> {
            #[inline(always)]
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.get().fmt(f)
            }
        }

        impl<const MIN: $internal, const MAX: $internal> fmt::LowerExp for $type<MIN, MAX> {
            #[inline(always)]
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.get().fmt(f)
            }
        }

        impl<const MIN: $internal, const MAX: $internal> fmt::UpperExp for $type<MIN, MAX> {
            #[inline(always)]
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.get().fmt(f)
            }
        }

        impl<const MIN: $internal, const MAX: $internal> fmt::Octal for $type<MIN, MAX> {
            #[inline(always)]
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.get().fmt(f)
            }
        }

        $(impl<const MIN: $internal, const MAX: $internal> From<$type<MIN,MAX>> for $into {
            #[inline(always)]
            fn from(value: $type<MIN, MAX>) -> Self {
                value.get().into()
            }
        })*

        $(impl<const MIN: $internal, const MAX: $internal> TryFrom<$type<MIN, MAX>> for $try_into {
            type Error = TryFromIntError;

            #[inline(always)]
            fn try_from(value: $type<MIN, MAX>) -> Result<Self, Self::Error> {
                value.get().try_into().map_err(|_| TryFromIntError)
            }
        })*

        $(impl<const MIN: $internal, const MAX: $internal> TryFrom<$try_from> for $type<MIN, MAX> {
            type Error = TryFromIntError;

            #[inline]
            fn try_from(value: $try_from) -> Result<Self, Self::Error> {
                let value = match TryInto::<$internal>::try_into(value) {
                    Ok(value) => value,
                    Err(_) => return Err(TryFromIntError)
                };

                if value < MIN || value > MAX {
                    Err(TryFromIntError)
                } else {
                    match TryFrom::try_from(value) {
                        // Safety: The value was previously checked for validity.
                        Ok(value) => Ok(unsafe { Self::new_unchecked(value) }),
                        Err(_) => Err(TryFromIntError),
                    }
                }
            }
        })*

        impl<const MIN: $internal, const MAX: $internal> FromStr for $type<MIN, MAX> {
            type Err = ParseIntError;

            #[inline]
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                let value = s.parse::<$internal>().map_err(|e| ParseIntError {
                    kind: e.kind().clone()
                })?;
                if value < MIN {
                    Err(ParseIntError { kind: IntErrorKind::NegOverflow })
                } else if value > MAX {
                    Err(ParseIntError { kind: IntErrorKind::PosOverflow })
                } else {
                    // Safety: The value was previously checked for validity.
                    Ok(unsafe { Self::new_unchecked(value) })
                }
            }
        }

        #[cfg(feature = "serde")]
        impl<const MIN: $internal, const MAX: $internal> serde::Serialize for $type<MIN, MAX> {
            #[inline(always)]
            fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                self.get().serialize(serializer)
            }
        }

        #[cfg(feature = "serde")]
        impl<
            'de,
            const MIN: $internal,
            const MAX: $internal,
        > serde::Deserialize<'de> for $type<MIN, MAX> {
            #[inline]
            fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                let internal = <$internal>::deserialize(deserializer)?;
                Self::new(internal).ok_or_else(|| <D::Error as serde::de::Error>::invalid_value(
                    serde::de::Unexpected::Other("integer"),
                    &format!("an integer in the range {}..={}", MIN, MAX).as_ref()
                ))
            }
        }

        #[cfg(feature = "rand")]
        impl<
            const MIN: $internal,
            const MAX: $internal,
        > rand::distributions::Distribution<$type<MIN, MAX>> for rand::distributions::Standard {
            #[inline]
            fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> $type<MIN, MAX> {
                $type::new(rng.gen_range(MIN..=MAX)).expect("rand failed to generate a valid value")
            }
        }

        #[cfg(feature = "num")]
        impl<const MIN: $internal, const MAX: $internal> num_traits::Bounded for $type<MIN, MAX> {
            #[inline(always)]
            fn min_value() -> Self {
                Self::MIN
            }

            #[inline(always)]
            fn max_value() -> Self {
                Self::MAX
            }
        }

        #[cfg(feature = "quickcheck")]
        impl<const MIN: $internal, const MAX: $internal> quickcheck::Arbitrary for $type<MIN, MAX> {
            #[inline]
            fn arbitrary(g: &mut quickcheck::Gen) -> Self {
                // Safety: The `rem_euclid` call and addition ensure that the value is in range.
                unsafe {
                    Self::new_unchecked($internal::arbitrary(g).rem_euclid(MAX - MIN + 1) + MIN)
                }
            }
        }
    )*};
}

impl_ranged! {
    RangedU8 {
        mod_name: ranged_u8
        old_type: U8
        internal: u8
        signed: false
        into: [u8, u16, u32, u64, u128, usize, i16, i32, i64, i128, isize]
        try_into: [i8]
        try_from: [u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize]
    }
    RangedU16 {
        mod_name: ranged_u16
        old_type: U16
        internal: u16
        signed: false
        into: [u16, u32, u64, u128, usize, i32, i64, i128]
        try_into: [u8, i8, i16, isize]
        try_from: [u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize]
    }
    RangedU32 {
        mod_name: ranged_u32
        old_type: U32
        internal: u32
        signed: false
        into: [u32, u64, u128, i64, i128]
        try_into: [u8, u16, usize, i8, i16, i32, isize]
        try_from: [u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize]
    }
    RangedU64 {
        mod_name: ranged_u64
        old_type: U64
        internal: u64
        signed: false
        into: [u64, u128, i128]
        try_into: [u8, u16, u32, usize, i8, i16, i32, i64, isize]
        try_from: [u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize]
    }
    RangedU128 {
        mod_name: ranged_u128
        old_type: U128
        internal: u128
        signed: false
        into: [u128]
        try_into: [u8, u16, u32, u64, usize, i8, i16, i32, i64, i128, isize]
        try_from: [u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize]
    }
    RangedUsize {
        mod_name: ranged_usize
        old_type: Usize
        internal: usize
        signed: false
        into: [usize]
        try_into: [u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, isize]
        try_from: [u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize]
    }
    RangedI8 {
        mod_name: ranged_i8
        old_type: I8
        internal: i8
        signed: true
        into: [i8, i16, i32, i64, i128, isize]
        try_into: [u8, u16, u32, u64, u128, usize]
        try_from: [u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize]
    }
    RangedI16 {
        mod_name: ranged_i16
        old_type: I16
        internal: i16
        signed: true
        into: [i16, i32, i64, i128, isize]
        try_into: [u8, u16, u32, u64, u128, usize, i8]
        try_from: [u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize]
    }
    RangedI32 {
        mod_name: ranged_i32
        old_type: I32
        internal: i32
        signed: true
        into: [i32, i64, i128]
        try_into: [u8, u16, u32, u64, u128, usize, i8, i16, isize]
        try_from: [u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize]
    }
    RangedI64 {
        mod_name: ranged_i64
        old_type: I64
        internal: i64
        signed: true
        into: [i64, i128]
        try_into: [u8, u16, u32, u64, u128, usize, i8, i16, i32, isize]
        try_from: [u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize]
    }
    RangedI128 {
        mod_name: ranged_i128
        old_type: I128
        internal: i128
        signed: true
        into: [i128]
        try_into: [u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, isize]
        try_from: [u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize]
    }
    RangedIsize {
        mod_name: ranged_isize
        old_type: Isize
        internal: isize
        signed: true
        into: [isize]
        try_into: [u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128]
        try_from: [u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize]
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::convert::TryFrom;

    use crate::{RangedU32, TryFromIntError};

    #[test]
    fn test_try_from_primitive_to_deranged() {
        assert_eq!(RangedU32::<100, 200>::try_from(50), Err(TryFromIntError));
        assert_eq!(
            RangedU32::<100, 200>::try_from(100),
            Ok(RangedU32::new(100).unwrap())
        );
        assert_eq!(
            RangedU32::<100, 200>::try_from(150),
            Ok(RangedU32::new(150).unwrap())
        );
        assert_eq!(
            RangedU32::<100, 200>::try_from(200),
            Ok(RangedU32::new(200).unwrap())
        );
        assert_eq!(RangedU32::<100, 200>::try_from(250), Err(TryFromIntError));
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_try_from_deranged_to_primitive() {
        assert_eq!(
            u32::try_from(RangedU32::<1000, 2000>::new(1500).unwrap()),
            Ok(1500)
        );
        assert_eq!(
            u8::try_from(RangedU32::<1000, 2000>::new(1500).unwrap()),
            Err(TryFromIntError)
        );
    }

    #[test]
    #[allow(clippy::enum_glob_use)]
    fn test_from_str() {
        type Target = RangedU32<100, 200>;
        use std::num::IntErrorKind as Kind;
        assert!(matches!("50".parse::<Target>(), Err(e) if e.kind() == &Kind::NegOverflow));
        assert!(matches!("100".parse::<Target>(), Ok(val) if val.get() == 100));
        assert!(matches!("150".parse::<Target>(), Ok(val) if val.get() == 150));
        assert!(matches!("200".parse::<Target>(), Ok(val) if val.get() == 200));
        assert!(matches!("250".parse::<Target>(), Err(e) if e.kind() == &Kind::PosOverflow));
        assert!(matches!("abc".parse::<Target>(), Err(e) if e.kind() == &Kind::InvalidDigit));
    }
}
