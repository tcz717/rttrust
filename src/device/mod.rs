use crate::ffi::*;
use crate::{Result, RtError};
use core::{any::Any, ptr::NonNull};

#[cfg(feature = "alloc")]
use crate::{cstr::RtName, Box};
use cty::c_void;

mod io;
#[cfg(feature = "io")]
pub use io::*;

mod cmd;
pub use cmd::*;

pub type DeviceType = rt_device_class_type;

bitflags! {
    pub struct OpenFlag: u16 {
        /// 设备已经关闭（内部使用）
        const CLOSE  = RT_DEVICE_OFLAG_CLOSE  as u16;
        /// 以只读方式打开设备
        const RDONLY = RT_DEVICE_OFLAG_RDONLY as u16;
        /// 以只写方式打开设备
        const WRONLY = RT_DEVICE_OFLAG_WRONLY as u16;
        /// 以读写方式打开设备
        const RDWR   = RT_DEVICE_OFLAG_RDWR   as u16;
        /// 设备已经打开（内部使用）
        const OPEN   = RT_DEVICE_OFLAG_OPEN   as u16;
        /// 设备以流模式打开
        const STREAM  = RT_DEVICE_FLAG_STREAM  as u16;
        /// 设备以中断接收模式打开
        const INT_RX  = RT_DEVICE_FLAG_INT_RX  as u16;
        /// 设备以 DMA 接收模式打开
        const DMA_RX  = RT_DEVICE_FLAG_DMA_RX  as u16;
        /// 设备以中断发送模式打开
        const INT_TX  = RT_DEVICE_FLAG_INT_TX  as u16;
        /// 设备以 DMA 发送模式打开
        const DMA_TX  = RT_DEVICE_FLAG_DMA_TX  as u16;
    }
}

bitflags! {
    pub struct RegisterFlag: u16 {
        /// 只读
        const RDONLY     = RT_DEVICE_FLAG_RDONLY     as u16;
        /// 只写
        const WRONLY     = RT_DEVICE_FLAG_WRONLY     as u16;
        /// 读写
        const RDWR       = RT_DEVICE_FLAG_RDWR       as u16;
        /// 可移除
        const REMOVABLE  = RT_DEVICE_FLAG_REMOVABLE  as u16;
        /// 独立
        const STANDALONE = RT_DEVICE_FLAG_STANDALONE as u16;
        /// 挂起
        const SUSPENDED  = RT_DEVICE_FLAG_SUSPENDED  as u16;
        /// 流模式
        const STREAM     = RT_DEVICE_FLAG_STREAM     as u16;
        /// 中断接收
        const INT_RX     = RT_DEVICE_FLAG_INT_RX     as u16;
        /// DMA 接收
        const DMA_RX     = RT_DEVICE_FLAG_DMA_RX     as u16;
        /// 中断发送
        const INT_TX     = RT_DEVICE_FLAG_INT_TX     as u16;
        /// DMA 发送
        const DMA_TX     = RT_DEVICE_FLAG_DMA_TX     as u16;
    }
}

#[derive(Debug)]
#[repr(transparent)]
pub struct Device {
    raw: rt_device_t,
}

pub trait DeviceOps: Any + Send + Sync {
    fn init(&mut self, device: &mut Device) -> Result<()>;
    fn open(&mut self, device: &mut Device, oflag: OpenFlag) -> Result<()>;
    fn close(&mut self, device: &mut Device) -> Result<()>;
    fn read(
        &mut self,
        device: &mut Device,
        pos: usize,
        buffer: &mut [u8],
        size: usize,
    ) -> Result<usize>;
    fn write(
        &mut self,
        device: &mut Device,
        pos: usize,
        buffer: &[u8],
        size: usize,
    ) -> Result<usize>;

    fn control(&mut self, device: &mut Device, cmd: i32, args: *mut c_void) -> Result<()>;

    #[inline]
    fn get_block_size(&self) -> usize {
        1
    }
}

unsafe extern "C" fn init_wrapper(dev: rt_device_t) -> rt_err_t {
    let device = NonNull::new(dev).expect("Null device ptr");
    let mut userdata: NonNull<Box<dyn DeviceOps>> =
        NonNull::new(device.as_ref().user_data.cast()).expect("Null device userdata");

    if let Err(err) = userdata.as_mut().init(&mut Device {
        raw: device.as_ptr(),
    }) {
        err.to_code()
    } else {
        0
    }
}

unsafe extern "C" fn open_wrapper(dev: rt_device_t, oflag: u16) -> rt_err_t {
    let device = NonNull::new(dev).expect("Null device ptr");
    let mut userdata: NonNull<Box<dyn DeviceOps>> =
        NonNull::new(device.as_ref().user_data.cast()).expect("Null device userdata");

    if let Err(err) = userdata.as_mut().open(
        &mut Device {
            raw: device.as_ptr(),
        },
        OpenFlag::from_bits_truncate(oflag),
    ) {
        err.to_code()
    } else {
        0
    }
}

unsafe extern "C" fn close_wrapper(dev: rt_device_t) -> rt_err_t {
    let device = NonNull::new(dev).expect("Null device ptr");
    let mut userdata: NonNull<Box<dyn DeviceOps>> =
        NonNull::new(device.as_ref().user_data.cast()).expect("Null device userdata");

    if let Err(err) = userdata.as_mut().close(&mut Device {
        raw: device.as_ptr(),
    }) {
        err.to_code()
    } else {
        0
    }
}

unsafe extern "C" fn read_wrapper(
    dev: rt_device_t,
    pos: rt_off_t,
    buffer: *mut cty::c_void,
    size: rt_size_t,
) -> rt_size_t {
    let block_size = size as usize;
    let device = NonNull::new(dev).expect("Null device ptr");
    let mut userdata: NonNull<Box<dyn DeviceOps>> =
        NonNull::new(device.as_ref().user_data.cast()).expect("Null device userdata");
    let size = block_size * userdata.as_ref().get_block_size();

    match userdata.as_mut().read(
        &mut Device {
            raw: device.as_ptr(),
        },
        (pos as usize) * block_size,
        core::slice::from_raw_parts_mut(buffer.cast(), size),
        block_size,
    ) {
        Ok(r) => r as rt_size_t,
        Err(err) => {
            rt_set_errno(err.to_code());
            0
        }
    }
}

unsafe extern "C" fn write_wrapper(
    dev: rt_device_t,
    pos: rt_off_t,
    buffer: *const cty::c_void,
    size: rt_size_t,
) -> rt_size_t {
    let block_size = size as usize;
    let device = NonNull::new(dev).expect("Null device ptr");
    let mut userdata: NonNull<Box<dyn DeviceOps>> =
        NonNull::new(device.as_ref().user_data.cast()).expect("Null device userdata");
    let size = block_size * userdata.as_ref().get_block_size();

    match userdata.as_mut().write(
        &mut Device {
            raw: device.as_ptr(),
        },
        (pos as usize) * block_size,
        core::slice::from_raw_parts(buffer.cast(), size),
        block_size,
    ) {
        Ok(r) => r as rt_size_t,
        Err(err) => {
            rt_set_errno(err.to_code());
            0
        }
    }
}

unsafe extern "C" fn control_wrapper(
    dev: rt_device_t,
    cmd: cty::c_int,
    args: *mut cty::c_void,
) -> rt_err_t {
    let device = NonNull::new(dev).expect("Null device ptr");
    let mut userdata: NonNull<Box<dyn DeviceOps>> =
        NonNull::new(device.as_ref().user_data.cast()).expect("Null device userdata");

    if let Err(err) = userdata.as_mut().control(
        &mut Device {
            raw: device.as_ptr(),
        },
        cmd,
        args,
    ) {
        err.to_code()
    } else {
        0
    }
}

impl Device {
    ///
    /// This function creates a device object with user data size.
    ///
    /// @param type, the kind type of this device object.
    /// @param attach_size, the size of user data.
    ///
    /// @return the allocated device object, or RT_NULL when failed.
    ///
    #[inline]
    pub unsafe fn create_uninit(type0: DeviceType, attach_size: usize) -> Result<Self> {
        NonNull::new(rt_device_create(type0.0 as i32, attach_size as i32))
            .map(|raw| Self { raw: raw.as_ptr() })
            .ok_or(RtError::Error)
    }

    /// TODO avoid multiple allocation
    #[cfg(feature = "alloc")]
    pub fn create<O>(type0: DeviceType) -> Result<Self>
    where
        O: DeviceOps + Default,
    {
        let device = unsafe { rt_device_create(type0.0 as i32, 0) };

        let mut device: NonNull<rt_device> = NonNull::new(device).ok_or(RtError::Error)?;
        let userdata: Box<Box<dyn DeviceOps>> = Box::new(Box::new(O::default()));
        unsafe {
            let device = device.as_mut();
            device.user_data = Box::into_raw(userdata).cast();
            device.init = Some(init_wrapper);
            device.open = Some(open_wrapper);
            device.close = Some(close_wrapper);
            device.read = Some(read_wrapper);
            device.write = Some(write_wrapper);
            device.control = Some(control_wrapper)
        }

        Ok(Self {
            raw: device.as_ptr(),
        })
    }

    ///
    /// This function destroy the specific device object.
    ///
    /// Must used on the device created by [create](#method.create)
    pub unsafe fn destroy(self) {
        if let Some(userdata) = NonNull::new(self.raw)
            .and_then(|dev| NonNull::<Box<dyn DeviceOps>>::new(dev.as_ref().user_data.cast()))
        {
            drop(userdata.as_ptr().read());
            rt_device_destroy(self.raw);
        }
    }

    ///
    /// This function registers a device driver with specified name.
    ///
    /// @param name the device driver's name
    /// @param flags the capabilities flag of device
    ///
    /// @return the error code, RT_EOK on initialization successfully.
    ///
    #[inline]
    pub fn register(&self, name: &str, flags: RegisterFlag) -> Result<()> {
        let name = RtName::from(name);
        let err = unsafe { rt_device_register(self.raw, name.into(), flags.bits()) };
        RtError::from_code_none(err, ())
    }

    ///
    /// This function removes a previously registered device driver
    ///
    /// @return the error code, RT_EOK on successfully.
    ///
    #[inline]
    pub fn unregister(&self) -> Result<()> {
        let err = unsafe { rt_device_unregister(self.raw) };
        RtError::from_code_none(err, ())
    }

    ///
    /// This function finds a device driver by specified name.
    ///
    /// @param name the device driver's name
    ///
    /// @return the registered device driver on successful, or RT_NULL on failure.
    ///
    #[inline]
    pub fn find(name: &str) -> Result<Self> {
        let name = RtName::from(name);
        let res = unsafe { rt_device_find(name.into()) };
        if res.is_null() {
            Ok(Self { raw: res })
        } else {
            Err(RtError::Error)
        }
    }

    ///
    /// This function will initialize the specified device
    ///
    /// @return the result
    ///
    #[inline]
    pub fn init(&mut self) -> Result<()> {
        let err = unsafe { rt_device_init(self.raw) };
        RtError::from_code_none(err, ())
    }

    ///
    /// This function will open a device
    ///
    /// @param oflag the flags for device open
    ///
    /// @return the result
    ///
    #[inline]
    pub fn open(&mut self, oflag: OpenFlag) -> Result<()> {
        let err = unsafe { rt_device_open(self.raw, oflag.bits()) };
        RtError::from_code_none(err, ())
    }

    ///
    /// This function will close a device
    ///
    /// @param dev the pointer of device driver structure
    ///
    /// @return the result
    ///
    #[inline]
    pub fn close(&mut self) -> Result<()> {
        let err = unsafe { rt_device_close(self.raw) };
        RtError::from_code_none(err, ())
    }

    ///
    /// This function will read some data from a device.
    ///
    /// @param pos the position of reading
    /// @param buffer the data buffer to save read data
    /// @param size the size of buffer
    ///
    /// @return the actually read size on successful, otherwise negative returned.
    ///
    /// @note since 0.4.0, the unit of size/pos is a block for block device.
    ///
    /// ## Safety
    /// 
    /// For block device, `size` parameter refers to the number of blocks. Therefore, 
    /// the size of buffer must be at least `size * BLOCK_SIZE`. `BLOCK_SIZE` can be 
    /// read through [GetGeometry](GetGeometry).
    /// 
    /// Providing wrong `size` and `buffer` will lead to memory overflow
    /// 
    #[inline]
    pub unsafe fn read(&mut self, pos: usize, buffer: &mut [u8], size: usize) -> Result<usize> {
        let res = rt_device_read(
            self.raw,
            pos as rt_off_t,
            buffer.as_mut_ptr().cast(),
            size as rt_size_t,
        );
        RtError::from_code_none(rt_get_errno(), res as usize)
    }

    ///
    /// This function will write some data to a device.
    ///
    /// @param pos the position of written
    /// @param buffer the data buffer to be written to device
    /// @param size the size of buffer
    ///
    /// @return the actually written size on successful, otherwise negative returned.
    ///
    /// @note since 0.4.0, the unit of size/pos is a block for block device.
    ///
    /// ## Safety
    /// 
    /// For block device, `size` parameter refers to the number of blocks. Therefore, 
    /// the size of buffer must be at least `size * BLOCK_SIZE`. `BLOCK_SIZE` can be 
    /// read through [GetGeometry](GetGeometry).
    /// 
    /// Providing wrong `size` and `buffer` will lead to memory overflow
    /// 
    #[inline]
    pub unsafe fn write(&mut self, pos: usize, buffer: &[u8], size: usize) -> Result<usize> {
        let res = rt_device_write(
            self.raw,
            pos as rt_off_t,
            buffer.as_ptr().cast(),
            size as rt_size_t,
        );
        RtError::from_code_none(rt_get_errno(), res as usize)
    }

    ///
    /// This function will perform a variety of control functions on devices.
    ///
    /// @param cmd the command sent to device
    ///
    /// @return the result
    ///
    #[inline]
    pub fn control<R>(&mut self, cmd: &mut dyn DeviceCommand<Return = R>) -> Result<()> {
        let err = unsafe { rt_device_control(self.raw, cmd.get_cmd(), cmd.get_arg()) };
        RtError::from_code_none(err, ())
    }

    #[inline]
    pub fn get_device_type(&self) -> DeviceType {
        unsafe { (*self.raw).type_ }
    }
}
