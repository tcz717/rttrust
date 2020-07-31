use super::IpcFlag;
use crate::{
    cstr::RtName,
    ffi::{rt_mutex_create, rt_mutex_delete, rt_mutex_release, rt_mutex_t, rt_mutex_take},
    Result, RtError,
};

#[derive(Copy, Clone, Debug)]
pub struct Mutex {
    raw: rt_mutex_t,
}

unsafe impl Send for Mutex {}
unsafe impl Sync for Mutex {}

// TODO: init/detach
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
