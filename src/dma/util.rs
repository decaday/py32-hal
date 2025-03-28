// The following code is modified from embassy-stm32
// https://github.com/embassy-rs/embassy/tree/main/embassy-stm32
// Special thanks to the Embassy Project and its contributors for their work!

use embassy_hal_internal::PeripheralRef;

use super::word::Word;
use super::{AnyChannel, Request, Transfer, TransferOptions};

/// Convenience wrapper, contains a channel and a request number.
///
/// Commonly used in peripheral drivers that own DMA channels.
pub(crate) struct ChannelAndRequest<'d> {
    pub channel: PeripheralRef<'d, AnyChannel>,
    pub request: Request,
}

impl<'d> ChannelAndRequest<'d> {
    #[allow(dead_code)]
    pub unsafe fn read<'a, W: Word>(
        &'a mut self,
        peri_addr: *mut W,
        buf: &'a mut [W],
        options: TransferOptions,
    ) -> Transfer<'a> {
        Transfer::new_read(&mut self.channel, self.request, peri_addr, buf, options)
    }

    #[allow(dead_code)]
    pub unsafe fn read_raw<'a, W: Word>(
        &'a mut self,
        peri_addr: *mut W,
        buf: *mut [W],
        options: TransferOptions,
    ) -> Transfer<'a> {
        Transfer::new_read_raw(&mut self.channel, self.request, peri_addr, buf, options)
    }

    pub unsafe fn write<'a, W: Word>(
        &'a mut self,
        buf: &'a [W],
        peri_addr: *mut W,
        options: TransferOptions,
    ) -> Transfer<'a> {
        Transfer::new_write(&mut self.channel, self.request, buf, peri_addr, options)
    }

    #[allow(dead_code)]
    pub unsafe fn write_raw<'a, W: Word>(
        &'a mut self,
        buf: *const [W],
        peri_addr: *mut W,
        options: TransferOptions,
    ) -> Transfer<'a> {
        Transfer::new_write_raw(&mut self.channel, self.request, buf, peri_addr, options)
    }

    #[allow(dead_code)]
    pub unsafe fn write_repeated<'a, W: Word>(
        &'a mut self,
        repeated: &'a W,
        count: usize,
        peri_addr: *mut W,
        options: TransferOptions,
    ) -> Transfer<'a> {
        Transfer::new_write_repeated(
            &mut self.channel,
            self.request,
            repeated,
            count,
            peri_addr,
            options,
        )
    }
}
