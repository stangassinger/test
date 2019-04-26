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


#[macro_use]
mod util;
mod font;

const ILI9341_RESET: u8 = 0x01;
const ILI9341_SLEEP_OUT: u8 = 0x11;
const ILI9341_DISPLAY_ON: u8 = 0x29;
const ILI9341_MAC: u8 = 0x36;
// const ILI9341_PIXEL_FORMAT: u8 = 0x3A;
const ILI9341_RGB_INTERFACE: u8 = 0xB0;
const ILI9341_INTERFACE: u8 = 0xF6;


const WIDTH: usize = 320;
const HEIGHT: usize = 240;
const PITCH: usize = 250;
const COLS: u16 = 53;
const ROWS: u16 = 24;
const CHARH: u16 = 10;
const CHARW: u16 = 6;
const DEFAULT_COLOR: u8 = 7;
const DEFAULT_BKGRD: u8 = 0;

// main framebuffer
static mut FRAMEBUF: [u8; 250*320] = [0; 250*320];
// cursor framebuffer, just the cursor itself
static CURSORBUF: [u8; 6] = [127; 6];

// TX receive buffer
static mut RXBUF: Option<ArrayDeque<[u8; 256]>> = None;


static mut MARKER : bool = false;

fn fifo() -> &'static mut ArrayDeque<[u8; 256]> {
    unsafe { RXBUF.get_or_insert_with(ArrayDeque::new) }
}


#[entry]
fn main() -> ! {
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

    // LCD pins
    gpioa.pa3 .into_alternate_af14().set_speed(Speed::VeryHigh);
    gpioa.pa4 .into_alternate_af14().set_speed(Speed::VeryHigh);
    gpioa.pa6 .into_alternate_af14().set_speed(Speed::VeryHigh);
    gpioa.pa11.into_alternate_af14().set_speed(Speed::VeryHigh);
    gpioa.pa12.into_alternate_af14().set_speed(Speed::VeryHigh);
    gpiob.pb0 .into_alternate_af9() .set_speed(Speed::VeryHigh);
    gpiob.pb1 .into_alternate_af9() .set_speed(Speed::VeryHigh);
    gpiob.pb8 .into_alternate_af14().set_speed(Speed::VeryHigh);
    gpiob.pb9 .into_alternate_af14().set_speed(Speed::VeryHigh);
    gpiob.pb10.into_alternate_af14().set_speed(Speed::VeryHigh);
    gpiob.pb11.into_alternate_af14().set_speed(Speed::VeryHigh);
    gpioc.pc6 .into_alternate_af14().set_speed(Speed::VeryHigh);
    gpioc.pc7  .into_alternate_af14().set_speed(Speed::VeryHigh);
    gpioc.pc10.into_alternate_af14().set_speed(Speed::VeryHigh);
    gpiod.pd3 .into_alternate_af14().set_speed(Speed::VeryHigh);
    gpiod.pd6 .into_alternate_af14().set_speed(Speed::VeryHigh);
/*    gpioe.pe11.into_alternate_af14().set_speed(Speed::VeryHigh);
    gpioe.pe12.into_alternate_af14().set_speed(Speed::VeryHigh);
    gpioe.pe13.into_alternate_af14().set_speed(Speed::VeryHigh);
    gpioe.pe14.into_alternate_af14().set_speed(Speed::VeryHigh);
    gpioe.pe15.into_alternate_af14().set_speed(Speed::VeryHigh);
  */  gpiof.pf10.into_alternate_af14().set_speed(Speed::VeryHigh);
    gpiog.pg6.into_alternate_af14().set_speed(Speed::VeryHigh);
    gpiog.pg7.into_alternate_af14().set_speed(Speed::VeryHigh);
    gpiog.pg11.into_alternate_af14().set_speed(Speed::VeryHigh);
    gpiog.pg10.into_alternate_af9().set_speed(Speed::VeryHigh);
    gpiog.pg12.into_alternate_af9().set_speed(Speed::VeryHigh);



    // Enable interrupts
    let mut nvic = cp.NVIC;
    nvic.enable(board::hal::stm32::Interrupt::TIM3);
    nvic.enable(board::hal::stm32::Interrupt::USART3);
    led_blink_timer.listen(Event::TimeOut);

    // until here ok with
    //http://www.lucadavidian.com/2017/10/02/stm32-using-the-ltdc-display-controller/

// partes taken from 
//https://github.com/birkenfeld/stm_display_rust/ branch discovery
//and
//https://github.com/therealprof/stm32f429i-disc  1eb659f195c4d26f2320c52d37f82f7139605f93


// Enable clocks
    modif!(RCC.apb2enr: ltdcen = true);
    modif!(RCC.ahb1enr: dma2den = true);
    // Enable PLLSAI for LTDC
    //   PLLSAI_VCO Input = HSE_VALUE/PLL_M = 1 Mhz
    //   PLLSAI_VCO Output = PLLSAI_VCO Input * PLLSAI_N = 216 Mhz (f=100..432 MHz)
    //   PLLLCDCLK = PLLSAI_VCO Output/PLLSAI_R = 216/3 = 72 Mhz  (r=2..7)
    //   LTDC clock frequency = PLLLCDCLK / RCC_PLLSAIDivR = 72/8 = 9 Mhz (/2 /4 /8 /16)
    write!(RCC.pllsaicfgr: pllsain = 192, pllsaiq = 7, pllsair = 4);
    write!(RCC.dckcfgr: pllsaidivr = 0b01);  // divide by 4
    // Enable PLLSAI and wait for it
    modif!(RCC.cr: pllsaion = true);
    wait_for!(RCC.cr: pllsairdy);

    // Basic ChromArt configuration
    write!(DMA2D.fgpfccr: cm = 0b0101);  // L8 in/out
    write!(DMA2D.opfccr:  cm = 0b0101);

    // for scrolling up one line
    write!(DMA2D.fgmar: ma = FRAMEBUF.as_ptr().offset(CHARH as isize) as u32);
    write!(DMA2D.fgor: lo = CHARH);
    write!(DMA2D.omar: ma = FRAMEBUF.as_ptr() as u32);
    write!(DMA2D.oor: lo = CHARH);
    write!(DMA2D.nlr: pl = HEIGHT as u16, nl = WIDTH as u16);


    // Configure LCD timings
    write!(LTDC.sscr: hsw = 9, vsh = 1);            // Vsync, Hsync
    write!(LTDC.bpcr: ahbp = 29, avbp = 3);         // Back porch
    write!(LTDC.awcr: aav = 269, aah = 323);        // Active width
    write!(LTDC.twcr: totalw = 279, totalh = 327);  // Total width           

    // Configure layer 1 (main framebuffer)

    // Horizontal and vertical window (coordinates include porches)
    write!(LTDC.l1whpcr: whstpos = 30, whsppos = 269);
    write!(LTDC.l1wvpcr: wvstpos = 4,  wvsppos = 323);
    // Pixel format
    write!(LTDC.l1pfcr: pf = 0b101);  // 8-bit (CLUT enabled below)
    // Constant alpha value
    write!(LTDC.l1cacr: consta = 0xFF);
    // Default color values
    write!(LTDC.l1dccr:  dcalpha = 0, dcred = 0, dcgreen = 0, dcblue = 0);
    // Blending factors
    write!(LTDC.l1bfcr: bf1 = 4, bf2 = 5);  // Constant alpha
    // Color frame buffer start address
    write!(LTDC.l1cfbar: cfbadd = FRAMEBUF.as_ptr() as u32);
    // Color frame buffer line length (active*bpp + 3), and pitch
    write!(LTDC.l1cfblr: cfbll = 240 + 3, cfbp = 250);
    // Frame buffer number of lines
    write!(LTDC.l1cfblnr: cfblnbr = 320);
    
   // Set up 256-color ANSI LUT
    for v in 0..16 {
        let b = (v & 1 != 0) as u8;
        let g = (v & 2 != 0) as u8;
        let r = (v & 4 != 0) as u8;
        let i = (v & 8 != 0) as u8;
        write!(LTDC.l1clutwr: clutadd = v,
               red = 0x55*(r<<1 | i), green = 0x55*(g<<1 | i), blue = 0x55*(b<<1 | i));
    }
    for r in 0..6 {
        for g in 0..6 {
            for b in 0..6 {
                write!(LTDC.l1clutwr: clutadd = 16 + 36*b + 6*g + r,
                       red = 0x33*r, green = 0x33*g, blue = 0x33*b);
            }
        }
    }
    for i in 0..24 {
        write!(LTDC.l1clutwr: clutadd = 232+i, red = 8+10*i, green = 8+10*i, blue = 8+10*i);
    }


    // Configure layer 2 (cursor)

    // Initial position: top left character
    write!(LTDC.l2whpcr: whstpos = 30+9, whsppos = 30+9);
    write!(LTDC.l2wvpcr: wvstpos = 4,  wvsppos = 4+6-1);
    write!(LTDC.l2pfcr: pf = 0b101);  // L-8 without CLUT
    write!(LTDC.l2cacr: consta = 0xFF);
    write!(LTDC.l2dccr: dcalpha = 0, dcred = 0, dcgreen = 0, dcblue = 0);
    write!(LTDC.l2bfcr: bf1 = 6, bf2 = 7);  // Constant alpha * Pixel alpha
    write!(LTDC.l2cfbar: cfbadd = CURSORBUF.as_ptr() as u32);
    write!(LTDC.l2cfblr: cfbll = 1 + 3, cfbp = 1);
    write!(LTDC.l2cfblnr: cfblnbr = 6);

    // Enable layer1, disable layer2 initially
    modif!(LTDC.l1cr: cluten = true, len = true);
    modif!(LTDC.l2cr: len = false);

    // Reload config (immediate)
    write!(LTDC.srcr: imr = true);

    // Dither on, display on
    modif!(LTDC.gcr: den = true, ltdcen = true);

    // Reload config to show display
    write!(LTDC.srcr: imr = true);



       // Get delay provider
        let mut timer = Delay::new( cp.SYST, clocks);


    // Initialize LCD controller  
    ili_cmd!(display_spi, cs, ds, ILI9341_RESET);
    timer.delay_ms(5u16);
    ili_cmd!(display_spi, cs, ds, ILI9341_MAC, 0xC0);
    ili_cmd!(display_spi, cs, ds, ILI9341_RGB_INTERFACE, 0xC2);
    ili_cmd!(display_spi, cs, ds, ILI9341_INTERFACE, 0x01, 0x00, 0x06);
    ili_cmd!(display_spi, cs, ds, ILI9341_SLEEP_OUT);
    timer.delay_ms(60u16);
    ili_cmd!(display_spi, cs, ds, ILI9341_DISPLAY_ON);




    draw(COLS-3, 1, b'O', 0b1010, 0b1100);
    draw(COLS-2, 1, b'K', 0b1010, 0b1100);
    draw(COLS-20, 10, b'-', 0b1010, 0b1100);
    draw(COLS-19, 10, b'a', 0b1010, 0b1100);
    draw(COLS-18, 10, b'l', 0b1010, 0b1100);
    draw(COLS-17, 10, b'l', 0b1010, 0b1100);
    draw(COLS-16, 10, b'0', 0b1010, 0b1100);
    draw(COLS-15, 10, b'B', 0b1010, 0b1100);
    draw(COLS-14, 10, b'l', 0b1010, 0b1100);
    draw(COLS-13, 10, b'u', 0b1010, 0b1100);    

        main_loop(console_tx);
     
        loop {
            // Turn LED on
            led_green.set_high();
           // blink( &mut true);

            // Delay twice for half a second due to limited timer resolution
            timer.delay_ms(100_u32);

            // Turn LED off
            led_green.set_low();
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

    loop {
        continue;
    }
}



fn process_escape(end: u8, seq: &[u8], cx: &mut u16, cy: &mut u16, color: &mut u8, bkgrd: &mut u8) {
    let mut args = seq.split(|&v| v == b';').map(|n| btoi(n).unwrap_or(0));
    match end {
        b'm' => while let Some(arg) = args.next() {
            match arg {
                0  => { *color = DEFAULT_COLOR; *bkgrd = DEFAULT_BKGRD; }
                1  => { *color |= 0b1000; } // XXX: only for 16colors
                22 => { *color &= !0b1000; }
                30...37 => { *color = arg as u8 - 30; }
                40...47 => { *bkgrd = arg as u8 - 40; }
                38 => { *color = args.nth(1).unwrap_or(0) as u8; }
                48 => { *bkgrd = args.nth(1).unwrap_or(0) as u8; }
                _ => {}
            }
        },
        b'H' | b'f' => {
            let y = args.next().unwrap_or(1);
            let x = args.next().unwrap_or(1);
            *cx = if x > 0 { x-1 } else { 0 };
            *cy = if y > 0 { y-1 } else { 0 };
        },
        b'A' => {
            let n = args.next().unwrap_or(1).max(1);
            *cy -= n.min(*cy);
        },
        b'B' => {
            let n = args.next().unwrap_or(1).max(1);
            *cy += n.min(ROWS - *cy - 1);
        },
        b'C' => {
            let n = args.next().unwrap_or(1).max(1);
            *cx += n.min(COLS - *cx - 1);
        },
        b'D' => {
            let n = args.next().unwrap_or(1).max(1);
            *cx -= n.min(*cx);
        },
        b'G' => {
            let x = args.next().unwrap_or(1).max(1);
            *cx = x-1;
        }
        b'J' => {}, // TODO: erase screen
        b'K' => {}, // TODO: erase line
        // otherwise, ignore
        _    => {}
    }
}



fn draw(cx: u16, cy: u16, ch: u8, color: u8, bkgrd: u8) {
    font::FONT[ch as usize].iter().zip(cy*CHARH..(cy+1)*CHARH).for_each(|(charrow, y)| {
        (0..CHARW).for_each(|x| unsafe {
            FRAMEBUF[(x + cx*CHARW) as usize * PITCH + y as usize] =
                if charrow & (1 << (CHARW - 1 - x)) != 0 { color } else { bkgrd };
        });
    });
}

fn cursor(cx: u16, cy: u16) {
    write!(LTDC.l2whpcr: whstpos = 30 + 9 + cy*CHARH, whsppos = 30 + 9 + cy*CHARH);
    write!(LTDC.l2wvpcr: wvstpos = 4 + cx*CHARW, wvsppos = 4 + 6 - 1 + cx*CHARW);
    // reload on next vsync
    write!(LTDC.srcr: vbr = true);
}




fn main_loop(mut console_tx: hal::serial::Tx< board::stm32::USART3>) -> ! {
    let mut cx = 0;
    let mut cy = 0;
    let mut color = DEFAULT_COLOR;
    let mut bkgrd = DEFAULT_BKGRD;
    let mut escape = 0;
    let mut escape_len = 0;
    let mut escape_seq = [0u8; 36];

    loop {
        if let Some(ch) = fifo().pop_front() {
            block!(console_tx.write(ch)).unwrap();

            if escape == 1 {
                escape_len = 0;
                escape = if ch == b'[' { 2 } else { 0 };
                continue;
            } else if escape == 2 {
                if (ch >= b'0' && ch <= b'9') || ch == b';' {
                    escape_seq[escape_len] = ch;
                    escape_len += 1;
                    if escape_len == escape_seq.len() {
                        escape = 0;
                    }
                } else {
                    process_escape(ch, &escape_seq[..escape_len],
                                   &mut cx, &mut cy, &mut color, &mut bkgrd);
                    cursor(cx, cy);
                    escape = 0;
                }
                continue;
            }

            if ch == b'\r' {
                // do nothing
            } else if ch == b'\n' {
                cx = 0;
                cy += 1;
                if cy == ROWS {
                    // scroll one row using DMA
                    modif!(DMA2D.cr: mode = 0, start = true);
                    wait_for!(DMA2D.cr: start);
                    cy -= 1;
                }
                cursor(cx, cy);
            } else if ch == b'\x08' {
                if cx > 0 {
                    cx -= 1;
                    draw(cx, cy, b' ', color, bkgrd);
                    cursor(cx, cy);
                }
            } else if ch == b'\x1b' {
                escape = 1;
            } else {
                draw(cx, cy, ch, color, bkgrd);
                cx = (cx + 1) % COLS;
                cursor(cx, cy);
            }
        }
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
    modif!(LTDC.l2cr: len = bit(*visible));
    write!(LTDC.srcr: vbr = true);

    // Reset timer
    modif!(TIM3.sr: uif = false);
    modif!(TIM3.cr1: cen = true);
}



#[cortex_m_rt::exception]
fn HardFault(ef: &cortex_m_rt::ExceptionFrame) -> ! {
    panic!("HardFault at {:#?}", ef);
}

#[cortex_m_rt::exception]
fn DefaultHandler(irqn: i16) {
    panic!("Unhandled exception (IRQn = {})", irqn);
}
