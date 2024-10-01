use crate::pac::{FLASH, RCC};

const HSI: u32 = 8_000_000;

pub trait RccExt {
    fn rcc_config() -> RccConfig;
    fn rtc_config() -> RtcConfig;
    fn mco_config() -> McoConfig;
}

impl RccExt for crate::pac::RCC {
    fn rcc_config() -> RccConfig {
        RccConfig {
            hse: 0,
            pll: 0,
            apb1_pre: 2,
            apb2_pre: 1,
            adc_pre: 6,
            hse_byp: false
        }
    }
    
    fn rtc_config() -> RtcConfig {
        RtcConfig
    }
    
    fn mco_config() -> McoConfig {
        McoConfig
    }
    
}
pub struct McoConfig;
pub struct RtcConfig;
pub struct RccConfig {
    hse: u32,
    pll: u8,
    apb1_pre: u8,
    apb2_pre: u8,
    adc_pre: u8,
    hse_byp: bool,
}
impl RccConfig {
    pub fn hse_byp_use(mut self, clock: u32) -> Self {
        self.hse_byp = true;
        self.hse = clock;
        self
    }

    pub fn hse_use(mut self, hse_clock: u32) -> Self {
        self.hse = hse_clock;
        self
    }
    
    pub fn pll(mut self, pll_mul: u8) -> Self {
        self.pll = pll_mul;
        self
    }
    
    pub fn apb1(mut self, div: u8) -> Self {
        self.apb1_pre = div;
        self
    }
    
    pub fn apb2(mut self, div: u8) -> Self {
        self.apb2_pre = div;
        self
    }

    pub fn adc(mut self, div: u8) -> Self {
        self.adc_pre = div;
        self
    }
    
    #[inline(always)]
    pub fn tune(self) -> Clock {
        let rcc = unsafe { &*RCC::ptr() };
        let flash = unsafe { &*FLASH::ptr()};
        let mut trash_hold: u16 = 0;

        let sh = if self.hse != 0 {
            if self.hse_byp == true {
                rcc.cr.modify(|_, w|w.hsebyp().set_bit());
            }
            rcc.cr.modify(|_, w|w.hseon().set_bit());
            
            while rcc.cr.read().hserdy().bit_is_clear() {

                trash_hold += 1;
                if trash_hold == 1000 {
                    rcc.cr.modify(|_, w|w.hseon().clear_bit());
                    panic!();
                }
            }
            self.hse
        } 
        else {
            if self.pll == 0{
                HSI
            }
            else {
                HSI / 2
            }
        };

        let sysclk = 
            if self.pll != 0 {
                let sysclk = self.pll as u32 * sh;
                
                assert!(sysclk <= 72_000_000);
                rcc.cfgr.modify(|_, w|
                    w.pllmul().bits((self.pll - 2) as u8)
                    .pllsrc().bit(if self.hse != 0 {
                        true
                    }
                    else {
                        false
                    })
                );
                rcc.cr.modify(|_, w| w.pllon().set_bit());
                
                while rcc.cr.read().pllrdy().bit_is_clear() {}
                sysclk
            }
            else {
                sh
            };

        let (apb1_pre_bits, pclk1) = match self.apb1_pre {
            2 => (0b100, sysclk / 2),
            4 => (0b101, sysclk / 4),
            8 => (0b110, sysclk / 8),
            16 => (0b111, sysclk / 16),
            _ => (0b0, sysclk),
        };
        assert!(pclk1 <= 36_000_000);
        
        let (apb2_pre_bits, pclk2) = match self.apb2_pre {
            2 => (0b100, sysclk / 2),
            4 => (0b101, sysclk / 4),
            8 => (0b110, sysclk / 8),
            16 => (0b111, sysclk / 16),
            _ => (0b0, sysclk),
        };

        let adc_pre_bits = match self.adc_pre {
            2 => 0b00,
            4 => 0b01,
            6 => 0b10,
            8 => 0b11,
            _ => 0b10,
        };
        let adc_clk = pclk2 / adc_pre_bits;
        assert!(adc_clk <= 14_000_000);

        unsafe {
            flash.acr.modify(|_, w|
                w.latency().bits( if sysclk <= 24_000_000 {
                        0b000
                    } 
                    else if sysclk <= 48_000_000 {
                        0b001
                    }
                    else {
                        0b010
                    }));

            rcc.cfgr.modify(|_, w|
                w.adcpre().bits(adc_pre_bits as u8)
                .ppre1().bits(apb1_pre_bits)
                .ppre2().bits(apb2_pre_bits)
                .sw().bits( if self.pll != 0 {
                    0b10 // PLL
                } 
                else if self.hse != 0 {
                    0b1 // HSE
                }
                else {
                    0b0 // HSI
                })
            );
        }

        Clock { 
            sysclk:(sysclk),
            pclk1: (pclk1), 
            pclk2: (pclk2), 
            adcclk: (adc_clk)
        }
    }
}

pub struct Clock {
    pub sysclk: u32,
    pub pclk1: u32,
    pub pclk2: u32,
    pub adcclk: u32,
}
