use crate::{
    cstr::RtNameRef,
    ffi::{rt_object, rt_object_get_type, rt_object_is_systemobject, RT_TRUE},
};
use core::ptr::NonNull;

pub trait Object {
    fn get_ptr(&self) -> NonNull<rt_object>;

    #[inline]
    fn is_systemobject(&self) -> bool {
        unsafe { rt_object_is_systemobject(self.get_ptr().as_ptr()) == RT_TRUE as i32 }
    }

    #[inline]
    fn get_type(&self) -> u8 {
        unsafe { rt_object_get_type(self.get_ptr().as_ptr()) }
    }

    #[inline]
    fn get_name<'a>(&'a self) -> RtNameRef<'a> {
        unsafe { RtNameRef::from(&self.get_ptr().as_ptr().as_ref().unwrap().name) }
    }
}
