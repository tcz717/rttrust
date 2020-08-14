use super::IpcFlag;
use crate::{
    cstr::RtName,
    ffi::{
        rt_object, rt_sem_create, rt_sem_delete, rt_sem_detach, rt_sem_init, rt_sem_release,
        rt_sem_t, rt_sem_take, rt_sem_trytake, rt_semaphore,
    },
    object::Object,
    Result, RtError,
};
use core::{cell::UnsafeCell, marker::PhantomPinned, mem::MaybeUninit, ptr::NonNull};

#[derive(Copy, Clone, Debug)]
pub struct Semaphore {
    raw: rt_sem_t,
}

// TODO: init/detach
impl Semaphore {
    pub fn create(name: &str, value: u32, flag: IpcFlag) -> Result<Self> {
        let name: RtName = name.into();
        let result = unsafe { rt_sem_create(name.into(), value, flag.into()) };
        if result.is_null() {
            Err(RtError::Error)
        } else {
            Ok(Semaphore { raw: result })
        }
    }

    #[inline]
    pub fn delete(self) -> Result<()> {
        let err = unsafe { rt_sem_delete(self.raw) };
        RtError::from_code_none(err, ())
    }

    #[inline]
    pub fn take(&mut self, time: i32) -> Result<()> {
        let err = unsafe { rt_sem_take(self.raw, time) };
        RtError::from_code_none(err, ())
    }

    #[inline]
    pub fn try_take(&mut self) -> Result<()> {
        let err = unsafe { rt_sem_trytake(self.raw) };
        RtError::from_code_none(err, ())
    }

    #[inline]
    pub fn release(&mut self) -> Result<()> {
        let err = unsafe { rt_sem_release(self.raw) };
        RtError::from_code_none(err, ())
    }
}

impl Object for Semaphore {
    fn get_ptr(&self) -> NonNull<rt_object> {
        NonNull::new(self.raw.cast()).expect("Unexpected null rt_sem_t")
    }
}

pub struct SemaphoreStatic {
    raw: UnsafeCell<MaybeUninit<rt_semaphore>>,
    _pinned: PhantomPinned,
}

unsafe impl Send for SemaphoreStatic {}
unsafe impl Sync for SemaphoreStatic {}

impl SemaphoreStatic {
    pub const fn new() -> Self {
        SemaphoreStatic {
            raw: UnsafeCell::new(core::mem::MaybeUninit::uninit()),
            _pinned: PhantomPinned {},
        }
    }
    pub fn init(&'static self, name: &str, value: u32, flag: IpcFlag) -> Result<()> {
        let name: RtName = name.into();
        let err = unsafe { rt_sem_init(self.raw.get().cast(), name.into(), value, flag.into()) };
        RtError::from_code_none(err, ())
    }

    #[inline]
    pub fn detach(&'static self) -> Result<()> {
        let err = unsafe { rt_sem_detach(self.raw.get().cast()) };
        RtError::from_code_none(err, ())
    }

    pub fn get(&'static self) -> Semaphore {
        Semaphore {
            raw: self.raw.get().cast(),
        }
    }
}
