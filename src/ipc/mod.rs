//! RT-Thread uses thread semaphores, mutexes, and event sets to achieve inter-thread synchronization. Thread synchronizes through the acquisition and release of semaphore and mutexes; the mutex uses priority inheritance to solve the common priority inversion problem in the real-time system. The thread synchronization mechanism allows threads to wait according to priorities or to acquire semaphores or mutexes following the first-in first-out method. Threads synchronize through sending and receiving of events; event sets allows "or trigger" and "and trigger" for multiple events, suitable for situations where threads are waiting for multiple events.

//! The concepts of semaphores, mutexes, and event sets are detailed in the "Inter-Thread Synchronization" chapter.
//! ### TODO
//! 1. rt_event

pub mod mutex;
pub mod sem;

use crate::ffi::{rt_enter_critical, rt_exit_critical, RT_IPC_FLAG_FIFO, RT_IPC_FLAG_PRIO};
use crate::Result;
use core::{
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
};

#[derive(Copy, Clone)]
pub enum IpcFlag {
    Fifo,
    Priority,
}

impl From<IpcFlag> for u8 {
    #[inline]
    fn from(flag: IpcFlag) -> Self {
        (match flag {
            IpcFlag::Fifo => RT_IPC_FLAG_FIFO,
            IpcFlag::Priority => RT_IPC_FLAG_PRIO,
        }) as u8
    }
}

#[must_use = "if unused the SpinLock will immediately unlock"]
pub struct SpinLock<T: ?Sized> {
    data: UnsafeCell<T>,
}

pub struct SpinLockGuard<'a, T: ?Sized + 'a> {
    lock: &'a SpinLock<T>,
}

unsafe impl<T: ?Sized + Send> Send for SpinLock<T> {}
unsafe impl<T: ?Sized + Send> Sync for SpinLock<T> {}

impl<T> SpinLock<T> {
    pub const fn new(data: T) -> Self {
        Self {
            data: UnsafeCell::new(data),
        }
    }

    pub fn lock(&self) -> SpinLockGuard<'_, T> {
        unsafe {
            rt_enter_critical();
        }
        SpinLockGuard { lock: self }
    }

    pub fn into_inner(self) -> Result<T>
    where
        T: Sized,
    {
        // We know statically that there are no outstanding references to
        // `self` so there's no need to lock the inner mutex.
        Ok(self.data.into_inner())
    }

    pub fn get_mut(&mut self) -> Result<&mut T> {
        // We know statically that there are no other references to `self`, so
        // there's no need to lock the inner mutex.
        Ok(unsafe { &mut *self.data.get() })
    }
}

impl<T: ?Sized> Deref for SpinLockGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.lock.data.get() }
    }
}

impl<T: ?Sized> DerefMut for SpinLockGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.lock.data.get() }
    }
}
impl<T: ?Sized> Drop for SpinLockGuard<'_, T> {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            rt_exit_critical();
        }
    }
}
