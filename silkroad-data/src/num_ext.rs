pub(crate) trait NumExt {
    type Output;
    fn to_option(self) -> Option<Self::Output>;
}

macro_rules! impl_opt {
    ($num_type:tt) => {
        impl NumExt for $num_type {
            type Output = $num_type;

            fn to_option(self) -> Option<Self::Output> {
                if self == 0 {
                    None
                } else {
                    Some(self)
                }
            }
        }
    };
}

impl_opt!(u32);
impl_opt!(u16);
impl_opt!(u8);
