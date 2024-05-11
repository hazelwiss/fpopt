#![doc = include_str!("../README")]
#![no_std]
#![deny(missing_docs)]

// enforce linking with libc.
extern crate libc;

#[path = "../binding/binding.rs"]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
#[allow(non_camel_case_types)]
#[allow(dead_code)]
#[allow(clippy::useless_transmute)]
#[allow(missing_docs)]
mod binding;

use binding::_bindgen_ty_1 as cfexcpt;
use binding::_bindgen_ty_2 as cfround;

use core::ffi::c_int;
use core::num::NonZeroI32;

#[allow(missing_docs)]
pub mod raw {
    pub use crate::binding::*;
}

// NOTE: uses NoneZeroI32 for optimization reasons.
type Result<T> = core::result::Result<T, NonZeroI32>;

#[inline(always)]
fn result(result: core::ffi::c_int) -> Result<()> {
    NonZeroI32::new(result).map(Err).unwrap_or(Ok(()))
}

macro_rules! flag_ty {
    (
        $(#[$($attr:meta)*])*
        struct $ident:ident ($ty:ty);
        $(
            const $c_ident:ident = $c_expr:expr;
        )*
    ) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        $(#[$($attr)*])*
        pub struct $ident($ty);

        impl $ident {
            $(
                #[allow(missing_docs)]
                pub const $c_ident: Self = Self($c_expr);
            )*

            #[allow(missing_docs)]
            #[inline(always)]
            pub fn none() -> Self {
                Self(0)
            }

            #[allow(missing_docs)]
            #[inline(always)]
            pub fn is_empty(self) -> bool {
                self.as_raw() == 0
            }

            /// See if the given flags exist.
            #[inline(always)]
            pub fn has(self, other: Self) -> bool {
                self.0 & other.0 != 0
            }

            /// Return a new set of flags which enbable all mutual flags between `self` and `other`.
            #[inline(always)]
            pub fn or(self, other: Self) -> Self {
                Self(self.0 | other.0)
            }

            /// Return a new set of flags which remove all mutual flags between `self` and `other`.
            #[inline(always)]
            pub fn not(self, other: Self) -> Self {
                Self(self.0 & !other.0)
            }

            /// Retrieve the raw representation of the flags.
            #[inline(always)]
            pub fn as_raw(self) -> $ty {
                self.0
            }
        }

        impl core::ops::BitOr for $ident {
            type Output = Self;

            #[inline(always)]
            fn bitor(self, rhs: Self) -> Self::Output {
                self.or(rhs)
            }
        }

        impl core::ops::BitOrAssign for $ident {
            #[inline(always)]
            fn bitor_assign(&mut self, rhs: Self) {
                *self = self.or(rhs);
            }
        }
    };
}

flag_ty! (
    /// A wrapper over floating point exception flags.
    ///
    /// This type encapsulate the behaviour of manipulating
    /// the configuration of floating point exceptions without
    /// needing to deal with the raw API.
    struct FExcept(cfexcpt::Type);
    const INVALID = cfexcpt::FE_INVALID;
    const DENORM = cfexcpt::__FE_DENORM;
    const DIV_BY_ZERO = cfexcpt::FE_DIVBYZERO;
    const OVERFLOW = cfexcpt::FE_OVERFLOW;
    const UNDERFLOW = cfexcpt::FE_UNDERFLOW;
    const INEXACT = cfexcpt::FE_INEXACT;
    const FE_ALL = binding::FE_ALL_EXCEPT;
);

flag_ty!(
    /// A wrapper over floating point rounding flags.
    ///
    /// This type encapsulates the behaviour of manipulating
    /// the configuration of floating point rounding without
    /// needing to deal with the raw API.
    struct FRound(cfround::Type);
    const NEAREST = cfround::FE_TONEAREST;
    const DOWNWARD = cfround::FE_DOWNWARD;
    const UPWARD = cfround::FE_UPWARD;
    const TOWARD_ZERO = cfround::FE_TOWARDZERO;
);

impl FExcept {
    /// visit: https://en.cppreference.com/w/c/numeric/fenv/feexceptflag
    pub fn from_env() -> Result<Self> {
        let mut excepts = binding::fexcept_t::default();
        result(unsafe {
            binding::fegetexceptflag(&mut excepts as *mut _, FExcept::FE_ALL.as_raw() as c_int)
        })
        .map(|_| Self(excepts as cfexcpt::Type))
    }

    /// visit: https://en.cppreference.com/w/c/numeric/fenv/feexceptflag
    #[inline(always)]
    pub fn set(self) -> Result<()> {
        let excepts = self.as_raw() as binding::fexcept_t;
        result(unsafe { binding::fesetexceptflag(&excepts as *const _, self.as_raw() as c_int) })
    }
    /// visit: https://en.cppreference.com/w/cpp/numeric/fenv/feclearexcept
    #[inline(always)]
    pub fn clear(self) -> Result<()> {
        result(unsafe { binding::feclearexcept(self.as_raw() as c_int) })
    }

    /// visit: https://en.cppreference.com/w/c/numeric/fenv/feraiseexcept
    #[inline(always)]
    pub fn raise(self) -> Result<()> {
        result(unsafe { binding::feraiseexcept(self.as_raw() as c_int) })
    }

    /// visit: https://en.cppreference.com/w/c/numeric/fenv/fetestexcept
    #[inline(always)]
    pub fn test(self) -> Self {
        Self(unsafe { binding::fetestexcept(self.as_raw() as c_int) } as cfexcpt::Type)
    }
}

impl FRound {
    /// visit: https://en.cppreference.com/w/c/numeric/fenv/feround
    #[inline(always)]
    pub fn from_env() -> Self {
        Self(unsafe { binding::fegetround() as cfround::Type })
    }

    /// visit: https://en.cppreference.com/w/c/numeric/fenv/feround
    #[inline(always)]
    pub fn set(self) -> Result<()> {
        result(unsafe { binding::fesetround(self.as_raw() as c_int) })
    }
}

/// visit: https://en.cppreference.com/w/c/numeric/fenv/feround
#[inline(always)]
pub fn set_rounding_mode(flags: FRound) -> Result<()> {
    flags.set()
}

/// visit: https://en.cppreference.com/w/c/numeric/fenv/feround
#[inline(always)]
pub fn get_rounding_mode() -> FRound {
    FRound::from_env()
}

/// A wrapper around the floating point environment.
///
/// visit: https://cplusplus.com/reference/cfenv/fenv_t/
/// for more information about the underlying `fenv_t` type.
#[derive(Clone, Debug)]
pub struct FEnv(binding::fenv_t);

impl FEnv {
    #[allow(missing_docs)]
    pub fn from_env() -> Result<Self> {
        let mut this: Self = unsafe { core::mem::zeroed() };
        result(unsafe { binding::fegetenv(&mut this.0 as *mut _) }).map(|_| this)
    }

    /// visit: https://en.cppreference.com/w/c/numeric/fenv/feenv
    #[inline(always)]
    pub fn set(&self) -> Result<()> {
        result(unsafe { binding::fesetenv(&self.0 as *const _) })
    }

    /// visit: https://en.cppreference.com/w/c/numeric/fenv/feholdexcept
    #[inline(always)]
    pub fn hold(&mut self) -> Result<()> {
        result(unsafe { binding::feholdexcept(&mut self.0 as *mut _) })
    }

    /// visit:
    #[inline(always)]
    pub fn update(&self) -> Result<()> {
        result(unsafe { binding::feupdateenv(&self.0 as *const _) })
    }

    #[allow(missing_docs)]
    #[inline(always)]
    pub fn inner(&self) -> &binding::fenv_t {
        &self.0
    }

    #[allow(missing_docs)]
    #[inline(always)]
    pub fn inner_mut(&mut self) -> &mut binding::fenv_t {
        &mut self.0
    }
}

impl core::ops::Deref for FEnv {
    type Target = binding::fenv_t;

    fn deref(&self) -> &Self::Target {
        self.inner()
    }
}

impl core::ops::DerefMut for FEnv {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner_mut()
    }
}
