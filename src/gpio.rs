use crate::pac;
pub enum GpioSpeed {
	Mhz2 = 0b10,
	Mhz10 = 0b01,
	Mhz50 = 0b11,
}
pub enum Edge {
	RISING,
	FALLING,
	RISINGFALLING,
}


pub struct Input;
pub struct Output;
pub struct Alternate;
pub struct Analog;
pub struct Reset;


pub trait InputPin {
	type Res;
	fn is_low(&self) -> bool;
	fn is_high(&self) -> bool;
	fn change_pull_up(&self);
	fn change_pull_down(&self);
	fn reset(self) -> Self::Res;
}
pub trait OutputPin {
	type Res;
	fn set_high(&self);
	fn set_low(&self);
	fn toggle(&self);
	fn is_set_low(&self) -> bool;
	fn is_set_high(&self) -> bool;
	fn reset(self) -> Self::Res;
}
pub trait ExtiPin: InputPin {
	fn interrupt_init(&self, edge: Edge);
	fn interrupt_enable(&self);
	fn interrupt_disable(&self);
	fn interrupt_check(&self) -> bool;
	fn interrupt_clear_pending_bit(&self);
	fn interrupt_generate(&self);
}

macro_rules! gpio_as_var {
	($gpiox:ident, $GPIOx:ident, $Portx:ident, $iopxen:ident, $extiport:expr, $($pxi:ident: ($PXi:ident, $pin:expr, $exticrx:ident, $crx:ident),)+) => {
		pub mod $gpiox {
			use super::pac::{$GPIOx, RCC, EXTI, AFIO};
			use super::{
				GpioSpeed,
				InputPin,
				OutputPin,
				ExtiPin,
				Output,
				Input,
				Alternate,
				Analog,
				Reset,
				Edge,
			};

			$(
				pub struct $PXi<T> {
					_mode: T
				}
			)+
							
			pub struct $Portx {
				$(
					pub $pxi: $PXi<Reset>,
				)+
			}

			impl $Portx {
				pub fn enable() -> $Portx {
					unsafe { (*RCC::ptr()).apb2enr.modify(|_, w|w.$iopxen().set_bit()); }
					$Portx { 
						$(
							$pxi: $PXi { _mode: Reset },
						)+
					}
				}	
			}
		
			$(
				impl $PXi<Reset> {
					pub fn push_pull(self, speed: GpioSpeed) -> $PXi<Output>{
						const OFFSET: u32 = (4 * $pin) % 32;
						let bits: u32 = (0b00 << 2) | speed as u32;
						
						unsafe{ (*$GPIOx::ptr()).$crx.modify(|r, w| w.bits((r.bits() & !(0b1111 << OFFSET)) | (bits << OFFSET))); }

						$PXi { _mode: Output }
					}
			
					pub fn open_drain(self, speed: GpioSpeed) -> $PXi<Output>{
						const OFFSET: u32 = (4 * $pin) % 32;
						let bits: u32 = (0b01 << 2) | speed as u32;
						
						unsafe{ (*$GPIOx::ptr()).$crx.modify(|r, w| w.bits((r.bits() & !(0b1111 << OFFSET)) | (bits << OFFSET))); }

						$PXi { _mode: Output }
					}
			
					pub fn alternate_push_pull(self, speed: GpioSpeed) -> $PXi<Alternate>{
						const OFFSET: u32 = (4 * $pin) % 32;
						let bits: u32 = (0b10 << 2) | speed as u32;
						
						unsafe{ (*$GPIOx::ptr()).$crx.modify(|r, w| w.bits((r.bits() & !(0b1111 << OFFSET)) | (bits << OFFSET))); }

						$PXi { _mode: Alternate }
					}
			
					pub fn alternate_open_drain(self, speed: GpioSpeed) -> $PXi<Alternate>{
						const OFFSET: u32 = (4 * $pin) % 32;
						let bits: u32 = (0b11 << 2) | speed as u32;
						
						unsafe{ (*$GPIOx::ptr()).$crx.modify(|r, w| w.bits((r.bits() & !(0b1111 << OFFSET)) | (bits << OFFSET))); }

						$PXi { _mode: Alternate }
					}
			
					pub fn pull_up(self) -> $PXi<Input>{
						const OFFSET: u32 = (4 * $pin) % 32;
                        const BITS: u32 = (0b10 << 2);

						unsafe{ 
							(*$GPIOx::ptr()).bsrr.write(|w| w.bits(1 << $pin));
							(*$GPIOx::ptr()).$crx.modify(|r, w| w.bits((r.bits() & !(0b1111 << OFFSET)) | (BITS << OFFSET))); 
						}

						$PXi { _mode: Input }
					}
			
					pub fn pull_down(self) -> $PXi<Input>{
						const OFFSET: u32 = (4 * $pin) % 32;
                        const BITS: u32 = (0b10 << 2);

						unsafe{ 
							(*$GPIOx::ptr()).bsrr.write(|w| w.bits(1 << (16 + $pin)));
							(*$GPIOx::ptr()).$crx.modify(|r, w| w.bits((r.bits() & !(0b1111 << OFFSET)) | (BITS << OFFSET))); 
						}

						$PXi { _mode: Input }
					}
			
					pub fn floating(self) -> $PXi<Input>{
						const OFFSET: u32 = (4 * $pin) % 32;
                        const BITS: u32 = (0b01 << 2);

                        unsafe{ (*$GPIOx::ptr()).$crx.modify(|r, w| w.bits((r.bits() & !(0b1111 << OFFSET)) | (BITS << OFFSET))); }

						$PXi { _mode: Input }
					}
			
					pub fn analog(self) -> $PXi<Analog>{
						const OFFSET: u32 = (4 * $pin) % 32;                        
                        const BITS: u32 = 0b0000;

                        unsafe{ (*$GPIOx::ptr()).$crx.modify(|r, w| w.bits((r.bits() & !(0b1111 << OFFSET)) | (BITS << OFFSET))); }

						$PXi { _mode: Analog }
					}
				}
				
				impl InputPin for $PXi<Input> {
					type Res = $PXi<Reset>;
			
					fn is_low(&self) -> bool {
						unsafe { (*$GPIOx::ptr()).idr.read().bits() & (1 << $pin) == 0}
					}
			
					fn is_high(&self) -> bool {
						!self.is_low()
					}
			
					fn change_pull_up(&self) {
						unsafe { (*$GPIOx::ptr()).bsrr.write(|w|w.bits(1 << $pin)) }
					}
			
					fn change_pull_down(&self) {
						unsafe { (*$GPIOx::ptr()).bsrr.write(|w|w.bits(1 << (16 + $pin))) }
					}
			
					fn reset(self) -> $PXi<Reset> {
						$PXi { _mode: Reset }
					}
				}

				impl OutputPin for $PXi<Output> {
					type Res = $PXi<Reset>;
					
					fn set_high(&self) {
						unsafe { (*$GPIOx::ptr()).bsrr.write(|w|w.bits(1 << $pin)) }
					}
			
					fn set_low(&self) {
						unsafe { (*$GPIOx::ptr()).bsrr.write(|w|w.bits(1 << (16 + $pin))) }
					}
			
					fn is_set_high(&self) -> bool {
						!self.is_set_low()
					}
			
					fn is_set_low(&self) -> bool {
						unsafe {  (*$GPIOx::ptr()).odr.read().bits() & (1 << $pin) == 0 }
					}
			
					fn toggle(&self) {
						if self.is_set_low(){
							self.set_high();
						}
						else {
							self.set_low();
						}
					}
				
					fn reset(self) -> $PXi<Reset> {
						$PXi { _mode: Reset }
					}

				}
			
				impl ExtiPin for $PXi<Input> {
					fn interrupt_check(&self) -> bool {
						unsafe { ((*EXTI::ptr()).pr.read().bits() & (1 << $pin)) != 0 }
					}
			
					fn interrupt_clear_pending_bit(&self) {
						unsafe { (*EXTI::ptr()).pr.write(|w| w.bits(1 << $pin)) };
					}
			
					fn interrupt_disable(&self) {
						unsafe { (*EXTI::ptr()).imr.modify(|r, w| w.bits(r.bits() & !(1 << $pin))) };
					}
			
					fn interrupt_enable(&self) {
						unsafe { (*EXTI::ptr()).imr.modify(|r, w| w.bits(r.bits() | (1 << $pin))) };
					}
			
					fn interrupt_generate(&self) {
						unsafe { (*EXTI::ptr()).swier.modify(|r, w| w.bits(r.bits() | (1 << $pin))) };
					}
			
					fn interrupt_init(&self, edge: Edge) {
						const OFFSET: u32 = 4 * ($pin % 4);

						unsafe { (*AFIO::ptr()).$exticrx.modify(|r, w| w.bits((r.bits() & (0xf << OFFSET)) | ($extiport << OFFSET))) };
						
						match edge {
							Edge::RISING => {
								unsafe {
									(*EXTI::ptr()).rtsr.modify(|r, w| w.bits(r.bits() | (1 << $pin)));
									(*EXTI::ptr()).ftsr.modify(|r, w| w.bits(r.bits() & !(1 << $pin)));
								}
							},
							Edge::FALLING => {
								unsafe {
									(*EXTI::ptr()).rtsr.modify(|r, w| w.bits(r.bits() & !(1 << $pin)));
									(*EXTI::ptr()).ftsr.modify(|r, w| w.bits(r.bits() | (1 << $pin)));
								}
							},
							Edge::RISINGFALLING => {
								unsafe {
									(*EXTI::ptr()).rtsr.modify(|r, w| w.bits(r.bits() | (1 << $pin)));
									(*EXTI::ptr()).ftsr.modify(|r, w| w.bits(r.bits() | (1 << $pin)));
								}
							}
						}
					}
				}
			)+
		}
	}
}

macro_rules! gpio_as_fn {
	($PORTx:ident, $GPIOx:ident, $iopxen:ident, $extiport:expr, $(($PXi:ident, $pin:expr, $exticrx:ident, $crx:ident),)+) => {
		#[allow(non_snake_case)]
		pub(crate) mod $PORTx{
			use super::pac::{$GPIOx, EXTI, AFIO, RCC};
			use super::{
				GpioSpeed,
				Edge
			};
			
			pub fn enable() {
				unsafe { (*RCC::ptr()).apb2enr.modify(|_, w|w.$iopxen().set_bit()); }
			}

			$(
				#[derive(Clone, Copy)]
				pub struct $PXi;
				impl $PXi {
					
					pub fn push_pull(speed: GpioSpeed){
						const OFFSET: u32 = (4 * $pin) % 32;
						let bits: u32 = (0b00 << 2) | speed as u32;
						
						unsafe{ (*$GPIOx::ptr()).$crx.modify(|r, w| w.bits((r.bits() & !(0b1111 << OFFSET)) | (bits << OFFSET))); }
					}
			
					pub fn open_drain(speed: GpioSpeed){
						const OFFSET: u32 = (4 * $pin) % 32;
						let bits: u32 = (0b01 << 2) | speed as u32;
						
						unsafe{ (*$GPIOx::ptr()).$crx.modify(|r, w| w.bits((r.bits() & !(0b1111 << OFFSET)) | (bits << OFFSET))); }
					}
			
					pub fn alternate_push_pull(speed: GpioSpeed){
						const OFFSET: u32 = (4 * $pin) % 32;
						let bits: u32 = (0b10 << 2) | speed as u32;
						
						unsafe{ (*$GPIOx::ptr()).$crx.modify(|r, w| w.bits((r.bits() & !(0b1111 << OFFSET)) | (bits << OFFSET))); }
					}
			
					pub fn alternate_open_drain(speed: GpioSpeed){
						const OFFSET: u32 = (4 * $pin) % 32;
						let bits: u32 = (0b11 << 2) | speed as u32;
						
						unsafe{ (*$GPIOx::ptr()).$crx.modify(|r, w| w.bits((r.bits() & !(0b1111 << OFFSET)) | (bits << OFFSET))); }
					}
			
					pub fn pull_up(){
						const OFFSET: u32 = (4 * $pin) % 32;
                        const BITS: u32 = (0b10 << 2);

						unsafe{ 
							(*$GPIOx::ptr()).bsrr.write(|w| w.bits(1 << $pin));
							(*$GPIOx::ptr()).$crx.modify(|r, w| w.bits((r.bits() & !(0b1111 << OFFSET)) | (BITS << OFFSET))); 
						}
					}
			
					pub fn pull_down(){
						const OFFSET: u32 = (4 * $pin) % 32;
                        const BITS: u32 = (0b10 << 2);

						unsafe{ 
							(*$GPIOx::ptr()).bsrr.write(|w| w.bits(1 << (16 + $pin)));
							(*$GPIOx::ptr()).$crx.modify(|r, w| w.bits((r.bits() & !(0b1111 << OFFSET)) | (BITS << OFFSET))); 
						}
					}
			
					pub fn floating(){
						const OFFSET: u32 = (4 * $pin) % 32;
                        const BITS: u32 = (0b01 << 2);

                        unsafe{ (*$GPIOx::ptr()).$crx.modify(|r, w| w.bits((r.bits() & !(0b1111 << OFFSET)) | (BITS << OFFSET))); }
					}
			
					pub fn analog(){
						const OFFSET: u32 = (4 * $pin) % 32;                        
                        const BITS: u32 = 0b0000;

                        unsafe{ (*$GPIOx::ptr()).$crx.modify(|r, w| w.bits((r.bits() & !(0b1111 << OFFSET)) | (BITS << OFFSET))); }
					}	
				
					pub fn is_low() -> bool {
						unsafe { (*$GPIOx::ptr()).idr.read().bits() & (1 << $pin) == 0 }
					}
			
					pub fn is_high() -> bool {
						!Self::is_low()
					}
			
					pub fn change_pull_up() {
						unsafe { (*$GPIOx::ptr()).bsrr.write(|w|w.bits(1 << $pin)) }
					}
			
					pub fn change_pull_down() {
						unsafe { (*$GPIOx::ptr()).bsrr.write(|w|w.bits(1 << (16 + $pin))) }
					}
				
					pub fn set_high() {
						unsafe { (*$GPIOx::ptr()).bsrr.write(|w|w.bits(1 << $pin)) }
					}
				
					pub fn set_low() {
						unsafe { (*$GPIOx::ptr()).bsrr.write(|w|w.bits(1 << (16 + $pin))) }
					}
				
					pub fn is_set_high() -> bool {
						!Self::is_set_low()
					}
				
					pub fn is_set_low() -> bool {
						unsafe {  (*$GPIOx::ptr()).odr.read().bits() & (1 << $pin) == 0 }
					}
				
					pub fn toggle() {
						if Self::is_set_low() {
							Self::set_high();
						}
						else {
							Self::set_low();
						}
					}
			
					pub fn interrupt_check() -> bool {
						unsafe { ((*EXTI::ptr()).pr.read().bits() & (1 << $pin)) != 0 }
					}
			
					pub fn interrupt_clear_pending_bit() {
						unsafe { (*EXTI::ptr()).pr.write(|w| w.bits(1 << $pin)) };
					}
			
					pub fn interrupt_disable() {
						unsafe { (*EXTI::ptr()).imr.modify(|r, w|  w.bits(r.bits() & !(1 << $pin))) };
					}
			
					pub fn interrupt_enable() {
						unsafe { (*EXTI::ptr()).imr.modify(|r, w|  w.bits(r.bits() | (1 << $pin))) };
					}
			
					pub fn interrupt_generate() {
						unsafe { (*EXTI::ptr()).swier.modify(|r, w|  w.bits(r.bits() | (1 << $pin))) };
					}
			
					pub fn interrupt_init(edge: Edge) {
						const OFFSET: u32 = 4 * ($pin % 4);

						unsafe { (*AFIO::ptr()).$exticrx.modify(|r, w|w.bits((r.bits() & (0xf << OFFSET)) | ($extiport << OFFSET))) };
						
						match edge {
							Edge::RISING => {
								unsafe {
									(*EXTI::ptr()).rtsr.modify(|r, w| w.bits(r.bits() | (1 << $pin)));
									(*EXTI::ptr()).ftsr.modify(|r, w| w.bits(r.bits() & !(1 << $pin)));
								}
							},
							Edge::FALLING => {
								unsafe {
									(*EXTI::ptr()).rtsr.modify(|r, w| w.bits(r.bits() & !(1 << $pin)));
									(*EXTI::ptr()).ftsr.modify(|r, w| w.bits(r.bits() | (1 << $pin)));
								}
							},
							Edge::RISINGFALLING => {
								unsafe {
									(*EXTI::ptr()).rtsr.modify(|r, w| w.bits(r.bits() | (1 << $pin)));
									(*EXTI::ptr()).ftsr.modify(|r, w| w.bits(r.bits() | (1 << $pin)));
								}
							}
						}
					}
				}
			)+
		}
	};
}


gpio_as_fn!{
	PORTA, GPIOA, iopaen, 0,
	(A0, 0, exticr1, crl),
	(A1, 1, exticr1, crl),
	(A2, 2, exticr1, crl),
	(A3, 3, exticr1, crl),
	(A4, 4, exticr2, crl),
	(A5, 5, exticr2, crl),
	(A6, 6, exticr2, crl),
	(A7, 7, exticr2, crl),
	(A8, 8, exticr3, crh),
	(A9, 9, exticr3, crh),
	(A10, 10, exticr3, crh),
	(A11, 11, exticr3, crh),
	(A12, 12, exticr4, crh),
	(A13, 13, exticr4, crh),
	(A14, 14, exticr4, crh),
	(A15, 15, exticr4, crh),
}

gpio_as_fn!{
	PORTB, GPIOB, iopben, 1,
	(B0, 0, exticr1, crl),
	(B1, 1, exticr1, crl),
	(B2, 2, exticr1, crl),
	(B3, 3, exticr1, crl),
	(B4, 4, exticr2, crl),
	(B5, 5, exticr2, crl),
	(B6, 6, exticr2, crl),
	(B7, 7, exticr2, crl),
	(B8, 8, exticr3, crh),
	(B9, 9, exticr3, crh),
	(B10, 10, exticr3, crh),
	(B11, 11, exticr3, crh),
	(B12, 12, exticr4, crh),
	(B13, 13, exticr4, crh),
	(B14, 14, exticr4, crh),
	(B15, 15, exticr4, crh),
}

gpio_as_fn!{
	PORTC, GPIOC, iopcen, 3,
	(C0, 0, exticr1, crl),
	(C1, 1, exticr1, crl),
	(C2, 2, exticr1, crl),
	(C3, 3, exticr1, crl),
	(C4, 4, exticr2, crl),
	(C5, 5, exticr2, crl),
	(C6, 6, exticr2, crl),
	(C7, 7, exticr2, crl),
	(C8, 8, exticr3, crh),
	(C9, 9, exticr3, crh),
	(C10, 10, exticr3, crh),
	(C11, 11, exticr3, crh),
	(C12, 12, exticr4, crh),
	(C13, 13, exticr4, crh),
	(C14, 14, exticr4, crh),
	(C15, 15, exticr4, crh),
}

gpio_as_var!{
	gpioa, GPIOA, Porta, iopaen, 0,
	pa0: (PA0, 0, exticr1, crl),
	pa1: (PA1, 1, exticr1, crl),
	pa2: (PA2, 2, exticr1, crl),
	pa3: (PA3, 3, exticr1, crl),
	pa4: (PA4, 4, exticr2, crl),
	pa5: (PA5, 5, exticr2, crl),
	pa6: (PA6, 6, exticr2, crl),
	pa7: (PA7, 7, exticr2, crl),
	pa8: (PA8, 8, exticr3, crh),
	pa9: (PA9, 9, exticr3, crh),
	pa11: (PA11, 10, exticr3, crh),
	pa12: (PA12, 11, exticr3, crh),
	pa10: (PA10, 12, exticr4, crh),
	pa13: (PA13, 13, exticr4, crh),
	pa14: (PA14, 14, exticr4, crh),
	pa15: (PA15, 15, exticr4, crh),
}

gpio_as_var!{
	gpiob, GPIOB, Portb, iopben, 1,
	pb0: (PB0, 0, exticr1, crl),
	pb1: (PB1, 1, exticr1, crl),
	pb2: (PB2, 2, exticr1, crl),
	pb3: (PB3, 3, exticr1, crl),
	pb4: (PB4, 4, exticr2, crl),
	pb5: (PB5, 5, exticr2, crl),
	pb6: (PB6, 6, exticr2, crl),
	pb7: (PB7, 7, exticr2, crl),
	pb8: (PB8, 8, exticr3, crh),
	pb9: (PB9, 9, exticr3, crh),
	pb11: (PB11, 10, exticr3, crh),
	pb12: (PB12, 11, exticr3, crh),
	pb10: (PB10, 12, exticr4, crh),
	pb13: (PB13, 13, exticr4, crh),
	pb14: (PB14, 14, exticr4, crh),
	pb15: (PB15, 15, exticr4, crh),
}

gpio_as_var!{
	gpioc, GPIOC, Portc, iopcen, 3,
	pc0: (PC0, 0, exticr1, crl),
	pc1: (PC1, 1, exticr1, crl),
	pc2: (PC2, 2, exticr1, crl),
	pc3: (PC3, 3, exticr1, crl),
	pc4: (PC4, 4, exticr2, crl),
	pc5: (PC5, 5, exticr2, crl),
	pc6: (PC6, 6, exticr2, crl),
	pc7: (PC7, 7, exticr2, crl),
	pc8: (PC8, 8, exticr3, crh),
	pc9: (PC9, 9, exticr3, crh),
	pc11: (PC11, 10, exticr3, crh),
	pc12: (PC12, 11, exticr3, crh),
	pc10: (PC10, 12, exticr4, crh),
	pc13: (PC13, 13, exticr4, crh),
	pc14: (PC14, 14, exticr4, crh),
	pc15: (PC15, 15, exticr4, crh),
}