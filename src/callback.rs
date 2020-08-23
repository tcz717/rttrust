use crate::Box;
use cty::c_void;

pub unsafe extern "C" fn callback_entry(parameter: *mut c_void) {
    let closure: Box<Box<dyn FnOnce() + 'static>> = Box::from_raw(parameter.cast());
    closure();
}

#[derive(Debug)]
pub struct CallbackParameter(*mut c_void);

impl CallbackParameter {
    pub fn get_ptr_mut(self) -> *mut c_void {
        self.0
    }
}

impl From<usize> for CallbackParameter {
    fn from(data: usize) -> Self {
        CallbackParameter(data as *mut () as *mut c_void)
    }
}
impl From<*mut c_void> for CallbackParameter {
    fn from(data: *mut c_void) -> Self {
        CallbackParameter(data)
    }
}
impl From<Box<Box<dyn FnOnce() + 'static>>> for CallbackParameter {
    fn from(data: Box<Box<dyn FnOnce() + 'static>>) -> Self {
        CallbackParameter(Box::into_raw(data).cast())
    }
}

pub trait Callback {
    fn into_parameter(self) -> CallbackParameter;
}

impl<F: FnOnce() + 'static> Callback for F {
    fn into_parameter(self) -> CallbackParameter {
        let closure: Box<dyn FnOnce() + 'static> = Box::new(self);
        Box::new(closure).into()
    }
}
