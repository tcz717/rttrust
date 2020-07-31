use crate::{
    cstr::RtName,
    ffi::{rt_sem_create, rt_sem_delete, rt_sem_release, rt_sem_t, rt_sem_take, rt_sem_trytake},
    Result, RtError,
};
use super::IpcFlag;

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
