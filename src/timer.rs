//! Timer refers to triggering an event after a certain specified time from a specified moment, for example, setting a timer to wake up yourself the next morning. Timer includes hardware timer and software timer

//! ### TODO
//! Timer control 

use crate::ffi::{
    rt_object, rt_tick_from_millisecond, rt_tick_t, rt_timer, rt_timer_create, rt_timer_delete,
    rt_timer_detach, rt_timer_init, rt_timer_start, rt_timer_stop, rt_timer_t,
    RT_TIMER_FLAG_HARD_TIMER, RT_TIMER_FLAG_ONE_SHOT, RT_TIMER_FLAG_PERIODIC,
    RT_TIMER_FLAG_SOFT_TIMER,
};
use crate::{cstr::RtName, Result, RtError,object::Object};

use core::{cell::UnsafeCell, ffi::c_void, marker::PhantomPinned, mem::MaybeUninit, ptr::NonNull};

#[cfg(feature = "alloc")]
use crate::Box;

bitflags! {
    pub struct TimerFlags: u8 {
        const ONE_SHOT = RT_TIMER_FLAG_ONE_SHOT as u8;
        const PERIODIC = RT_TIMER_FLAG_PERIODIC as u8;
        const HARD_TIMER = RT_TIMER_FLAG_HARD_TIMER as u8;
        const SOFT_TIMER = RT_TIMER_FLAG_SOFT_TIMER as u8;
    }
}

#[derive(Debug)]
pub struct TimerParameter(*mut c_void);

pub type TimerEntry<P> = unsafe extern "C" fn(P);

impl From<usize> for TimerParameter {
    fn from(data: usize) -> Self {
        TimerParameter(data as *mut () as *mut c_void)
    }
}
impl From<*mut c_void> for TimerParameter {
    fn from(data: *mut c_void) -> Self {
        TimerParameter(data)
    }
}
impl From<Box<Box<dyn FnOnce() + 'static>>> for TimerParameter {
    fn from(data: Box<Box<dyn FnOnce() + 'static>>) -> Self {
        TimerParameter(Box::into_raw(data).cast())
    }
}

pub struct TimerStatic {
    raw: UnsafeCell<MaybeUninit<rt_timer>>,
    _pinned: PhantomPinned,
}

unsafe impl Send for TimerStatic {}
/// Will be removed once find a way to static init
unsafe impl Sync for TimerStatic {}

impl TimerStatic {
    pub const fn new() -> Self {
        TimerStatic {
            raw: UnsafeCell::new(core::mem::MaybeUninit::uninit()),
            _pinned: PhantomPinned {},
        }
    }

    pub fn init<P>(
        &'static self,
        name: &str,
        entry: TimerEntry<P>,
        parameter: P,
        time: u32,
        flag: TimerFlags,
    ) where
        P: Into<TimerParameter>,
    {
        let name: RtName = name.into();
        unsafe {
            rt_timer_init(
                self.raw.get().cast(),
                name.into(),
                Some(core::mem::transmute(entry)),
                Into::<TimerParameter>::into(parameter).0,
                time,
                flag.bits(),
            )
        }
    }

    /// ### Wanring:
    /// Rust timer has no cleanup function and is not able to unwind.
    /// Directly stopping a Rust timer may lead to resoucre leak.
    #[inline]
    pub fn detach(&'static self) -> Result<()> {
        let err = unsafe { rt_timer_detach(self.raw.get().cast()) };
        RtError::from_code_none(err, ())
    }

    pub fn get(&'static self) -> Timer {
        Timer {
            raw: self.raw.get().cast(),
        }
    }
}

/// All methods use immutable self since the safety is guaranteed by rt-thread internal
///
/// TODO: timer control
#[derive(Copy, Clone)]
pub struct Timer {
    raw: rt_timer_t,
}

unsafe impl Send for Timer {}

impl Timer {
    pub fn create<EP, P>(
        name: &str,
        entry: TimerEntry<EP>,
        parameter: P,
        time: u32,
        flag: TimerFlags,
    ) -> Result<Timer>
    where
        EP: Into<TimerParameter>,
        P: Into<TimerParameter>,
    {
        let name: RtName = name.into();
        let result = unsafe {
            rt_timer_create(
                name.into(),
                Some(core::mem::transmute(entry)),
                Into::<TimerParameter>::into(parameter).0,
                time,
                flag.bits(),
            )
        };
        if result.is_null() {
            Err(RtError::Error)
        } else {
            Ok(Timer { raw: result })
        }
    }

    #[cfg(feature = "alloc")]
    #[inline]
    pub fn create_closure<F>(name: &str, entry: F, time: u32, flag: TimerFlags) -> Result<Timer>
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
            time,
            flag,
        )
    }

    #[inline]
    pub fn start(&self) -> Result<()> {
        let err = unsafe { rt_timer_start(self.raw) };
        RtError::from_code_none(err, ())
    }

    /// ### Wanring:
    /// Rust timer has no cleanup function and is not able to unwind.
    /// Directly stopping a Rust timer may lead to resoucre leak.
    #[inline]
    pub fn delete(self) -> Result<()> {
        let err = unsafe { rt_timer_delete(self.raw) };
        RtError::from_code_none(err, ())
    }

    /// ### Wanring:
    /// Rust timer has no cleanup function and is not able to unwind.
    /// Directly stopping a Rust timer may lead to resoucre leak.
    #[inline]
    pub fn stop(&self) -> Result<()> {
        let err = unsafe { rt_timer_stop(self.raw) };
        RtError::from_code_none(err, ())
    }
}

impl Object for Timer {
    #[inline]
    fn get_ptr(&self) -> NonNull<rt_object> {
        NonNull::new(self.raw.cast()).expect("Unexpected null rt_timer_t")
    }
}

#[inline]
pub fn from_millisecond(ms: i32) -> rt_tick_t {
    unsafe { rt_tick_from_millisecond(ms) }
}
