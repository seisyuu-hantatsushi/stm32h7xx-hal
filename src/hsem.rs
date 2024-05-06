//! Hardware Semaphore (HSEM)

use crate::stm32;
use crate::rcc::{rec, ResetEnable};

const CPUID_CPU1:u8 = 0x03;
const CPUID_CPU2:u8 = 0x01;

#[cfg(feature="cm7")]
const CPUID:u8 = CPUID_CPU1;
#[cfg(feature="cm4")]
const CPUID:u8 = CPUID_CPU2;

pub trait HsemExt {
    type Rec: ResetEnable;

    fn hsem(self, prec: Self::Rec) -> Hsem;
    fn hsem_without_reset(self, prec: Self::Rec) -> Hsem;
}

pub struct Hsem {
}

impl HsemExt for stm32::HSEM {
    type Rec = rec::Hsem;
    fn hsem(self, prec: Self::Rec) -> Hsem {
        prec.enable().reset();
        Hsem {
        }
    }
    fn hsem_without_reset(self, prec: Self::Rec) -> Hsem {
        prec.enable();
        Hsem {
        }
    }
}

impl Hsem {
    pub fn sema(&self, no:usize) -> Sema {
        Sema  {
            no,
        }
    }
}

pub struct Sema {
    no: usize,
}

impl Sema {

    pub fn enable_irq(&mut self) {
        let rb_ptr = crate::pac::HSEM::ptr();
        match CPUID {
            CPUID_CPU1 => unsafe { // cm7
                (*rb_ptr).c1ier().modify(|r,w| { w.bits(r.bits() | 0b1 << self.no) });
            },
            CPUID_CPU2 => unsafe { // cm4
                (*rb_ptr).c2ier().modify(|r,w| { w.bits(r.bits() | 0b1 << self.no) });
            },
            _ => unreachable!()
        }
    }

    pub fn disable_irq(&mut self) {
        let rb_ptr = crate::pac::HSEM::ptr();
        let ch_bit : u32 = 0b1 << self.no;
        match CPUID {
            CPUID_CPU1 => unsafe { // cm7
                (*rb_ptr).c1ier().modify(|r,w| { w.bits(r.bits() & !ch_bit) });
            },
            CPUID_CPU2 => unsafe { // cm4
                (*rb_ptr).c2ier().modify(|r,w| { w.bits(r.bits() & !ch_bit) });
            },
            _ => unreachable!()
        }
    }

    pub fn clear_irq(&mut self){
        let rb_ptr = crate::pac::HSEM::ptr();
        let ch_bit : u32 = 0b1 << self.no;
        match CPUID {
            CPUID_CPU1 => unsafe { // cm7
                (*rb_ptr).c1icr().write(|w| { w.bits(ch_bit) });
            },
            CPUID_CPU2 => unsafe { // cm4
                (*rb_ptr).c2icr().write(|w| { w.bits(ch_bit) });
            },
            _ => unreachable!()
        }
    }

    pub fn fast_take(&mut self) -> bool {
        let rb_ptr = crate::pac::HSEM::ptr();
        let rlr = unsafe {
            let rlr = (*rb_ptr).rlr(self.no).read().bits();
            ((rlr & (0x1 << 31)) == (0x1 << 31), ((rlr >> 8) & 0x0000_000F), (rlr & 0x000000FF))
        };
        #[cfg(feature = "log")]
        log::debug!("fast_take {:?}", rlr);
        rlr.0 && (rlr.1 == CPUID.into()) && rlr.2 == 0
    }

    pub fn release(&mut self, proc_id: u8) {
        let rb_ptr = crate::pac::HSEM::ptr();
        let r:u32 = ((CPUID as u32) << 8) | (proc_id as u32);
        #[cfg(feature = "log")]
        log::debug!("release 0x{:08x}", r);
        unsafe { (*rb_ptr).r(self.no).modify(|_,w| { w.bits(r) }) };
    }
}
