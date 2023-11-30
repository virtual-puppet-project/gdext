/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::builtin::meta::{impl_godot_as_self, FromGodot, GodotConvert, GodotType, ToGodot};
use godot_ffi as sys;

// The following ToGodot/FromGodot/Convert impls are auto-generated for each engine type, co-located with their definitions:
// - enum
// - const/mut pointer to native struct

// ----------------------------------------------------------------------------------------------------------------------------------------------
// Option

impl<T: GodotConvert> GodotConvert for Option<T>
where
    Option<T::Via>: GodotType,
{
    type Via = Option<T::Via>;
}

impl<T: ToGodot> ToGodot for Option<T>
where
    Option<T::Via>: GodotType,
{
    fn to_godot(&self) -> Self::Via {
        self.as_ref().map(ToGodot::to_godot)
    }

    fn into_godot(self) -> Self::Via {
        self.map(ToGodot::into_godot)
    }
}

impl<T: FromGodot> FromGodot for Option<T>
where
    Option<T::Via>: GodotType,
{
    fn try_from_godot(via: Self::Via) -> Option<Self> {
        match via {
            Some(via) => T::try_from_godot(via).map(Some),
            None => Some(None),
        }
    }

    fn from_godot(via: Self::Via) -> Self {
        via.map(T::from_godot)
    }
}

// ----------------------------------------------------------------------------------------------------------------------------------------------
// Builtin Godot types

impl GodotConvert for sys::VariantType {
    type Via = i32;
}

impl ToGodot for sys::VariantType {
    fn to_godot(&self) -> Self::Via {
        *self as i32
    }

    fn into_godot(self) -> Self::Via {
        self as i32
    }
}

impl FromGodot for sys::VariantType {
    fn try_from_godot(via: Self::Via) -> Option<Self> {
        Some(Self::from_sys(via as sys::GDExtensionVariantType))
    }
}

impl GodotConvert for sys::VariantOperator {
    type Via = i32;
}

impl ToGodot for sys::VariantOperator {
    fn to_godot(&self) -> Self::Via {
        *self as i32
    }

    fn into_godot(self) -> Self::Via {
        self as i32
    }
}

impl FromGodot for sys::VariantOperator {
    fn try_from_godot(via: Self::Via) -> Option<Self> {
        Some(Self::from_sys(via as sys::GDExtensionVariantOperator))
    }
}

// ----------------------------------------------------------------------------------------------------------------------------------------------
// Scalars

macro_rules! impl_godot_scalar {
    ($T:ty as $Via:ty $(, $param_metadata:expr)?) => {
        impl GodotType for $T {
            type Ffi = $Via;

            fn to_ffi(&self) -> Self::Ffi {
                (*self).into()
            }

            fn into_ffi(self) -> Self::Ffi {
                self.into()
            }

            fn try_from_ffi(ffi: Self::Ffi) -> Option<Self> {
                Self::try_from(ffi).ok()
            }

            $(
                fn param_metadata() -> sys::GDExtensionClassMethodArgumentMetadata {
                    $param_metadata
                }
            )?
        }

        impl GodotConvert for $T {
            type Via = $T;
        }

        impl ToGodot for $T {
            fn to_godot(&self) -> Self::Via {
                *self
            }
        }

        impl FromGodot for $T {
            fn try_from_godot(via: Self::Via) -> Option<Self> {
                Some(via)
            }
        }
    };

    ($T:ty as $Via:ty $(, $param_metadata:expr)?; lossy) => {
        impl GodotType for $T {
            type Ffi = $Via;

            fn to_ffi(&self) -> Self::Ffi {
                *self as $Via
            }

            fn into_ffi(self) -> Self::Ffi {
                self as $Via
            }

            fn try_from_ffi(ffi: Self::Ffi) -> Option<Self> {
                Some(ffi as $T)
            }

            $(
                fn param_metadata() -> sys::GDExtensionClassMethodArgumentMetadata {
                    $param_metadata
                }
            )?
        }

        impl GodotConvert for $T {
            type Via = $T;
        }

        impl ToGodot for $T {
            fn to_godot(&self) -> Self::Via {
                *self
            }
        }

        impl FromGodot for $T {
            fn try_from_godot(via: Self::Via) -> Option<Self> {
                Some(via)
            }
        }
    };
}

// `GodotType` for these three is implemented in `godot-core/src/builtin/variant/impls.rs`.
impl_godot_as_self!(bool);
impl_godot_as_self!(i64);
impl_godot_as_self!(f64);
impl_godot_as_self!(());

impl_godot_scalar!(
    i32 as i64,
    sys::GDEXTENSION_METHOD_ARGUMENT_METADATA_INT_IS_INT32
);
impl_godot_scalar!(
    i16 as i64,
    sys::GDEXTENSION_METHOD_ARGUMENT_METADATA_INT_IS_INT16
);
impl_godot_scalar!(
    i8 as i64,
    sys::GDEXTENSION_METHOD_ARGUMENT_METADATA_INT_IS_INT8
);
impl_godot_scalar!(
    u32 as i64,
    sys::GDEXTENSION_METHOD_ARGUMENT_METADATA_INT_IS_UINT32
);
impl_godot_scalar!(
    u16 as i64,
    sys::GDEXTENSION_METHOD_ARGUMENT_METADATA_INT_IS_UINT16
);
impl_godot_scalar!(
    u8 as i64,
    sys::GDEXTENSION_METHOD_ARGUMENT_METADATA_INT_IS_UINT8
);
impl_godot_scalar!(
    u64 as i64,
    sys::GDEXTENSION_METHOD_ARGUMENT_METADATA_INT_IS_UINT64;
    lossy
);
impl_godot_scalar!(
    f32 as f64,
    sys::GDEXTENSION_METHOD_ARGUMENT_METADATA_REAL_IS_FLOAT;
    lossy
);

// ----------------------------------------------------------------------------------------------------------------------------------------------
// Raw pointers

// const void* is used in some APIs like OpenXrApiExtension::transform_from_pose().
// Other impls for raw pointers are generated for native structures.

impl GodotConvert for *const std::ffi::c_void {
    type Via = i64;
}

impl ToGodot for *const std::ffi::c_void {
    fn to_godot(&self) -> Self::Via {
        *self as i64
    }
}

impl FromGodot for *const std::ffi::c_void {
    fn try_from_godot(via: Self::Via) -> Option<Self> {
        Some(via as Self)
    }
}
