//! Hardware Semaphore (HSEM)

use core::marker::PhantomData;

use crate::stm32;
use crate::rcc::{rec, ResetEnable};

const CPUID_CPU1:u8 = 0x01;
const CPUID_CPU2:u8 = 0x03;

#[cfg(feature="cm7")]
const CPUID:u8 = CPUID_CPU1;
#[cfg(feature="cm4")]
const CPUID:u8 = CPUID_CPU2;

pub trait HsemExt {
    type Rec: ResetEnable;

    fn hsem(self, prec: Self::Rec) -> Hsem;
    fn hsem_without_reset(self) -> Hsem;
}

pub struct Hsem {
    pub(crate) rb: stm32::HSEM,
}

impl HsemExt for stm32::HSEM {
    type Rec = rec::Hsem;
    fn hsem(self, prec: Self::Rec) -> Hsem {
        prec.enable().reset();
        Hsem {
            rb: self
        }
    }
    fn hsem_without_reset(self) -> Hsem {
        Hsem {
            rb: self
        }
    }
}

impl Hsem {
    pub fn sema<'a>(&'a self, no:usize) -> Sema<'a> {
        Sema  {
            no,
            rb: &self.rb
        }
    }
}

pub struct Sema<'a> {
    no: usize,
    rb: &'a stm32::HSEM
}

impl<'a> Sema<'a> {
    pub fn enable_interrupt(&mut self) {
        match CPUID {
            CPUID_CPU1 => { // cm7
                self.rb.c1ier().modify(|r,w| unsafe { w.bits(r.bits() | 0b1 << self.no) });
            },
            CPUID_CPU2 => { // cm4
                self.rb.c2ier().modify(|r,w| unsafe { w.bits(r.bits() | 0b1 << self.no) });
            },
            _ => unreachable!()
        }
        self.rb.r(self.no).modify(|r,w| unsafe { w.bits(
            r.bits() | (0b1 << self.no)
        )})
    }
}
