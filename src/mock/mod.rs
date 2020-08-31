pub(crate) mod device {
    use crate::device::DeviceCommand;
    use crate::device::DeviceType;
    use crate::device::OpenFlag;
    use crate::device::RegisterFlag;
    use crate::Result;
    use core::fmt::Debug;
    use mockall::mock;

    mock! {
        pub Device {
            unsafe fn create_uninit(type0: DeviceType, attach_size: usize) -> Result<Self>;
            fn create<O:'static>(type0: DeviceType) -> Result<Self>;
            unsafe fn destroy(self);
            fn register(&self, name: &str, flags: RegisterFlag) -> Result<()>;
            fn unregister(&self) -> Result<()>;
            fn find(name: &str) -> Result<Self>;
            fn init(&mut self) -> Result<()>;
            fn open(&mut self, oflag: OpenFlag) -> Result<()>;
            fn close(&mut self) -> Result<()>;
            unsafe fn read(&mut self, pos: usize, buffer: &mut [u8], size: usize) -> Result<usize>;
            unsafe fn write(&mut self, pos: usize, buffer: &[u8], size: usize) -> Result<usize>;
            fn control<R:'static>(&mut self, cmd: &mut (dyn DeviceCommand<Return = R> + 'static)) -> Result<()>;
            fn get_device_type(&self) -> DeviceType;
        }
    }
    impl Debug for MockDevice {
        fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
            unimplemented!()
        }
    }
}
