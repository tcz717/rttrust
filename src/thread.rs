use crate::ffi::{
    rt_thread_create, rt_thread_delay, rt_thread_delete, rt_thread_find, rt_thread_mdelay,
    rt_thread_resume, rt_thread_self, rt_thread_startup, rt_thread_suspend, rt_thread_t,
    rt_thread_yield, rt_tick_t, RT_NAME_MAX,
};
use crate::{Box, Result, RtError};
use arrayvec::ArrayString;
use core::fmt::Write;
use core::{
    ffi::c_void,
    sync::atomic::{AtomicUsize, Ordering},
};

static SPAWNED_THREAD_COUNT: AtomicUsize = AtomicUsize::new(0);

const DEFAULT_STACK_SIZE: u32 = 256;
const DEFAULT_PRIORITY: u8 = 127;
const DEFAULT_TICK: u32 = 10;

unsafe extern "C" fn entry_wrapper<F>(user_data: *mut c_void)
where
    F: FnOnce(),
    F: Send + 'static,
{
    let closure = Box::from_raw(user_data as *mut F);
    closure();
}

pub struct ThreadParameter(*mut c_void);

impl From<usize> for ThreadParameter {
    fn from(data: usize) -> Self {
        ThreadParameter(data as *mut () as *mut c_void)
    }
}
impl From<*mut c_void> for ThreadParameter {
    fn from(data: *mut c_void) -> Self {
        ThreadParameter(data)
    }
}
impl<T> From<Box<T>> for ThreadParameter {
    fn from(data: Box<T>) -> Self {
        ThreadParameter(Box::into_raw(data) as *mut c_void)
    }
}

#[derive(Copy, Clone)]
pub struct Thread {
    raw: rt_thread_t,
}

// TODO: thread cleanup and unwind
// TODO: static thread init/detach and control
impl Thread {
    pub fn create<P>(
        name: &str,
        entry: unsafe extern "C" fn(P),
        parameter: P,
        stack_size: u32,
        priority: u8,
        tick: u32,
    ) -> Result<Thread>
    where
        P: Into<ThreadParameter>,
    {
        let mut buf = ArrayString::<[_; RT_NAME_MAX as usize]>::new();
        buf.push_str(name.get(..RT_NAME_MAX as usize).unwrap_or(name));
        let result = unsafe {
            rt_thread_create(
                buf.as_ptr().cast(),
                Some(core::mem::transmute(entry)),
                Into::<ThreadParameter>::into(parameter).0,
                stack_size,
                priority,
                tick,
            )
        };
        if result.is_null() {
            Err(RtError::Error)
        } else {
            Ok(Thread { raw: result })
        }
    }

    #[cfg(feature = "alloc")]
    #[inline]
    pub fn create_closure<F>(
        name: &str,
        entry: F,
        stack_size: u32,
        priority: u8,
        tick: u32,
    ) -> Result<Thread>
    where
        F: FnOnce(),
        F: Send + 'static,
    {
        let closure = Box::new(entry);
        Self::create(
            name,
            entry_wrapper::<Box<F>>,
            Into::<ThreadParameter>::into(closure).0,
            stack_size,
            priority,
            tick,
        )
    }

    // rt_thread_t rt_thread_self(void);
    // rt_thread_t rt_thread_find(char *name);
    #[inline]
    pub fn current() -> Result<Thread> {
        let result = unsafe { rt_thread_self() };
        if result.is_null() {
            Err(RtError::Error)
        } else {
            Ok(Thread { raw: result })
        }
    }

    pub fn find(name: &str) -> Result<Thread> {
        let mut buf = ArrayString::<[_; RT_NAME_MAX as usize]>::new();
        buf.push_str(name.get(..RT_NAME_MAX as usize).unwrap_or(name));
        let result = unsafe { rt_thread_find(buf.as_mut_ptr().cast()) };
        if result.is_null() {
            Err(RtError::Error)
        } else {
            Ok(Thread { raw: result })
        }
    }

    #[inline]
    pub fn startup(&mut self) -> Result<()> {
        let err = unsafe { rt_thread_startup(self.raw) };
        RtError::from_code_none(err, ())
    }

    #[inline]
    pub fn delete(self) -> Result<()> {
        let err = unsafe { rt_thread_delete(self.raw) };
        RtError::from_code_none(err, ())
    }

    #[inline]
    pub fn yield0() -> Result<()> {
        let err = unsafe { rt_thread_yield() };
        RtError::from_code_none(err, ())
    }

    #[inline]
    pub fn delay(tick: rt_tick_t) -> Result<()> {
        let err = unsafe { rt_thread_delay(tick) };
        RtError::from_code_none(err, ())
    }

    #[inline]
    pub fn mdelay(ms: i32) -> Result<()> {
        let err = unsafe { rt_thread_mdelay(ms) };
        RtError::from_code_none(err, ())
    }

    #[inline]
    pub fn suspend(&mut self) -> Result<()> {
        let err = unsafe { rt_thread_suspend(self.raw) };
        RtError::from_code_none(err, ())
    }

    #[inline]
    pub fn resume(&mut self) -> Result<()> {
        let err = unsafe { rt_thread_resume(self.raw) };
        RtError::from_code_none(err, ())
    }
}

#[cfg(feature = "alloc")]
pub fn spawn<F>(f: F) -> Thread
where
    F: FnOnce(),
    F: Send + 'static,
{
    let id = SPAWNED_THREAD_COUNT.fetch_add(1, Ordering::SeqCst);
    let mut name = ArrayString::<[_; RT_NAME_MAX as usize]>::new();
    write!(&mut name, "rust{}", id).ok();
    let mut thread = Thread::create_closure(
        name.as_str(),
        f,
        DEFAULT_STACK_SIZE,
        DEFAULT_PRIORITY,
        DEFAULT_TICK,
    )
    .unwrap();

    thread.startup().unwrap();
    thread
}
