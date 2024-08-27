use core::ptr::addr_of;
use cortex_m_rt::exception;

#[exception]
fn SysTick() {
    unsafe { G_TIME = G_TIME.wrapping_add(1) };
}

static mut G_TIME: u32 = 0;
static mut G_AHB_CLK: u32 = 8;

pub struct SysTime{}
impl SysTime {
    pub fn new(ahb_clk: u32){
        let stk = unsafe { &*crate::pac::STK::ptr() };
        unsafe { G_AHB_CLK = ahb_clk / 1_000_000 };
        stk.load_.write(|w| unsafe { w.bits(ahb_clk / 1000 - 1) });
        stk.val.write(|w|unsafe{ w.bits(0) });
        stk.ctrl.modify(|_, w|w
            .tickint().set_bit()
            .clksource().set_bit()
            .enable().set_bit());
        SysTime::dwt_enable(); 
    }
    
    pub fn now() -> u32 {
        unsafe { core::ptr::read_volatile(addr_of!(G_TIME)) }   
    }

    pub fn delay<T: QuantTime>(delay: T) {
        delay.delay();
    }

    /// Returns microseconds since power on systime
    ///
    /// # Safety
    ///
    /// - Do not call this function inside an `interrupt::free` critical section
    pub unsafe fn now_us() -> u32 {
        let stk = unsafe{ &*crate::pac::STK::ptr() };
        let  cur_val: u32;
        let  ms: u32;

        cortex_m::interrupt::disable();
            cur_val = stk.val.read().bits();
            if  stk.ctrl.read().countflag().bit() {
                unsafe { G_TIME += 1 };
            }
            ms = SysTime::now() as u32;
        unsafe { cortex_m::interrupt::enable(); }
        unsafe { ms * 1000 + 999 - cur_val / G_AHB_CLK }
    }
    

    #[inline(always)]
    pub fn dwt_now() -> u32 {
        unsafe { (*crate::pac::DWT::PTR).cyccnt.read() }
    }
    
    #[inline(always)]
    pub fn dwt_start() {
        unsafe { (*crate::pac::DWT::PTR).cyccnt.write(0)};
    }
    
    fn dwt_enable() {
        unsafe {
            (*crate::pac::DCB::PTR).demcr.modify(|r| r | (1 << 24));
            (*crate::pac::DWT::PTR).ctrl.modify(|r| r | (1 << 0));
        }
    }
    
}


pub trait U32time {
    fn hz(self) -> u32;
    fn khz(self) -> u32;
    fn mhz(self) -> u32;
    fn ms(self) -> MilliSeconds;
    fn us(self) -> MicroSeconds;
}

impl U32time for u32 {
    fn hz(self) -> u32 {
        self
    }

    fn khz(self) -> u32 {
        self * 1000
    }

    fn mhz(self) -> u32 {
        self.khz() * 1000
    }

    fn ms(self) -> MilliSeconds {
        MilliSeconds(self)
    }

    fn us(self) -> MicroSeconds {
        MicroSeconds(self)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MilliSeconds(pub u32);

#[derive(Debug, Clone, Copy)]
pub struct MicroSeconds(pub u32);


pub trait QuantTime{
    fn delay(self);
}

impl QuantTime for MicroSeconds {
    fn delay(self) {
        let us = unsafe {
            if G_AHB_CLK <= 24 {
                G_AHB_CLK * self.0
            }
            else if G_AHB_CLK <= 48 {
                (G_AHB_CLK / 2) * self.0
            }
            else {
                (G_AHB_CLK / 3) * self.0
            }
        };
        cortex_m::asm::delay(us);
    }
}

impl QuantTime for MilliSeconds {
    fn delay(self) {
        let delay = SysTime::now();
        while SysTime::now() - delay < self.0 {}
    }
}
