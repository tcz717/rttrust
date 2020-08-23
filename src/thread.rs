//! Thread is the smallest scheduling unit in the RT-Thread operating system.
//! 
//! Thread scheduling algorithm is a priority-based full preemptive multi-thread scheduling algorithm, that is, except the interrupt handler, the code of the scheduler's locked part, and the code that prohibits the interrupt, other parts of the system can be preempted, including the thread scheduler itself. The system supports 256 thread priorities (can also be changed to a maximum of 32 or 8 thread priority via configuration file; for STM32 default configuration, it is set as 32 thread priorities), 0 priority represents highest priority, and lowest priority is reserved for idle threads; at the same time, it also supports creating multiple threads with the same priority. The same priority threads are scheduled with a time slice rotation scheduling algorithm, so that each thread runs for the same time; in addition, when the scheduler is looking for threads that are at the highest priority thread and ready, the elapsed time is constant. The system does not limit the number of threads, the number of threads is only related to the specific memory of the hardware platform.

//! ### TODO
//! 1. Thread control
//! 1. Extract ThreadParameter to reuse

use crate::ffi::{
    rt_object, rt_thread, rt_thread_create, rt_thread_delay, rt_thread_delete, rt_thread_detach,
    rt_thread_find, rt_thread_init, rt_thread_mdelay, rt_thread_resume, rt_thread_self,
    rt_thread_startup, rt_thread_suspend, rt_thread_t, rt_thread_yield, rt_tick_t,
};
use crate::{cstr::RtName, object::Object, Result, RtError};

use core::{
    cell::UnsafeCell, ffi::c_void, marker::PhantomPinned, mem::MaybeUninit, ops::Deref,
    ptr::NonNull,
};

#[cfg(feature = "alloc")]
use crate::callback::{callback_entry, Callback, CallbackParameter};
use FnOnce;

pub type ThreadEntry<P> = unsafe extern "C" fn(P);

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

    ///
    /// This function will initialize a thread, normally it's used to initialize a
    /// static thread object.
    ///
    /// @param name the name of thread, which shall be unique
    /// @param entry the entry function of thread
    /// @param parameter the parameter of thread enter function
    /// @param stack the thread stack slice
    /// @param priority the priority of thread
    /// @param tick the time slice if there are same priority thread
    ///
    /// @return the operation status, RT_EOK on OK, -RT_ERROR on error
    ///
    pub fn init<P>(
        &'static self,
        name: &str,
        entry: ThreadEntry<P>,
        parameter: CallbackParameter,
        stack: &'static mut [c_void],
        priority: u8,
        tick: u32,
    ) -> Result<()>
    where
        P: Into<CallbackParameter>,
    {
        let name: RtName = name.into();
        let err = unsafe {
            rt_thread_init(
                self.raw.get().cast(),
                name.into(),
                Some(core::mem::transmute(entry)),
                parameter.get_ptr_mut(),
                stack.as_mut_ptr(),
                stack.len() as u32,
                priority,
                tick,
            )
        };
        RtError::from_code_none(err, ())
    }

    ///
    /// This function will detach a thread. The thread object will be removed from
    /// thread queue and detached/deleted from system object management.
    ///
    /// @return the operation status, RT_EOK on OK, -RT_ERROR on error
    ///
    /// ### Wanring:
    /// Rust thread has no cleanup function and is not able to unwind.
    /// Directly stopping a Rust thread may lead to resoucre leak.
    #[inline]
    pub fn detach(&'static self) -> Result<()> {
        let err = unsafe { rt_thread_detach(self.raw.get().cast()) };
        RtError::from_code_none(err, ())
    }

    pub fn get_thread(&'static self) -> Thread {
        Thread {
            raw: self.raw.get().cast(),
        }
    }
}

impl Deref for ThreadStatic {
    type Target = Thread;

    fn deref(&self) -> &Self::Target {
        unsafe { Thread::new(&mut *self.raw.get().cast()) }
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
#[repr(transparent)]
pub struct Thread {
    raw: rt_thread_t,
}

unsafe impl Send for Thread {}

impl Thread {
    pub fn new(thread: &mut rt_thread) -> &Thread {
        unsafe { &*(thread as rt_thread_t as *mut Thread) }
    }
    ///
    /// This function will create a thread object and allocate thread object memory
    /// and stack.
    ///
    /// @param name the name of thread, which shall be unique
    /// @param entry the entry function of thread
    /// @param parameter the parameter of thread enter function
    /// @param stack_size the size of thread stack
    /// @param priority the priority of thread
    /// @param tick the time slice if there are same priority thread
    ///
    /// @return the created thread object
    ///
    pub fn create<P>(
        name: &str,
        entry: ThreadEntry<P>,
        parameter: CallbackParameter,
        stack_size: u32,
        priority: u8,
        tick: u32,
    ) -> Result<Thread>
    where
        P: Into<CallbackParameter>,
    {
        let name: RtName = name.into();
        let result = unsafe {
            rt_thread_create(
                name.into(),
                Some(core::mem::transmute(entry)),
                parameter.get_ptr_mut(),
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
        Self::create(
            name,
            callback_entry,
            entry.into_parameter(),
            stack_size,
            priority,
            tick,
        )
    }

    ///
    /// This function will return self thread object
    ///
    /// @return the self thread object
    ///
    #[inline]
    pub fn current() -> Result<Thread> {
        let result = unsafe { rt_thread_self() };
        if result.is_null() {
            Err(RtError::Error)
        } else {
            Ok(Thread { raw: result })
        }
    }

    ///
    /// This function will find the specified thread.
    ///
    /// @param name the name of thread finding
    ///
    /// @return the found thread
    ///
    /// @note please don't invoke this function in interrupt status.
    ///
    pub fn find(name: &str) -> Result<Thread> {
        let name: RtName = name.into();
        let result = unsafe { rt_thread_find(name.into()) };
        if result.is_null() {
            Err(RtError::Error)
        } else {
            Ok(Thread { raw: result })
        }
    }

    ///
    /// This function will start a thread and put it to system ready queue
    ///
    /// @param thread the thread to be started
    ///
    /// @return the operation status, RT_EOK on OK, -RT_ERROR on error
    ///
    #[inline]
    pub fn startup(&self) -> Result<()> {
        let err = unsafe { rt_thread_startup(self.raw) };
        RtError::from_code_none(err, ())
    }

    ///
    /// This function will delete a thread. The thread object will be removed from
    /// thread queue and deleted from system object management in the idle thread.
    ///
    /// @param thread the thread to be deleted
    ///
    /// @return the operation status, RT_EOK on OK, -RT_ERROR on error
    ///
    /// ### Wanring:
    /// Rust thread has no cleanup function and is not able to unwind.
    /// Directly stopping a Rust thread may lead to resoucre leak.
    #[inline]
    pub fn delete(self) -> Result<()> {
        let err = unsafe { rt_thread_delete(self.raw) };
        RtError::from_code_none(err, ())
    }

    ///
    /// This function will let current thread yield processor, and scheduler will
    /// choose a highest thread to run. After yield processor, the current thread
    /// is still in READY state.
    ///
    /// @return RT_EOK
    ///
    #[inline]
    pub fn yield0() -> Result<()> {
        let err = unsafe { rt_thread_yield() };
        RtError::from_code_none(err, ())
    }

    ///
    /// This function will let current thread delay for some ticks.
    ///
    /// @param tick the delay ticks
    ///
    /// @return RT_EOK
    ///
    #[inline]
    pub fn delay(tick: rt_tick_t) -> Result<()> {
        let err = unsafe { rt_thread_delay(tick) };
        RtError::from_code_none(err, ())
    }

    ///
    /// This function will let current thread delay for some milliseconds.
    ///
    /// @param ms the delay ms time
    ///
    /// @return RT_EOK
    ///
    #[inline]
    pub fn mdelay(ms: i32) -> Result<()> {
        let err = unsafe { rt_thread_mdelay(ms) };
        RtError::from_code_none(err, ())
    }

    ///
    /// This function will suspend the specified thread.
    ///
    /// @return the operation status, RT_EOK on OK, -RT_ERROR on error
    ///
    /// @note if suspend self thread, after this function call, the
    /// rt_schedule() must be invoked.
    ///
    #[inline]
    pub fn suspend(&self) -> Result<()> {
        let err = unsafe { rt_thread_suspend(self.raw) };
        RtError::from_code_none(err, ())
    }

    ///
    /// This function will resume a thread and put it to system ready queue.
    ///
    /// @param thread the thread to be resumed
    ///
    /// @return the operation status, RT_EOK on OK, -RT_ERROR on error
    ///
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
