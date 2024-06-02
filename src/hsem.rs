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

const ERROR_MSGS:[&str;3] = [
    "this samephore has already been taken.",
    "permission of semaphore operation has already been moved.",
    "permission of interrupt operation has already been moved."
];

macro_rules! hsem_define {
    [$($name:ident : $n:literal),+] => {
        pub struct Hsem {
            $($name: Option<Sema<$n>>,)+
        }
    };
}

macro_rules! hsem_init {
    [$($name:ident : $n:literal),+] => {
        Hsem {
            $($name : Some(Sema::<$n>{ op: Some(SemaOp::<$n>{}), intr: Some(SemaIntr::<$n>{}) }),)+
        }
    }
}

macro_rules! hsem_fn_sema {
    [$($name:ident : $n:literal),+] => {
        $(
            pub fn $name(&mut self) -> Sema<$n> {
                if let Some(sema) = self.$name.take() {
                    sema
                }
                else {
                    panic!("this samephore has already been taken.");
                }
            }
        )+
    }
}

pub struct SemaOp<const N:usize> {
}

pub struct SemaIntr<const N:usize> {
}

pub struct Sema<const N:usize> {
    op: Option<SemaOp<N>>,
    intr: Option<SemaIntr<N>>,
}

impl<const N:usize> SemaIntr<N> {

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
}

impl<const N:usize> SemaOp <N> {

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

impl<const N:usize> Sema<N> {

    pub fn split(&mut self) -> (SemaOp<N>, SemaIntr<N>) {
        let intr = if let Some(intr) = self.intr.take() {
            intr
        } else {
            panic!("{}",ERROR_MSGS[2]);
        };

        let op = if let Some(op) = self.op.take() {
            op
        } else {
            panic!("{}",ERROR_MSGS[1]);
        };
        (op, intr)
    }

    pub fn enable_irq(&mut self) {
        if let Some(ref mut intr) = self.intr {
            intr.enable_irq()
        }
        else {
            panic!("{}",ERROR_MSGS[2]);
        }
    }

    pub fn disable_irq(&mut self) {
        if let Some(ref mut intr) = self.intr {
            intr.disable_irq()
        }
        else {
            panic!("{}",ERROR_MSGS[2]);
        }
    }

    pub fn status_irq(&mut self) -> bool {
        if let Some(ref mut intr) = self.intr {
            intr.status_irq()
        }
        else {
            panic!("{}",ERROR_MSGS[2]);
        }
    }

    pub fn clear_irq(&mut self){
        if let Some(ref mut intr) = self.intr {
            intr.clear_irq()
        }
        else {
            panic!("{}",ERROR_MSGS[2]);
        }
    }

    pub fn take(&mut self, proc_id: u8) -> bool {
        if let Some(ref mut op) = self.op {
            op.take(proc_id)
        }
        else {
            panic!("{}",ERROR_MSGS[1]);
        }
    }

    pub fn fast_take(&mut self) -> bool {
        if let Some(ref mut op) = self.op {
            op.fast_take()
        }
        else {
            panic!("{}",ERROR_MSGS[1]);
        }
    }

    pub fn release(&mut self, proc_id: u8) {
        if let Some(ref mut op) = self.op {
            op.release(proc_id)
        }
        else {
            panic!("{}",ERROR_MSGS[1]);
        }
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
