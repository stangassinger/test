#![no_main]
#![no_std]
// could try this:
// http://nercury.github.io/rust/embedded/experiments/2019/01/27/rust-embedded-02-measuring-the-clock.html

pub extern crate stm32f4xx_hal as hal;

extern crate cortex_m;
extern crate cortex_m_rt;


extern crate panic_halt;

extern crate stm32f429i_disc as board;
extern crate arraydeque;
extern crate btoi;
use cortex_m_rt::entry;

use board::hal::delay::Delay;
use board::hal::prelude::*;
use board::hal::stm32;
use board::hal::time::*;
use board::hal::timer::{Timer, Event};
use board::hal::gpio::Speed;
use board::hal::spi::Spi;
use board::hal::serial::{Serial, config::Config as SerialConfig};

use cortex_m::peripheral::Peripherals;

use arraydeque::ArrayDeque;
use board::nb::block;
use btoi::btoi;


// used when invoking C code to configure system clock
 extern "C" {
     fn HAL_Init();
     fn HAL_IncTick();
     fn SystemClock_Config();
     fn MX_GPIO_Init();
     fn amy_func();
     fn amy_delay() -> u32; 
 }





#[macro_use]




static mut MARKER : bool = false;




#[entry]
fn main() -> ! {
   // used when invoking C code to configure system clock
    unsafe { HAL_Init(); }
    unsafe { SystemClock_Config(); }    
    unsafe { MX_GPIO_Init(); }
    unsafe { amy_func(); }
    unsafe { amy_delay(); }
    



/*
    if let (Some(p), Some(cp)) = (stm32::Peripherals::take(), Peripherals::take()) {
        // Constrain clock registers
        let mut rcc = p.RCC.constrain();

        // Configure clock to 168 MHz  and freeze it
        rcc.cfgr = rcc.cfgr.sysclk(MegaHertz(168))
                        .hclk(MegaHertz(168))
                        .pclk1(MegaHertz(42))
                        .pclk2(MegaHertz(84));
        let clocks = rcc.cfgr.freeze();

// Set up pins
    let gpioa = p.GPIOA.split();
    let gpiob = p.GPIOB.split();
    let gpioc = p.GPIOC.split();
    let gpiod = p.GPIOD.split();
    let gpioe = p.GPIOE.split();   
    let gpiof = p.GPIOF.split();   
    let gpiog = p.GPIOG.split();

    // LCD SPI
    let mut cs = gpioc.pc2.into_push_pull_output();
    let mut ds = gpiod.pd13.into_push_pull_output();
    let sclk = gpiof.pf7.into_alternate_af5();
    let miso = gpiof.pf8.into_alternate_af5();
    let mosi = gpiof.pf9.into_alternate_af5();
    let mut display_spi = Spi::spi5( p.SPI5, (sclk, miso, mosi),
                                    embedded_hal::spi::MODE_0,
                                    Hertz(1_000_000), clocks);    


    // Set up blinking timer
    let mut led_blink_timer = Timer::tim3(p.TIM3, Hertz(4), clocks);

    // (Re-)configure PG13 (green LED) as output
    let mut led_green = gpiog.pg13.into_push_pull_output(); 
    let mut led_red   = gpiog.pg14.into_push_pull_output(); 


    // LCD backlight enable
 //   let mut backlight = gpiod.pd12.into_push_pull_output();
 //   backlight.set_high();

    // Output pin connected to Boot0 + capacitor
 //   let mut bootpin = gpiob.pb7.into_push_pull_output();
  //  bootpin.set_low();


  // Console UART (UART #3)
    let utx = gpiod.pd8 .into_alternate_af7();
    let urx = gpiod.pd9 .into_alternate_af7();
    let mut console_uart = Serial::usart3(p.USART3, (utx, urx),
                                          SerialConfig::default().baudrate(Bps(115200)),
                                          clocks).unwrap();
    console_uart.listen(hal::serial::Event::Rxne);
    let (console_tx, _) = console_uart.split();

  

       // Get delay provider
        let mut timer = Delay::new( cp.SYST, clocks);

*/

      /*  
        loop {
            // Turn LED on
            led_green.set_high();
            led_red.set_low();
           // blink( &mut true);

            // Delay twice for half a second due to limited timer resolution
            timer.delay_ms(100_u32);

            // Turn LED off
            led_green.set_low();
            led_red.set_high();
           // blink( &mut false );

            // Delay twice for half a second due to limited timer resolution
            timer.delay_ms(100_u32);

unsafe{ 
            if MARKER == true{
                led_red.set_low();
            } else{
                led_red.set_high();
            } 
       }


        }
        
    }
*/
    let mut output : u32 = 0;
    loop {
        output = unsafe { amy_delay() };
        unsafe { amy_func() }
        
        
        continue;
    }
}









board::hal::stm32f4::interrupt!(TIM3, led_blink, state: bool = false);

fn led_blink(visible: &mut bool) {
    *visible = !*visible;
    unsafe{ 
    MARKER = !MARKER;
    } 
    /*
    if *visible == true{
        //led_green.set_low();   
        modif!( )
    } else{

    } */  

    // Reset timer
  //  modif!(TIM3.sr: uif = false);
  //  modif!(TIM3.cr1: cen = true);
}



#[cortex_m_rt::exception]
fn HardFault(ef: &cortex_m_rt::ExceptionFrame) -> ! {
    panic!("HardFault at {:#?}", ef);
}

#[cortex_m_rt::exception]
fn DefaultHandler(irqn: i16) {
    panic!("Unhandled exception (IRQn = {})", irqn);
}



// used when invoking C code to configure system clock
//
#[cortex_m_rt::exception]
fn SysTick() {
     unsafe { HAL_IncTick() }
}