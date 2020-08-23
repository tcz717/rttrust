//! Timer refers to triggering an event after a certain specified time from a specified moment. 
//! 
//! For example, setting a timer to wake up yourself the next morning. Timer includes hardware timer and software timer

//! ### TODO
//! Timer control

use crate::ffi::{
    rt_object, rt_tick_from_millisecond, rt_tick_t, rt_timer, rt_timer_create, rt_timer_delete,
    rt_timer_detach, rt_timer_init, rt_timer_start, rt_timer_stop, rt_timer_t,
    RT_TIMER_FLAG_HARD_TIMER, RT_TIMER_FLAG_ONE_SHOT, RT_TIMER_FLAG_PERIODIC,
    RT_TIMER_FLAG_SOFT_TIMER,
};
use crate::{cstr::RtName, object::Object, Result, RtError};

use core::{cell::UnsafeCell, marker::PhantomPinned, mem::MaybeUninit, ptr::NonNull, ops::Deref};

#[cfg(feature = "alloc")]
use crate::callback::{callback_entry, Callback, CallbackParameter};
use FnOnce;

bitflags! {
    pub struct TimerFlags: u8 {
        const ONE_SHOT = RT_TIMER_FLAG_ONE_SHOT as u8;
        const PERIODIC = RT_TIMER_FLAG_PERIODIC as u8;
        const HARD_TIMER = RT_TIMER_FLAG_HARD_TIMER as u8;
        const SOFT_TIMER = RT_TIMER_FLAG_SOFT_TIMER as u8;
    }
}

pub type TimerEntry<P> = unsafe extern "C" fn(P);

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

    ///
    /// This function will initialize a timer, normally this function is used to
    /// initialize a static timer object.
    ///
    /// @param name the name of timer
    /// @param timeout the timeout function
    /// @param parameter the parameter of timeout function
    /// @param time the tick of timer
    /// @param flag the flag of timer
    ///
    pub fn init<P>(
        &'static self,
        name: &str,
        entry: TimerEntry<P>,
        parameter: CallbackParameter,
        time: u32,
        flag: TimerFlags,
    ) where
        P: Into<CallbackParameter>,
    {
        let name: RtName = name.into();
        unsafe {
            rt_timer_init(
                self.raw.get().cast(),
                name.into(),
                Some(core::mem::transmute(entry)),
                parameter.get_ptr_mut(),
                time,
                flag.bits(),
            )
        }
    }

    ///
    /// This function will detach a timer from timer management.
    ///
    /// @param timer the static timer object
    ///
    /// @return the operation status, RT_EOK on OK; RT_ERROR on error
    ///
    /// ### Wanring:
    /// Rust timer has no cleanup function and is not able to unwind.
    /// Directly stopping a Rust timer may lead to resoucre leak.
    #[inline]
    pub fn detach(&'static self) -> Result<()> {
        let err = unsafe { rt_timer_detach(self.raw.get().cast()) };
        RtError::from_code_none(err, ())
    }

    pub fn get_timer(&'static self) -> Timer {
        Timer {
            raw: self.raw.get().cast(),
        }
    }
}

impl Deref for TimerStatic {
    type Target = Timer;

    fn deref(&self) -> &Self::Target {
        unsafe { Timer::new(&mut *self.raw.get().cast()) }
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
    pub fn new(timer: &mut rt_timer) -> &Timer {
        unsafe { &*(timer as rt_timer_t as *mut Timer) }
    }
    ///
    /// This function will create a timer
    ///
    /// @param name the name of timer
    /// @param timeout the timeout function
    /// @param parameter the parameter of timeout function
    /// @param time the tick of timer
    /// @param flag the flag of timer
    ///
    /// @return the created timer object
    ///
    pub fn create<P>(
        name: &str,
        entry: TimerEntry<P>,
        parameter: CallbackParameter,
        time: u32,
        flag: TimerFlags,
    ) -> Result<Timer>
    where
        P: Into<CallbackParameter>,
    {
        let name: RtName = name.into();
        let result = unsafe {
            rt_timer_create(
                name.into(),
                Some(core::mem::transmute(entry)),
                parameter.get_ptr_mut(),
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
        Self::create(name, callback_entry, entry.into_parameter(), time, flag)
    }

    ///
    /// This function will start the timer
    ///
    /// @return the operation status, RT_EOK on OK, -RT_ERROR on error
    ///
    #[inline]
    pub fn start(&self) -> Result<()> {
        let err = unsafe { rt_timer_start(self.raw) };
        RtError::from_code_none(err, ())
    }

    ///
    /// This function will delete a timer and release timer memory
    ///
    /// @return the operation status, RT_EOK on OK; RT_ERROR on error
    ///
    /// ### Wanring:
    /// Rust timer has no cleanup function and is not able to unwind.
    /// Directly stopping a Rust timer may lead to resoucre leak.
    #[inline]
    pub fn delete(self) -> Result<()> {
        let err = unsafe { rt_timer_delete(self.raw) };
        RtError::from_code_none(err, ())
    }

    ///
    /// This function will stop the timer
    ///
    /// @return the operation status, RT_EOK on OK, -RT_ERROR on error
    ///
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
