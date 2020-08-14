use crate::ffi::*;
use crate::{Result, RtError};
use core::{any::Any, ptr::NonNull};

#[cfg(feature = "alloc")]
use crate::{cstr::RtName, Box};

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
    /// TODO: add args trait
    fn control(&mut self, device: &mut Device, cmd: i32, args: ()) -> Result<()>;

    #[inline]
    fn get_block_size(&self) -> usize {
        512
    }
}

unsafe extern "C" fn init_warper(dev: rt_device_t) -> rt_err_t {
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

unsafe extern "C" fn open_warper(dev: rt_device_t, oflag: u16) -> rt_err_t {
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

unsafe extern "C" fn close_warper(dev: rt_device_t) -> rt_err_t {
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

unsafe extern "C" fn read_warper(
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

    if let Ok(r) = userdata.as_mut().read(
        &mut Device {
            raw: device.as_ptr(),
        },
        pos as usize,
        core::slice::from_raw_parts_mut(buffer.cast(), size),
        block_size,
    ) {
        r as rt_size_t
    } else {
        // set errno
        0
    }
}

unsafe extern "C" fn write_warper(
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

    if let Ok(r) = userdata.as_mut().write(
        &mut Device {
            raw: device.as_ptr(),
        },
        pos as usize,
        core::slice::from_raw_parts(buffer.cast(), size),
        block_size,
    ) {
        r as rt_size_t
    } else {
        // set errno
        0
    }
}

unsafe extern "C" fn control_warper(
    dev: rt_device_t,
    cmd: cty::c_int,
    _args: *mut cty::c_void,
) -> rt_err_t {
    let device = NonNull::new(dev).expect("Null device ptr");
    let mut userdata: NonNull<Box<dyn DeviceOps>> =
        NonNull::new(device.as_ref().user_data.cast()).expect("Null device userdata");

    if let Err(err) = userdata.as_mut().control(
        &mut Device {
            raw: device.as_ptr(),
        },
        cmd,
        (),
    ) {
        err.to_code()
    } else {
        0
    }
}

impl Device {
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
            device.init = Some(init_warper);
            device.open = Some(open_warper);
            device.close = Some(close_warper);
            device.read = Some(read_warper);
            device.write = Some(write_warper);
            device.control = Some(control_warper)
        }

        Ok(Self {
            raw: device.as_ptr(),
        })
    }

    /// Must used on the device created by [create](#method.create)
    pub unsafe fn destroy(self) {
        if let Some(userdata) = NonNull::new(self.raw)
            .and_then(|dev| NonNull::<Box<dyn DeviceOps>>::new(dev.as_ref().user_data.cast()))
        {
            drop(userdata.as_ptr().read());
            rt_device_destroy(self.raw);
        }
    }

    #[inline]
    pub fn register(&self, name: &str, flags: RegisterFlag) -> Result<()> {
        let name = RtName::from(name);
        let err = unsafe { rt_device_register(self.raw, name.into(), flags.bits()) };
        RtError::from_code_none(err, ())
    }

    #[inline]
    pub fn unregister(&self) -> Result<()> {
        let err = unsafe { rt_device_unregister(self.raw) };
        RtError::from_code_none(err, ())
    }

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

    #[inline]
    pub fn init(&self) -> Result<()> {
        let err = unsafe { rt_device_init(self.raw) };
        RtError::from_code_none(err, ())
    }

    #[inline]
    pub fn open(&self, oflag: OpenFlag) -> Result<()> {
        let err = unsafe { rt_device_open(self.raw, oflag.bits()) };
        RtError::from_code_none(err, ())
    }

    #[inline]
    pub fn close(&self) -> Result<()> {
        let err = unsafe { rt_device_close(self.raw) };
        RtError::from_code_none(err, ())
    }

    #[inline]
    pub fn read(&self, pos: usize, buffer: &mut [u8], size: usize) -> Result<usize> {
        let res = unsafe {
            rt_device_read(
                self.raw,
                pos as rt_off_t,
                buffer.as_mut_ptr().cast(),
                size as rt_size_t,
            )
        };
        // TODO: read errno
        Ok(res as usize)
    }

    #[inline]
    pub fn write(&self, pos: usize, buffer: &[u8], size: usize) -> Result<usize> {
        let res = unsafe {
            rt_device_write(
                self.raw,
                pos as rt_off_t,
                buffer.as_ptr().cast(),
                size as rt_size_t,
            )
        };
        // TODO: read errno
        Ok(res as usize)
    }

    #[inline]
    /// TODO: add args trait
    pub fn control(&self, cmd: i32, _args: ()) -> Result<()> {
        let err = unsafe { rt_device_control(self.raw, cmd, core::ptr::null_mut()) };
        RtError::from_code_none(err, ())
    }
}
