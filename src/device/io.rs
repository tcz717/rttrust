#![cfg(feature = "io")]

cfg_if! {
    if #[cfg(test)] {
        use crate::mock::device::MockDevice as Device;
    } else {
        use crate::device::Device;
    }
}

use super::{DeviceCommand, DeviceType, GetGeometry};
use crate::{Result, RtError};
use genio::{Read, Write};

#[cfg(feature = "alloc")]
use crate::{
    io::{Seek, SeekFrom, WriteFmt},
    Box,
};
use core::cmp::min;
use core::convert::TryInto;

#[derive(Debug, PartialEq)]
enum BlockStatus {
    Uninit,
    Clean,
    Dirty,
}

/// Providing special device operation for [block device](crate::device::DeviceType)
#[cfg(feature = "alloc")]
#[derive(Debug)]
pub struct BlockDevice<'a> {
    device: &'a mut Device,
    byte_pos: usize,
    buf: Option<Box<[u8]>>,
    status: BlockStatus,
    block_pos: usize,
    block_size: usize,
}

#[cfg(feature = "alloc")]
impl<'a> BlockDevice<'a> {
    /// Create a BlockDevice for reading and writing.alloc
    ///
    /// If `device` is not [DeviceType](DeviceType)::RT_Device_Class_Block,
    /// the method will return `RtError::NoSys`.
    ///
    /// The device must support `RT_DEVICE_CTRL_BLK_GETGEOME` to get block size.
    pub fn new(device: &'a mut Device) -> Result<Self> {
        if device.get_device_type() != DeviceType::RT_Device_Class_Block {
            return Err(RtError::NoSys);
        }
        let block_size = GetGeometry::new().exec(device)?.block_size as usize;
        Ok(Self {
            device,
            byte_pos: 0,
            buf: None,
            status: BlockStatus::Uninit,
            block_pos: 0,
            block_size,
        })
    }
    #[inline]
    fn remain(&self) -> usize {
        self.block_size - self.byte_pos
    }
    #[inline]
    fn is_end(&self) -> bool {
        self.block_size == self.byte_pos
    }
    #[inline]
    fn is_begin(&self) -> bool {
        self.byte_pos == 0
    }

    fn write_block(&mut self, data: &[u8]) -> Result<usize> {
        let write_len = min(data.len(), self.remain());
        let block_size = self.block_size;
        let buf = self
            .buf
            .get_or_insert_with(move || vec![0; block_size].into_boxed_slice())
            .as_mut();
        if self.status == BlockStatus::Uninit {
            unsafe {
                self.device.read(self.block_pos, buf.as_mut(), 1)?;
            }
        }
        buf[self.byte_pos..self.byte_pos + write_len].copy_from_slice(&data[..write_len]);
        self.byte_pos += write_len;
        self.status = BlockStatus::Dirty;
        Ok(write_len)
    }

    fn read_block(&mut self, data: &mut [u8]) -> Result<usize> {
        let read_len = min(data.len(), self.remain());
        let block_size = self.block_size;
        let buf = self
            .buf
            .get_or_insert_with(move || vec![0; block_size].into_boxed_slice())
            .as_mut();
        if self.status == BlockStatus::Uninit {
            unsafe {
                self.device.read(self.block_pos, buf.as_mut(), 1)?;
            }
        }
        data[..read_len].copy_from_slice(&buf[self.byte_pos..self.byte_pos + read_len]);
        self.byte_pos += read_len;
        self.status = BlockStatus::Clean;
        Ok(read_len)
    }
    fn forward(&mut self, steps: isize) -> Result<()> {
        let mut steps = steps;
        if self.is_end() {
            steps += 1;
            self.byte_pos = 0;
        }
        if steps == 0 {
            return Ok(());
        }
        if self.status == BlockStatus::Dirty {
            if let Some(ref buf) = self.buf {
                unsafe {
                    self.device.write(self.block_pos, buf.as_ref(), 1)?;
                }
            }
        }
        self.status = BlockStatus::Uninit;
        if steps.is_positive() {
            self.block_pos = self
                .block_pos
                .checked_add(steps as usize)
                .ok_or(RtError::Inval)?;
        } else if steps.is_negative() {
            self.block_pos = self
                .block_pos
                .checked_sub(steps.abs() as usize)
                .ok_or(RtError::Inval)?;
        }
        Ok(())
    }
}

#[cfg(feature = "alloc")]
impl Write for BlockDevice<'_> {
    type WriteError = RtError;

    type FlushError = RtError;

    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let mut written = 0;
        let mut buf = buf;
        if !self.is_begin() {
            written += self.write_block(buf)?;
            buf = &buf[written..];
            self.forward(0)?;
        }
        let mut forward = 0;
        while buf.len() >= self.block_size {
            let block_num = buf.len() / self.block_size;
            let block_written = unsafe {
                self.device.write(
                    self.block_pos,
                    &buf[..block_num * self.block_size],
                    block_num,
                )?
            };
            written += block_written * self.block_size;
            buf = &buf[block_written * self.block_size..];
            forward += block_num;
        }
        self.forward(forward as isize)?;
        if !buf.is_empty() {
            written += self.write_block(buf)?;
        }

        Ok(written)
    }

    fn flush(&mut self) -> Result<()> {
        let device = &mut self.device;
        if self.status == BlockStatus::Dirty {
            unsafe {
                device.write(self.block_pos, self.buf.as_ref().unwrap(), 1)?;
            }
            self.status = BlockStatus::Clean;
        }
        Ok(())
    }

    #[inline]
    fn size_hint(&mut self, _bytes: usize) {}
}

impl Read for BlockDevice<'_> {
    type ReadError = RtError;

    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let mut read_len = 0;
        let mut buf = buf;
        if !self.is_begin() {
            read_len += self.read_block(buf)?;
            buf = &mut buf[read_len..];
            self.forward(0)?;
        }
        let mut forward = 0;
        while buf.len() >= self.block_size {
            let block_num = buf.len() / self.block_size;
            let block_read = unsafe {
                self.device.read(
                    self.block_pos,
                    &mut buf[..block_num * self.block_size],
                    block_num,
                )?
            };
            read_len += block_read * self.block_size;
            buf = &mut buf[block_read * self.block_size..];
            forward += block_num;
        }
        self.forward(forward as isize)?;
        if !buf.is_empty() {
            read_len += self.read_block(buf)?;
        }

        Ok(read_len)
    }
}

impl Seek for BlockDevice<'_> {
    fn seek(&mut self, pos: SeekFrom) -> Result<i64> {
        let offset: usize = match pos {
            SeekFrom::Start(pos) => pos.try_into().map_err(|_| RtError::Inval)?,
            SeekFrom::End(_) => return Err(RtError::Inval),
            SeekFrom::Current(pos) => ((self.byte_pos + self.block_pos * self.block_size) as i64
                + pos)
                .try_into()
                .map_err(|_| RtError::Inval)?,
        };
        let new_block_pos = offset / self.block_size;
        let new_byte_size = offset % self.block_size;
        if new_block_pos != self.block_pos {
            self.flush()?;
            self.block_pos = new_block_pos;
            self.status = BlockStatus::Uninit;
        }
        self.byte_pos = new_byte_size;
        Ok(offset as i64)
    }
}

impl WriteFmt for BlockDevice<'_> {}

/// Providing regular device operation for non-[block device](crate::device::DeviceType)
pub struct CharDevice<'a> {
    device: &'a mut Device,
    pos: usize,
}

impl<'a> CharDevice<'a> {
    /// Create a CharDevice for reading and writing non-block device.
    ///
    /// If `device` is [DeviceType](DeviceType)::RT_Device_Class_Block,
    /// the method will return `RtError::NoSys`.
    pub fn new(device: &'a mut Device) -> Result<Self> {
        if device.get_device_type() == DeviceType::RT_Device_Class_Block {
            return Err(RtError::NoSys);
        }
        Ok(Self { device, pos: 0 })
    }
}
impl Write for CharDevice<'_> {
    type WriteError = RtError;

    type FlushError = RtError;

    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let read_len = unsafe { self.device.write(self.pos, buf, buf.len())? };
        self.pos += read_len;
        Ok(read_len)
    }

    fn flush(&mut self) -> Result<()> {
        Ok(())
    }

    fn size_hint(&mut self, _bytes: usize) {}
}

impl Read for CharDevice<'_> {
    type ReadError = RtError;

    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let read_len = unsafe { self.device.read(self.pos, buf, buf.len())? };
        self.pos += read_len;
        Ok(read_len)
    }
}

impl Seek for CharDevice<'_> {
    fn seek(&mut self, pos: SeekFrom) -> Result<i64> {
        let offset: usize = match pos {
            SeekFrom::Start(pos) => pos.try_into().map_err(|_| RtError::Inval)?,
            SeekFrom::End(_) => return Err(RtError::Inval),
            SeekFrom::Current(pos) => (self.pos as i64 + pos)
                .try_into()
                .map_err(|_| RtError::Inval)?,
        };
        self.pos = offset;
        Ok(offset as i64)
    }
}

impl WriteFmt for CharDevice<'_> {}

#[cfg(test)]
#[allow(unused_imports)]
mod test {
    use super::BlockDevice;
    use crate::ffi::rt_device_blk_geometry;
    use crate::{device::DeviceType, mock::device::MockDevice, Result};
    use genio::{Read, Write};
    use mockall::predicate::*;

    fn setup_block_device(block_size: usize) -> MockDevice {
        let mut device = MockDevice::new();
        device
            .expect_get_device_type()
            .returning(|| DeviceType::RT_Device_Class_Block);
        device
            .expect_control::<rt_device_blk_geometry>()
            .returning(move |cmd| {
                let data = rt_device_blk_geometry {
                    block_size: block_size as u32,
                    bytes_per_sector: 0,
                    sector_count: 0,
                };
                unsafe { (cmd.get_arg() as *mut rt_device_blk_geometry).write(data) };
                Ok(())
            });
        device
    }
    #[test]
    fn block_single_aligned_write() -> Result<()> {
        const BLK_SIZE: usize = 10;
        let mut device = setup_block_device(BLK_SIZE);
        device
            .expect_write()
            .withf(|pos, buf, len| *pos == 0 && *len == 1 && buf.len() == BLK_SIZE)
            .times(1)
            .returning(|_, _, _| Ok(1));
        let mut block_device = BlockDevice::new(&mut device)?;
        block_device.write(&vec![1; BLK_SIZE])?;
        assert_eq!(block_device.block_pos, 1);
        assert_eq!(block_device.byte_pos, 0);
        device.checkpoint();
        Ok(())
    }
    #[test]
    fn block_multiple_aligned_write() -> Result<()> {
        const BLK_SIZE: usize = 10;
        let mut device = setup_block_device(BLK_SIZE);
        device
            .expect_write()
            .withf(|pos, buf, len| *pos == 0 && *len == 3 && buf.len() == BLK_SIZE * 3)
            .times(1)
            .returning(|_, _, _| Ok(3));
        let mut block_device = BlockDevice::new(&mut device)?;
        block_device.write(&vec![1; BLK_SIZE * 3])?;
        assert_eq!(block_device.block_pos, 3);
        assert_eq!(block_device.byte_pos, 0);
        device.checkpoint();
        Ok(())
    }
    #[test]
    fn block_unaligned_write() -> Result<()> {
        const BLK_SIZE: usize = 10;
        let buf: Vec<_> = (0..10).collect();
        let mut device = setup_block_device(BLK_SIZE);
        // Read first block
        device
            .expect_read()
            .withf(|pos, buf, len| *pos == 0 && *len == 1 && buf.len() == BLK_SIZE)
            .times(1)
            .returning(|_, _, _| Ok(1));
        // (3 + 10 * 3 + 3) = 36 => read block 3
        device
            .expect_read()
            .withf(|pos, buf, len| *pos == 3 && *len == 1 && buf.len() == BLK_SIZE)
            .times(1)
            .returning(|_, _, _| Ok(1));
        // Write first block
        device
            .expect_write()
            .withf(|pos, buf, len| *pos == 0 && *len == 1 && buf.len() == BLK_SIZE)
            .times(1)
            .returning(|_, _, _| Ok(1));
        // Write 1 - 2 blocks
        device
            .expect_write()
            .withf(|pos, buf, len| *pos == 1 && *len == 2 && buf.len() == BLK_SIZE * 2)
            .times(1)
            .returning(|_, _, _| Ok(2));
        let mut block_device = BlockDevice::new(&mut device)?;
        block_device.write(&buf[..3])?;
        assert_eq!(block_device.block_pos, 0);
        assert_eq!(block_device.byte_pos, 3);
        block_device.write(&vec![1; BLK_SIZE * 3 + 3])?;
        assert_eq!(block_device.block_pos, 3);
        assert_eq!(block_device.byte_pos, 6);
        device.checkpoint();
        Ok(())
    }
    #[test]
    fn block_partial_write() -> Result<()> {
        const BLK_SIZE: usize = 10;
        let buf: Vec<_> = (0..5).collect();
        let mut device = setup_block_device(BLK_SIZE);
        device
            .expect_read()
            .withf(|pos, buf, len| *pos == 0 && *len == 1 && buf.len() == BLK_SIZE)
            .times(1)
            .returning(|_, _, _| Ok(1));
        let mut block_device = BlockDevice::new(&mut device)?;
        block_device.write(&buf[..3])?;
        assert_eq!(block_device.block_pos, 0);
        assert_eq!(block_device.byte_pos, 3);
        block_device.write(&buf[..3])?;
        assert_eq!(block_device.block_pos, 0);
        assert_eq!(block_device.byte_pos, 6);
        device.checkpoint();
        Ok(())
    }
    #[test]
    fn block_flush() -> Result<()> {
        const BLK_SIZE: usize = 10;
        let buf: Vec<_> = (0..5).collect();
        let mut device = setup_block_device(BLK_SIZE);
        device
            .expect_read()
            .withf(|pos, buf, len| *pos == 0 && *len == 1 && buf.len() == BLK_SIZE)
            .times(1)
            .returning(|_, _, _| Ok(1));
        device
            .expect_write()
            .withf(|pos, buf, len| *pos == 0 && *len == 1 && buf.len() == BLK_SIZE)
            .times(1)
            .returning(|_, buf, _| {
                assert_eq!(buf, [0, 1, 2, 0, 1, 2, 0, 0, 0, 0]);
                Ok(1)
            });
        let mut block_device = BlockDevice::new(&mut device)?;
        block_device.write(&buf[..3])?;
        assert_eq!(block_device.block_pos, 0);
        assert_eq!(block_device.byte_pos, 3);
        block_device.write(&buf[..3])?;
        assert_eq!(block_device.block_pos, 0);
        assert_eq!(block_device.byte_pos, 6);
        block_device.flush()?;
        assert_eq!(block_device.block_pos, 0);
        assert_eq!(block_device.byte_pos, 6);
        device.checkpoint();
        Ok(())
    }
    #[test]
    fn block_single_aligned_read() -> Result<()> {
        const BLK_SIZE: usize = 10;
        let mut device = setup_block_device(BLK_SIZE);
        device
            .expect_read()
            .withf(|pos, buf, len| *pos == 0 && *len == 1 && buf.len() == BLK_SIZE)
            .times(1)
            .returning(|_, _, _| Ok(1));
        let mut block_device = BlockDevice::new(&mut device)?;
        block_device.read(&mut vec![1; BLK_SIZE])?;
        assert_eq!(block_device.block_pos, 1);
        assert_eq!(block_device.byte_pos, 0);
        device.checkpoint();
        Ok(())
    }
    #[test]
    fn block_multiple_aligned_read() -> Result<()> {
        const BLK_SIZE: usize = 10;
        let mut device = setup_block_device(BLK_SIZE);
        device
            .expect_read()
            .withf(|pos, buf, len| *pos == 0 && *len == 3 && buf.len() == BLK_SIZE * 3)
            .times(1)
            .returning(|_, _, _| Ok(3));
        let mut block_device = BlockDevice::new(&mut device)?;
        block_device.read(&mut vec![1; BLK_SIZE * 3])?;
        assert_eq!(block_device.block_pos, 3);
        assert_eq!(block_device.byte_pos, 0);
        device.checkpoint();
        Ok(())
    }
    #[test]
    fn block_unaligned_read() -> Result<()> {
        const BLK_SIZE: usize = 10;
        let mut buf: Vec<_> = (0..10).collect();
        let mut device = setup_block_device(BLK_SIZE);
        // Read first block
        device
            .expect_read()
            .withf(|pos, buf, len| *pos == 0 && *len == 1 && buf.len() == BLK_SIZE)
            .times(1)
            .returning(|_, _, _| Ok(1));
        // (3 + 10 * 3 + 3) = 36 => read block 3
        device
            .expect_read()
            .withf(|pos, buf, len| *pos == 3 && *len == 1 && buf.len() == BLK_SIZE)
            .times(1)
            .returning(|_, _, _| Ok(1));
        // Write 1 - 2 blocks
        device
            .expect_read()
            .withf(|pos, buf, len| *pos == 1 && *len == 2 && buf.len() == BLK_SIZE * 2)
            .times(1)
            .returning(|_, _, _| Ok(2));
        let mut block_device = BlockDevice::new(&mut device)?;
        block_device.read(&mut buf[..3])?;
        assert_eq!(block_device.block_pos, 0);
        assert_eq!(block_device.byte_pos, 3);
        block_device.read(&mut vec![1; BLK_SIZE * 3 + 3])?;
        assert_eq!(block_device.block_pos, 3);
        assert_eq!(block_device.byte_pos, 6);
        device.checkpoint();
        Ok(())
    }
    #[test]
    fn block_partial_read() -> Result<()> {
        const BLK_SIZE: usize = 10;
        let mut buf: Vec<_> = (0..5).collect();
        let mut device = setup_block_device(BLK_SIZE);
        device
            .expect_read()
            .withf(|pos, buf, len| *pos == 0 && *len == 1 && buf.len() == BLK_SIZE)
            .times(1)
            .returning(|_, _, _| Ok(1));
        let mut block_device = BlockDevice::new(&mut device)?;
        block_device.read(&mut buf[..3])?;
        assert_eq!(block_device.block_pos, 0);
        assert_eq!(block_device.byte_pos, 3);
        block_device.read(&mut buf[..3])?;
        assert_eq!(block_device.block_pos, 0);
        assert_eq!(block_device.byte_pos, 6);
        device.checkpoint();
        Ok(())
    }
}
