use super::IpcFlag;
use crate::{
    cstr::RtName,
    ffi::{
        rt_mutex, rt_mutex_create, rt_mutex_delete, rt_mutex_detach, rt_mutex_init,
        rt_mutex_release, rt_mutex_t, rt_mutex_take, rt_object,
    },
    object::Object,
    Result, RtError,
};
use core::{cell::UnsafeCell, marker::PhantomPinned, mem::MaybeUninit, ptr::NonNull};

#[derive(Copy, Clone, Debug)]
pub struct Mutex {
    raw: rt_mutex_t,
}

unsafe impl Send for Mutex {}
unsafe impl Sync for Mutex {}

impl Mutex {
    pub fn create(name: &str, flag: IpcFlag) -> Result<Self> {
        let name: RtName = name.into();
        let result = unsafe { rt_mutex_create(name.into(), flag.into()) };
        if result.is_null() {
            Err(RtError::Error)
        } else {
            Ok(Mutex { raw: result })
        }
    }

    #[inline]
    pub fn delete(self) -> Result<()> {
        let err = unsafe { rt_mutex_delete(self.raw) };
        RtError::from_code_none(err, ())
    }

    #[inline]
    pub fn take(&self, time: i32) -> Result<()> {
        let err = unsafe { rt_mutex_take(self.raw, time) };
        RtError::from_code_none(err, ())
    }

    #[inline]
    pub fn release(&self) -> Result<()> {
        let err = unsafe { rt_mutex_release(self.raw) };
        RtError::from_code_none(err, ())
    }
}

impl Object for Mutex {
    fn get_ptr(&self) -> NonNull<rt_object> {
        NonNull::new(self.raw.cast()).expect("Unexpected null rt_mutex_t")
    }
}

pub struct MutexStatic {
    raw: UnsafeCell<MaybeUninit<rt_mutex>>,
    _pinned: PhantomPinned,
}

unsafe impl Send for MutexStatic {}
unsafe impl Sync for MutexStatic {}

impl MutexStatic {
    pub const fn new() -> Self {
        MutexStatic {
            raw: UnsafeCell::new(core::mem::MaybeUninit::uninit()),
            _pinned: PhantomPinned {},
        }
    }
    pub fn init(&'static self, name: &str, flag: IpcFlag) -> Result<()> {
        let name: RtName = name.into();
        let err = unsafe { rt_mutex_init(self.raw.get().cast(), name.into(), flag.into()) };
        RtError::from_code_none(err, ())
    }

    #[inline]
    pub fn detach(&'static self) -> Result<()> {
        let err = unsafe { rt_mutex_detach(self.raw.get().cast()) };
        RtError::from_code_none(err, ())
    }

    pub fn get(&'static self) -> Mutex {
        Mutex {
            raw: self.raw.get().cast(),
        }
    }
}
