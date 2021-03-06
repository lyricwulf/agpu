#[macro_export]
macro_rules! const_flag {
    // The pattern for a single bitflag
    ($t:ty, $e:ident) => {
        <$t>::$e
    };

    // Combine multiple bitflags passing directly to `bitor_variadic!()`
    ($t:ty, $($rest:ident),*) => {
        <$t>::from_bits_truncate(
            $crate::bitor_variadic!($t, $($rest),*)
        )
    };

    // Combine multiple bitflags passing directly to `bitor_variadic!()`
    ($name:ident, $t:ty, $($rest:ident),*) => {
        const $name: $t = <$t>::from_bits_truncate(
            $crate::bitor_variadic!($t, $($rest),*)
        );
    };
}

#[macro_export]
macro_rules! pub_const_flag {
    // The pattern for a single bitflag
    ($name:ident : $t:ty = $($rest:ident)|*) => {
        pub const $name: $t = <$t>::from_bits_truncate(
            $crate::bitor_variadic!($t, $($rest),*)
        );
    };
}

#[macro_export]
macro_rules! bitor {
    ($name:ident : $t:ty = $($rest:ident)|*) => {
        let $name: $t = <$t>::from_bits_truncate(
            $crate::bitor_variadic!($t, $($rest),*)
        );
    };

    ($t:ty : $($rest:ident)|*) => {
        <$t>::from_bits_truncate(
            $crate::bitor_variadic!($t, $($rest),*)
        )
    };
}

#[macro_export]
// Used by `const_flags!` to combine bitflags using | on .bits()
macro_rules! bitor_variadic {
    // Match the trivial case
    ($t:ty, $i:ident) => {
        <$t>::$i.bits()
    };
    // Match case with two identifiers
    // They will need to be :: then call .bits() before | operator
    ($t:ty, $i1:ident, $i2:ident) => {
        <$t>::$i1.bits() | <$t>::$i2.bits()
    };
    // Match case with list of identifiers
    // This will be the topmost macro, with none of the arguments being eval yet.
    ($t:ty, $i1:ident, $i2:ident, $($rest:ident), *) => {
        //               f  expr
        $crate::bitor_variadic!($t, <$t>::$i1.bits() | <$t>::$i2.bits(), $($rest), *)
    };
    // Match case with expr and list of idents
    // When the first argument is an expr, it has been :: and .bits() already
    // so we just append the next argument with :: and | to the expr
    ($t:ty, $e:expr, $i:ident, $($rest:ident), *) => {
        $crate::bitor_variadic!($t, $e | <$t>::$i.bits(), $($rest), *)
    };
    // Match case with expr and ident
    // This will be the final macro call
    ($t:ty, $e:expr, $i:ident) => {
        $e | <$t>::$i.bits()
    };
}

// Internal use macro to (ab)use Deref trait to get the underlying wgpu type
#[macro_export]
macro_rules! wgpu_inner_deref {
    ($name: tt, $wgpu: tt) => {
        impl ::std::ops::Deref for $name {
            type Target = ::wgpu::$wgpu;
            fn deref(&self) -> &Self::Target {
                &self.inner
            }
        }
        impl ::std::ops::DerefMut for $name {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.inner
            }
        }
    };
    ($name: tt<'_>, $wgpu: tt) => {
        impl ::std::ops::Deref for $name<'_> {
            type Target = ::wgpu::$wgpu;
            fn deref(&self) -> &Self::Target {
                &self.inner
            }
        }
        impl ::std::ops::DerefMut for $name<'_> {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.inner
            }
        }
    };
    ($name: tt <$($gen: tt),*>, $wgpu: tt) => {
        impl<$($gen),*> ::std::ops::Deref for $name<$($gen),*> {
            type Target = ::wgpu::$wgpu;
            fn deref(&self) -> &Self::Target {
                &self.inner
            }
        }
        impl<$($gen),*> ::std::ops::DerefMut for $name<$($gen),*> {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.inner
            }
        }
    };
    ($name: tt) => {
        // We shouldn't need the `$crate` but it won't compile without it
        $crate::wgpu_inner_deref!($name, $name);
    };
}
