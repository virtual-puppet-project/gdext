/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use godot_ffi as sys;

use crate::builtin::{inner, StringName, ToVariant, Variant, VariantArray};
use crate::engine::Object;
use crate::obj::mem::Memory;
use crate::obj::{Gd, GodotClass, InstanceId};
use std::{fmt, ptr};
use sys::{ffi_methods, GodotFfi};

/// A `Callable` represents a function in Godot.
///
/// Usually a callable is a reference to an `Object` and a method name, this is a standard callable. But can
/// also be a custom callable, which is usually created from `bind`, `unbind`, or a GDScript lambda. See
/// [`Callable::is_custom`].
///
/// Currently it is impossible to use `bind` and `unbind` in GDExtension, see [godot-cpp#802].
///
/// [godot-cpp#802]: https://github.com/godotengine/godot-cpp/issues/802
#[repr(C, align(8))]
pub struct Callable {
    opaque: sys::types::OpaqueCallable,
}

impl Callable {
    fn from_opaque(opaque: sys::types::OpaqueCallable) -> Self {
        Self { opaque }
    }

    /// Create a callable for the method `object::method_name`.
    ///
    /// _Godot equivalent: `Callable(Object object, StringName method)`_
    pub fn from_object_method<T, S>(object: Gd<T>, method_name: S) -> Self
    where
        T: GodotClass, // + Inherits<Object>,
        S: Into<StringName>,
    {
        // upcast not needed
        let method = method_name.into();
        unsafe {
            sys::from_sys_init_or_init_default::<Self>(|self_ptr| {
                let ctor = sys::builtin_fn!(callable_from_object_method);
                let args = [object.as_arg_ptr(), method.sys_const()];
                ctor(self_ptr, args.as_ptr());
            })
        }
    }

    /// Create a callable representing a Rust function.
    // #[cfg(since_api = "4.2")]
    pub fn from_custom<C: RustCallable>(callable: C) -> Self {
        // Double box could be avoided if we propagate C through all functions.
        let wrapper = CallableUserdata {
            callable: Box::new(callable),
        };
        Self::from_custom_inner(Box::new(wrapper))
    }

    // #[cfg(since_api = "4.2")]
    fn from_custom_inner(callable: Box<CallableUserdata>) -> Self {
        let mut info = sys::GDExtensionCallableCustomInfo {
            callable_userdata: Box::into_raw(callable) as *mut std::ffi::c_void,
            token: ptr::null_mut(),
            object: ptr::null_mut(),
            call_func: Some(rust_callable_call),
            is_valid_func: None,
            free_func: Some(rust_callable_destroy),
            hash_func: Some(rust_callable_hash),
            equal_func: Some(rust_callable_equal),
            // < is only used in niche scenarios and default is usually good enough, see https://github.com/godotengine/godot/issues/81901.
            less_than_func: None,
            to_string_func: Some(rust_callable_to_string),
        };

        unsafe {
            Callable::from_sys_init(|type_ptr| {
                sys::interface_fn!(callable_custom_create)(type_ptr, ptr::addr_of_mut!(info))
            })
        }
    }

    /// Creates an invalid/empty object that is not able to be called.
    ///
    /// _Godot equivalent: `Callable()`_
    pub fn invalid() -> Self {
        unsafe {
            Self::from_sys_init(|self_ptr| {
                let ctor = sys::builtin_fn!(callable_construct_default);
                ctor(self_ptr, ptr::null_mut())
            })
        }
    }

    /// Calls the method represented by this callable.
    ///
    /// Arguments passed should match the method's signature.
    ///
    /// - If called with more arguments than expected by the method, the extra arguments will be ignored and
    ///   the call continues as normal.
    /// - If called with fewer arguments than expected it will crash Godot, without triggering UB.
    /// - If called with arguments of the wrong type then an error will be printed and the call will return
    ///   `NIL`.
    /// - If called on an invalid Callable then no error is printed, and `NIL` is returned.
    ///
    /// _Godot equivalent: `callv`_
    pub fn callv(&self, arguments: VariantArray) -> Variant {
        self.as_inner().callv(arguments)
    }

    /// Returns the name of the method represented by this callable. If the callable is a lambda function,
    /// returns the function's name.
    ///
    /// ## Known Bugs
    ///
    /// Getting the name of a lambda errors instead of returning its name, see [godot#73052].
    ///
    /// _Godot equivalent: `get_method`_
    ///
    /// [godot#73052]: https://github.com/godotengine/godot/issues/73052
    pub fn method_name(&self) -> Option<StringName> {
        let method_name = self.as_inner().get_method();
        if method_name.is_empty() {
            None
        } else {
            Some(method_name)
        }
    }

    /// Returns the object on which this callable is called.
    ///
    /// Returns `None` when this callable doesn't have any target object to call a method on, regardless of
    /// if the method exists for that target or not.
    ///
    /// _Godot equivalent: `get_object`_
    pub fn object(&self) -> Option<Gd<Object>> {
        // Increment refcount because we're getting a reference, and `InnerCallable::get_object` doesn't
        // increment the refcount.
        self.as_inner().get_object().map(|object| {
            <Object as GodotClass>::Mem::maybe_inc_ref(&object);
            object
        })
    }

    /// Returns the ID of this callable's object, see also [`Gd::instance_id`].
    ///
    /// Returns `None` when this callable doesn't have any target to call a method on.
    ///
    /// _Godot equivalent: `get_object_id`_
    pub fn object_id(&self) -> Option<InstanceId> {
        let id = self.as_inner().get_object_id();
        InstanceId::try_from_i64(id)
    }

    /// Returns the 32-bit hash value of this callable's object.
    ///
    /// _Godot equivalent: `hash`_
    pub fn hash(&self) -> u32 {
        self.as_inner().hash().try_into().unwrap()
    }

    /// Returns true if this callable is a custom callable.
    ///
    /// Custom callables are mainly created from bind or unbind. In GDScript, lambda functions are also
    /// custom callables.
    ///
    /// If a callable is not a custom callable, then it is considered a standard callable, this function is
    /// the opposite of [`Callable.is_standard`].
    ///
    /// _Godot equivalent: `is_custom`_
    ///
    /// [`Callable.is_standard`]: https://docs.godotengine.org/en/stable/classes/class_callable.html#class-callable-method-is-standard
    #[doc(alias = "is_standard")]
    pub fn is_custom(&self) -> bool {
        self.as_inner().is_custom()
    }

    /// Returns true if this callable has no target to call the method on.
    ///
    /// This is not the negated form of [`is_valid`][Self::is_valid], as `is_valid` will return `false` if the callable has a
    /// target but the method does not exist.
    ///
    /// _Godot equivalent: `is_null`_
    pub fn is_null(&self) -> bool {
        self.as_inner().is_null()
    }

    /// Returns true if the callable's object exists and has a valid method name assigned, or is a custom
    /// callable.
    ///
    /// _Godot equivalent: `is_valid`_
    pub fn is_valid(&self) -> bool {
        self.as_inner().is_valid()
    }

    #[doc(hidden)]
    pub fn as_inner(&self) -> inner::InnerCallable {
        inner::InnerCallable::from_outer(self)
    }
}

impl_builtin_traits! {
    for Callable {
        // Currently no Default::default() to encourage explicit valid initialization.
        //Default => callable_construct_default;

        // Equality for custom callables depend on the equality implementation of that custom callable.
        // So we cannot implement `Eq` here and be confident equality will be total for all future custom callables.
        PartialEq => callable_operator_equal;
        // PartialOrd => callable_operator_less;
        Clone => callable_construct_copy;
        Drop => callable_destroy;
    }
}

// SAFETY:
// The `opaque` in `Callable` is just a pair of pointers, and requires no special initialization or cleanup
// beyond what is done in `from_opaque` and `drop`. So using `*mut Opaque` is safe.
unsafe impl GodotFfi for Callable {
    ffi_methods! { type sys::GDExtensionTypePtr = *mut Opaque; .. }

    unsafe fn from_sys_init_default(init_fn: impl FnOnce(sys::GDExtensionTypePtr)) -> Self {
        let mut result = Self::invalid();
        init_fn(result.sys_mut());
        result
    }
}

impl fmt::Debug for Callable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let method = self.method_name();
        let object = self.object();

        f.debug_struct("Callable")
            .field("method", &method)
            .field("object", &object)
            .finish()
    }
}

impl fmt::Display for Callable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_variant())
    }
}

// ----------------------------------------------------------------------------------------------------------------------------------------------
// Callbacks for custom implementations

// #[cfg(since_api = "4.2")]
use custom_callable::*;

// #[cfg(since_api = "4.2")]
pub use custom_callable::RustCallable;

// #[cfg(since_api = "4.2")]
mod custom_callable {
    use super::*;
    use crate::builtin::GodotString;
    use std::hash::Hash;

    pub struct CallableUserdata {
        pub callable: Box<dyn RustCallable>,
    }

    impl CallableUserdata {
        unsafe fn inner_from_raw<'a>(void_ptr: *mut std::ffi::c_void) -> &'a dyn RustCallable {
            let ptr = void_ptr as *mut CallableUserdata;
            &(*ptr).callable
        }
    }

    pub trait RustCallable:
        'static + Sized + PartialEq + Hash + fmt::Display + Send + Sync
    where
        Self: Sized,
    {
        fn invoke(&mut self, args: &[&Variant]) -> Result<Variant, ()>;
    }

    pub unsafe extern "C" fn rust_callable_call(
        callable_userdata: *mut std::ffi::c_void,
        p_args: *const sys::GDExtensionConstVariantPtr,
        p_argument_count: sys::GDExtensionInt,
        r_return: sys::GDExtensionVariantPtr,
        r_error: *mut sys::GDExtensionCallError,
    ) {
        let arg_refs: &[&Variant] =
            Variant::unbounded_refs_from_sys(p_args, p_argument_count as usize);

        let mut c = CallableUserdata::inner_from_raw(callable_userdata);

        let result = c.invoke(arg_refs);
        crate::builtin::meta::varcall_return_checked(result, r_return, r_error);
    }

    pub unsafe extern "C" fn rust_callable_destroy(callable_userdata: *mut std::ffi::c_void) {
        let rust_ptr = callable_userdata as *mut CallableUserdata;
        let _drop = Box::from_raw(rust_ptr);
    }

    pub unsafe extern "C" fn rust_callable_hash(callable_userdata: *mut std::ffi::c_void) -> u32 {
        let c = CallableUserdata::inner_from_raw(callable_userdata);

        // Just cut off top bits, not best-possible hash.
        sys::hash_value(c) as u32
    }

    pub unsafe extern "C" fn rust_callable_equal(
        callable_userdata_a: *mut std::ffi::c_void,
        callable_userdata_b: *mut std::ffi::c_void,
    ) -> sys::GDExtensionBool {
        let a = CallableUserdata::inner_from_raw(callable_userdata_a);
        let b = CallableUserdata::inner_from_raw(callable_userdata_b);

        (a == b) as sys::GDExtensionBool
    }

    pub unsafe extern "C" fn rust_callable_less(
        callable_userdata_a: *mut std::ffi::c_void,
        callable_userdata_b: *mut std::ffi::c_void,
    ) -> sys::GDExtensionBool {
        let a = CallableUserdata::inner_from_raw(callable_userdata_a);
        let b = CallableUserdata::inner_from_raw(callable_userdata_b);

        (a < b) as sys::GDExtensionBool
    }

    pub unsafe extern "C" fn rust_callable_to_string(
        callable_userdata: *mut std::ffi::c_void,
        r_is_valid: *mut sys::GDExtensionBool,
        r_out: sys::GDExtensionStringPtr,
    ) {
        let c = CallableUserdata::inner_from_raw(callable_userdata);
        let s = StringName::from(c.to_string());

        s.move_string_ptr(r_out);

        *r_is_valid = true as sys::GDExtensionBool;
    }
}
