//! Hardware Semaphore (HSEM)

use crate::stm32;
use crate::rcc::{rec, ResetEnable};

const CPUID_CPU1:u8 = 0x03;
const CPUID_CPU2:u8 = 0x01;

#[cfg(feature="cm7")]
const CPUID:u8 = CPUID_CPU1;
#[cfg(feature="cm4")]
const CPUID:u8 = CPUID_CPU2;

const LOCK_BIT:u32 = 0x01 << 31;

macro_rules! hsem_define {
    [$($name:ident : $n:literal),+] => {
        pub struct Hsem {
            $($name: Sema<$n>,)+
        }
    };
}

macro_rules! hsem_init {
    [$($name:ident : $n:literal),+] => {
        Hsem {
            $($name : Sema::<$n>{},)+
        }
    }
}

macro_rules! hsem_fn_sema {
    [$($name:ident : $n:literal),+] => {
        $(
            pub fn $name(&self) -> Sema<$n> {
                self.$name
            }
        )+
    }
}

#[derive(Clone,Copy)]
pub struct Sema<const N:usize> {}

impl<const N:usize> Sema<N> {

    pub fn enable_irq(&mut self) {
        let rb_ptr = crate::pac::HSEM::ptr();
        match CPUID {
            CPUID_CPU1 => unsafe { // cm7
                //(*rb_ptr).c1ier().modify(|r,w| { w.bits(r.bits() | 0b1 << N) });
                (*rb_ptr).c1ier().modify(|_,w| { w.isem(N as u8).set_bit() });
            },
            CPUID_CPU2 => unsafe { // cm4
                (*rb_ptr).c2ier().modify(|_,w| { w.isem(N as u8).set_bit() });
            },
            _ => unreachable!()
        }
    }

    pub fn disable_irq(&mut self) {
        let rb_ptr = crate::pac::HSEM::ptr();
        match CPUID {
            CPUID_CPU1 => unsafe { // cm7
                (*rb_ptr).c1ier().modify(|_,w| { w.isem(N as u8).clear_bit() });
            },
            CPUID_CPU2 => unsafe { // cm4
                (*rb_ptr).c2ier().modify(|_,w| { w.isem(N as u8).clear_bit() });
            },
            _ => unreachable!()
        }
    }

    pub fn status_irq(&mut self) -> bool {
        let rb_ptr = crate::pac::HSEM::ptr();
        match CPUID {
            CPUID_CPU1 => unsafe { // cm7
                (*rb_ptr).c1isr().read().isem(N as u8).bit()
            },
            CPUID_CPU2 => unsafe { // cm4
                (*rb_ptr).c2isr().read().isem(N as u8).bit()
            },
            _ => unreachable!()
        }
    }

    pub fn clear_irq(&mut self){
        let rb_ptr = crate::pac::HSEM::ptr();
        match CPUID {
            CPUID_CPU1 => unsafe { // cm7
                (*rb_ptr).c1icr().write(|w| { w.isem(N as u8).set_bit() });
            },
            CPUID_CPU2 => unsafe { // cm4
                (*rb_ptr).c2icr().write(|w| { w.isem(N as u8).set_bit() });
            },
            _ => unreachable!()
        }
    }

    pub fn take(&mut self, proc_id: u8) -> bool {
        let rb_ptr = crate::pac::HSEM::ptr();
        let r:u32 = LOCK_BIT | ((CPUID as u32) << 8) | (proc_id as u32);
        unsafe {
            (*rb_ptr).r(N).write(|w| w.bits(r) );
            (*rb_ptr).r(N).read().bits() == r
        }
    }

    pub fn fast_take(&mut self) -> bool {
        let rb_ptr = crate::pac::HSEM::ptr();
        let rlr = unsafe {
            let rlr = (*rb_ptr).rlr(N).read().bits();
            ((rlr & LOCK_BIT) == LOCK_BIT, ((rlr >> 8) & 0x0000_000F), (rlr & 0x000000FF))
        };

        rlr.0 && (rlr.1 == CPUID.into()) && rlr.2 == 0
    }

    pub fn release(&mut self, proc_id: u8) {
        let rb_ptr = crate::pac::HSEM::ptr();
        let r:u32 = ((CPUID as u32) << 8) | (proc_id as u32);
        unsafe { (*rb_ptr).r(N).write(|w| { w.bits(r) }) };
    }
}

pub trait HsemExt {
    type Rec: ResetEnable;

    fn hsem(self, prec: Self::Rec) -> Hsem;
    fn hsem_without_reset(self, prec: Self::Rec) -> Hsem;
}

hsem_define! [
    sema0:0, sema1:1, sema2:2, sema3:3,
    sema4:4, sema5:5, sema6:6, sema7:7,
    sema8:8, sema9:9, sema10:10, sema11:11,
    sema12:12, sema13:13, sema14:14, sema15:15,
    sema16:16, sema17:17, sema18:18, sema19:19,
    sema20:20, sema21:21, sema22:22, sema23:23,
    sema24:24, sema25:25, sema26:26, sema27:27,
    sema28:28, sema29:29, sema30:30, sema31:31
];

impl HsemExt for stm32::HSEM {
    type Rec = rec::Hsem;
    fn hsem(self, prec: Self::Rec) -> Hsem {
        prec.enable().reset();
        hsem_init![
            sema0:0, sema1:1, sema2:2, sema3:3,
            sema4:4, sema5:5, sema6:6, sema7:7,
            sema8:8, sema9:9, sema10:10, sema11:11,
            sema12:12, sema13:13, sema14:14, sema15:15,
            sema16:16, sema17:17, sema18:18, sema19:19,
            sema20:20, sema21:21, sema22:22, sema23:23,
            sema24:24, sema25:25, sema26:26, sema27:27,
            sema28:28, sema29:29, sema30:30, sema31:31
        ]
    }
    fn hsem_without_reset(self, prec: Self::Rec) -> Hsem {
        prec.enable();
        hsem_init![
            sema0:0, sema1:1, sema2:2, sema3:3,
            sema4:4, sema5:5, sema6:6, sema7:7,
            sema8:8, sema9:9, sema10:10, sema11:11,
            sema12:12, sema13:13, sema14:14, sema15:15,
            sema16:16, sema17:17, sema18:18, sema19:19,
            sema20:20, sema21:21, sema22:22, sema23:23,
            sema24:24, sema25:25, sema26:26, sema27:27,
            sema28:28, sema29:29, sema30:30, sema31:31
        ]
    }
}

impl Hsem {
    hsem_fn_sema! [
        sema0:0, sema1:1, sema2:2, sema3:3,
        sema4:4, sema5:5, sema6:6, sema7:7,
        sema8:8, sema9:9, sema10:10, sema11:11,
        sema12:12, sema13:13, sema14:14, sema15:15,
        sema16:16, sema17:17, sema18:18, sema19:19,
        sema20:20, sema21:21, sema22:22, sema23:23,
        sema24:24, sema25:25, sema26:26, sema27:27,
        sema28:28, sema29:29, sema30:30, sema31:31
    ];
}
