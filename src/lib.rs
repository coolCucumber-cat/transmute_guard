// #![feature(ascii_char)]

/// # Safety
/// Only implement this trait if transmuting from `T` to `Self` and vice versa is safe
pub unsafe trait TransmuteGuard<T>
where
    T: ?Sized,
{
}
unsafe impl<T> TransmuteGuard<T> for T where T: ?Sized {}

/// # Safety
/// Only implement this trait if transmuting from `T` to `Self` and vice versa is safe
pub unsafe trait SafeTransmuteFrom<T>: TransmuteGuard<T> + Sized {
    fn safe_transmute_from(value: T) -> Self;
}
unsafe impl<T> SafeTransmuteFrom<T> for T {
    #[inline]
    fn safe_transmute_from(value: T) -> Self {
        value
    }
}

/// # Safety
/// Only implement this trait if transmuting from `T` to `Self` and vice versa is safe
pub unsafe trait SafeTransmuteRefFrom<T>: TransmuteGuard<T>
where
    T: ?Sized,
{
    fn safe_transmute_ref_from(value: &T) -> &Self;
}
unsafe impl<T, U> SafeTransmuteRefFrom<U> for T
where
    T: TransmuteGuard<U>,
{
    #[inline]
    fn safe_transmute_ref_from(value: &U) -> &Self {
        safe_transmute_ref_const(value)
    }
}

/// # Safety
/// Only implement this trait if transmuting from `T` to `Self` and vice versa is safe
pub unsafe trait SafeTransmuteMutFrom<T>: TransmuteGuard<T>
where
    T: ?Sized,
{
    fn safe_transmute_mut_from(value: &mut T) -> &mut Self;
}
unsafe impl<T, U> SafeTransmuteMutFrom<U> for T
where
    T: TransmuteGuard<U>,
{
    #[inline]
    fn safe_transmute_mut_from(value: &mut U) -> &mut Self {
        safe_transmute_mut_const(value)
    }
}

#[cfg(feature = "ascii_char")]
unsafe impl<U> TransmuteGuard<[U]> for str where core::ascii::Char: SafeTransmuteFrom<U> {}
#[cfg(feature = "ascii_char")]
unsafe impl<T> SafeTransmuteRefFrom<[T]> for str
where
    core::ascii::Char: SafeTransmuteFrom<T>,
{
    #[inline]
    fn safe_transmute_ref_from(value: &[T]) -> &Self {
        let u_ptr = core::ptr::from_ref(value);
        let s_ptr = u_ptr as *const [u8];
        let s: &[u8] = unsafe { &*(s_ptr) };
        unsafe { core::str::from_utf8_unchecked(s) }
    }
}
#[cfg(feature = "ascii_char")]
unsafe impl<U> SafeTransmuteMutFrom<[U]> for str
where
    core::ascii::Char: SafeTransmuteFrom<U>,
{
    #[inline]
    fn safe_transmute_mut_from(value: &mut [U]) -> &mut Self {
        let u_ptr = core::ptr::from_mut(value);
        let s_ptr = u_ptr as *mut [u8];
        let s: &mut [u8] = unsafe { &mut *(s_ptr) };
        unsafe { core::str::from_utf8_unchecked_mut(s) }
    }
}

#[cfg(feature = "ascii_char")]
unsafe impl TransmuteGuard<core::ascii::Char> for u8 {}
#[cfg(feature = "ascii_char")]
unsafe impl SafeTransmuteFrom<core::ascii::Char> for u8 {
    #[inline]
    fn safe_transmute_from(value: core::ascii::Char) -> Self {
        value.to_u8()
    }
}

#[inline]
pub fn safe_transmute<Src, Dst>(src: Src) -> Dst
where
    Dst: SafeTransmuteFrom<Src>,
{
    Dst::safe_transmute_from(src)
}

#[inline]
pub fn safe_transmute_ref<Src, Dst>(src: &Src) -> &Dst
where
    Dst: SafeTransmuteRefFrom<Src> + ?Sized,
    Src: ?Sized,
{
    Dst::safe_transmute_ref_from(src)
}

#[inline]
pub fn safe_transmute_mut<Src, Dst>(src: &mut Src) -> &mut Dst
where
    Dst: SafeTransmuteMutFrom<Src> + ?Sized,
    Src: ?Sized,
{
    Dst::safe_transmute_mut_from(src)
}

#[inline]
pub const fn safe_transmute_ref_const<Src, Dst>(src: &Src) -> &Dst
where
    Dst: SafeTransmuteRefFrom<Src>,
{
    let src_ptr = core::ptr::from_ref(src);
    let dst_ptr: *const Dst = src_ptr.cast();
    unsafe { &*dst_ptr }
}

#[inline]
pub const fn safe_transmute_mut_const<Src, Dst>(src: &mut Src) -> &mut Dst
where
    Dst: SafeTransmuteMutFrom<Src>,
{
    let src_ptr = core::ptr::from_mut(src);
    let dst_ptr: *mut Dst = src_ptr.cast();
    unsafe { &mut *dst_ptr }
}

#[inline]
pub const fn safe_transmute_slice<Src, Dst>(src: &[Src]) -> &[Dst]
where
    Dst: TransmuteGuard<Src>,
{
    let u_ptr = core::ptr::from_ref(src);
    let t_ptr = u_ptr as *const [Dst];
    unsafe { &*t_ptr }
}

#[inline]
pub const fn safe_transmute_slice_mut<Src, Dst>(src: &mut [Src]) -> &mut [Dst]
where
    Dst: TransmuteGuard<Src>,
{
    let u_ptr = core::ptr::from_mut(src);
    let t_ptr = u_ptr as *mut [Dst];
    unsafe { &mut *t_ptr }
}

#[macro_export]
macro_rules! impl_transmute_guard {
    { unsafe ?Sized $From:ty => $To:ty } => {
        unsafe impl $crate::TransmuteGuard<$From> for $To {}
    };
    { unsafe $From:ty => $To:ty } => {
        unsafe impl $crate::TransmuteGuard<$From> for $To {}
        unsafe impl $crate::SafeTransmuteFrom<$From> for $To {
            #[inline]
            fn safe_transmute_from(value: $From) -> Self {
                unsafe { ::core::mem::transmute::<$From, $To>(value) }
            }
        }
    };
}

#[macro_export]
macro_rules! enum_alias {
    {
        $(#[$meta:meta])*
        $vis:vis enum $name:ident: $ty:ty = {$(
            $variant0:ident $(| $variant:ident)*
        )?};
    } => {
        #[repr(u8)]
        #[derive(Debug, Copy, Clone, PartialEq, Eq)]
        $(#[$meta])*
        enum $name {
            $(
                $variant0 = <$ty>::$variant0 as u8,
                $($variant = <$ty>::$variant as u8, )*
            )?
        }

        impl $name {
            #[inline]
            $vis const fn as_u8(self) -> u8 {
                self as u8
            }

            #[inline]
            $vis const fn as_parent(self) -> $ty {
                unsafe { ::core::mem::transmute(self) }
            }

            #[inline]
            $vis const fn try_from_parent(value: $ty) -> ::core::result::Result<Self, ()> {
                match value {
                    $(
                        <$ty>::$variant0 => ::core::result::Result::Ok(<$name>::$variant0),
                        $(
                            <$ty>::$variant => ::core::result::Result::Ok(<$name>::$variant),
                        )*
                        _ => ::core::result::Result::Err(()),
                    )?
                }
            }

        }

        unsafe impl $crate::utils::TransmuteGuard<$name> for $ty {}
        unsafe impl $crate::utils::SafeTransmuteFrom<$name> for $ty {
            #[inline]
            fn safe_transmute_from(value: $name) -> Self {
                $name::as_parent(value)
            }
        }

        impl ::core::convert::From<$name> for $ty {
            #[inline]
            fn from(value: $name) -> Self {
                #[cfg(debug_assertions)]
                let self_dev: Self = match value {
                    $(
                        <$name>::$variant0 => <$ty>::$variant0,
                        $(
                            <$name>::$variant => <$ty>::$variant,
                        )*
                    )?
                };
                let self_prod: Self = $name::as_parent(value);
                #[cfg(debug_assertions)]
                {
                    ::core::debug_assert_eq!(self_dev, self_prod, ::core::concat!(::core::stringify!(::core::convert::From<$name> for $ty)));
                }
                self_prod
            }
        }

        impl ::core::convert::TryFrom<$ty> for $name {
            type Error = ();
            #[inline]
            fn try_from(value: $ty) -> ::core::result::Result<Self, Self::Error> {
                Self::try_from_parent(value)
            }
        }
    };
}
