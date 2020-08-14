//! Thread is the smallest scheduling unit in the RT-Thread operating system. Thread scheduling algorithm is a priority-based full preemptive multi-thread scheduling algorithm, that is, except the interrupt handler, the code of the scheduler's locked part, and the code that prohibits the interrupt, other parts of the system can be preempted, including the thread scheduler itself. The system supports 256 thread priorities (can also be changed to a maximum of 32 or 8 thread priority via configuration file; for STM32 default configuration, it is set as 32 thread priorities), 0 priority represents highest priority, and lowest priority is reserved for idle threads; at the same time, it also supports creating multiple threads with the same priority. The same priority threads are scheduled with a time slice rotation scheduling algorithm, so that each thread runs for the same time; in addition, when the scheduler is looking for threads that are at the highest priority thread and ready, the elapsed time is constant. The system does not limit the number of threads, the number of threads is only related to the specific memory of the hardware platform.

//! ### TODO
//! 1. Thread control 
//! 1. Extract ThreadParameter to reuse

use crate::ffi::{
    rt_object, rt_thread, rt_thread_create, rt_thread_delay, rt_thread_delete, rt_thread_detach,
    rt_thread_find, rt_thread_init, rt_thread_mdelay, rt_thread_resume, rt_thread_self,
    rt_thread_startup, rt_thread_suspend, rt_thread_t, rt_thread_yield, rt_tick_t,
};
use crate::{cstr::RtName, Result, RtError,object::Object};

use core::{cell::UnsafeCell, ffi::c_void, marker::PhantomPinned, mem::MaybeUninit, ptr::NonNull};

#[cfg(feature = "alloc")]
use crate::Box;

#[derive(Debug)]
pub struct ThreadParameter(*mut c_void);

pub type ThreadEntry<P> = unsafe extern "C" fn(P);

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
impl From<Box<Box<dyn FnOnce() + 'static>>> for ThreadParameter {
    fn from(data: Box<Box<dyn FnOnce() + 'static>>) -> Self {
        ThreadParameter(Box::into_raw(data).cast())
    }
}

pub struct ThreadStatic {
    raw: UnsafeCell<MaybeUninit<rt_thread>>,
    _pinned: PhantomPinned,
}

unsafe impl Send for ThreadStatic {}
/// Will be removed once find a way to static init
unsafe impl Sync for ThreadStatic {}

impl ThreadStatic {
    pub const fn new() -> Self {
        ThreadStatic {
            raw: UnsafeCell::new(core::mem::MaybeUninit::uninit()),
            _pinned: PhantomPinned {},
        }
    }

    pub fn init<P>(
        &'static self,
        name: &str,
        entry: ThreadEntry<P>,
        parameter: P,
        stack: &'static mut [c_void],
        priority: u8,
        tick: u32,
    ) -> Result<()>
    where
        P: Into<ThreadParameter>,
    {
        let name: RtName = name.into();
        let err = unsafe {
            rt_thread_init(
                self.raw.get().cast(),
                name.into(),
                Some(core::mem::transmute(entry)),
                Into::<ThreadParameter>::into(parameter).0,
                stack.as_mut_ptr(),
                stack.len() as u32,
                priority,
                tick,
            )
        };
        RtError::from_code_none(err, ())
    }

    /// ### Wanring:
    /// Rust thread has no cleanup function and is not able to unwind.
    /// Directly stopping a Rust thread may lead to resoucre leak.
    #[inline]
    pub fn detach(&'static self) -> Result<()> {
        let err = unsafe { rt_thread_detach(self.raw.get().cast()) };
        RtError::from_code_none(err, ())
    }

    pub fn get(&'static self) -> Thread {
        Thread {
            raw: self.raw.get().cast(),
        }
    }
}

/// All methods use immutable self since the safety is guaranteed by rt-thread internal
///
/// TODO: thread cleanup and unwind
///
/// https://blog.rust-lang.org/inside-rust/2020/02/27/ffi-unwind-design-meeting.html
///
/// TODO: thread control
#[derive(Copy, Clone)]
pub struct Thread {
    raw: rt_thread_t,
}

unsafe impl Send for Thread {}

impl Thread {
    pub fn create<EP, P>(
        name: &str,
        entry: ThreadEntry<EP>,
        parameter: P,
        stack_size: u32,
        priority: u8,
        tick: u32,
    ) -> Result<Thread>
    where
        EP: Into<ThreadParameter>,
        P: Into<ThreadParameter>,
    {
        let name: RtName = name.into();
        let result = unsafe {
            rt_thread_create(
                name.into(),
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
        unsafe extern "C" fn entry_wrapper(user_data: *mut c_void) {
            let closure: Box<Box<dyn FnOnce() + 'static>> = Box::from_raw(user_data.cast());
            closure();
        }

        let closure: Box<dyn FnOnce() + 'static> = Box::new(entry);
        Self::create(
            name,
            entry_wrapper,
            // Trait object is a fat pointer and has to be put in another Box
            Box::new(closure),
            stack_size,
            priority,
            tick,
        )
    }

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
        let name: RtName = name.into();
        let result = unsafe { rt_thread_find(name.into()) };
        if result.is_null() {
            Err(RtError::Error)
        } else {
            Ok(Thread { raw: result })
        }
    }

    #[inline]
    pub fn startup(&self) -> Result<()> {
        let err = unsafe { rt_thread_startup(self.raw) };
        RtError::from_code_none(err, ())
    }

    /// ### Wanring:
    /// Rust thread has no cleanup function and is not able to unwind.
    /// Directly stopping a Rust thread may lead to resoucre leak.
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
    pub fn suspend(&self) -> Result<()> {
        let err = unsafe { rt_thread_suspend(self.raw) };
        RtError::from_code_none(err, ())
    }

    #[inline]
    pub fn resume(&self) -> Result<()> {
        let err = unsafe { rt_thread_resume(self.raw) };
        RtError::from_code_none(err, ())
    }
}

impl Object for Thread {
    fn get_ptr(&self) -> NonNull<rt_object> {
        NonNull::new(self.raw.cast()).expect("Unexpected null rt_thread_t")
    }
}
