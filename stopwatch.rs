#![no_std]
#![no_main]

use panic_halt as _;


use embedded_graphics::image::{Image, ImageRaw};
use embedded_graphics::mono_font::{ascii::FONT_5X8, MonoTextStyleBuilder};
use embedded_graphics::pixelcolor::raw::LittleEndian;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{PrimitiveStyle, Rectangle};
use embedded_graphics::text::Text;
use longan_nano::hal::{pac, prelude::*};
use longan_nano::{lcd, lcd_pins};


use longan_nano::hal::delay::McycleDelay;
use riscv_rt::entry;

use embedded_hal::digital::v2::InputPin;
use gd32vf103xx_hal::pac::Interrupt;
use gd32vf103xx_hal::timer;
use gd32vf103xx_hal::timer::Timer;
use longan_nano::hal::{eclic::*, pac::*};
use longan_nano::led::{rgb, Led, RED};

static mut R_LED: Option<RED> = None;
static mut G_TIMER1: Option<Timer<TIMER1>> = None;
static mut G_COLOR: Rgb565 = Rgb565::BLACK;
static mut G_START: bool = false;
static mut G_C2: usize = 0;

const FERRIS: &[u8] = include_bytes!("ferris.raw");

#[entry]
fn main() -> ! {
    let digit: [&str; 10] = ["0", "1", "2", "3", "4", "5", "6", "7", "8", "9"];
    let dp = pac::Peripherals::take().unwrap();

    // Configure clocks
    let mut rcu = dp
        .RCU
        .configure()
        .ext_hf_clock(8.mhz())
        .sysclk(108.mhz())
        .freeze();

    let mut afio = dp.AFIO.constrain(&mut rcu);
    let mut pmu: PMU = dp.PMU;

    let usart0: USART0 = dp.USART0;

    let gpioa = dp.GPIOA.split(&mut rcu);
    let gpiob = dp.GPIOB.split(&mut rcu);
    let gpioc = dp.GPIOC.split(&mut rcu);

    let lcd_pins = lcd_pins!(gpioa, gpiob);
    let mut lcd = lcd::configure(dp.SPI0, lcd_pins, &mut afio, &mut rcu);
    let (width, height) = (lcd.size().width as i32, lcd.size().height as i32);
    let (mut red, mut green, mut blue) = rgb(gpioc.pc13, gpioa.pa1, gpioa.pa2);

    red.off();
    green.off();
    blue.off();
    //interrupt
  
    unsafe {
        R_LED = Some(red);
    };

    ECLIC::reset();
    ECLIC::set_threshold_level(Level::L0);
    ECLIC::set_level_priority_bits(LevelPriorityBits::L3P1);
    // timer
    let mut timer = Timer::timer1(dp.TIMER1, 1.hz(), &mut rcu);
    timer.listen(timer::Event::Update);
    unsafe { G_TIMER1 = Some(timer) };

    ECLIC::setup(
        Interrupt::TIMER1,
        TriggerType::Level,
        Level::L1,
        Priority::P1,
    );
    unsafe {
        ECLIC::unmask(Interrupt::TIMER1);
        riscv::interrupt::enable();
    };

    let style1 = MonoTextStyleBuilder::new()
        .font(&FONT_5X8)
        .text_color(Rgb565::BLACK)
        .background_color(Rgb565::RED)
        .build();

    let style2 = MonoTextStyleBuilder::new()
        .font(&FONT_5X8)
        .text_color(Rgb565::BLACK)
        .background_color(Rgb565::GREEN)
        .build();

    let mut i_style = 0;
    let mut styles = [style1, style2];
    let mut style = styles[i_style];
    let mut delay = McycleDelay::new(&rcu.clocks);
    let mut ip = 0;
    let mut adelay = 0;
    let mut hdelay:[i32;3]=[0,0,0];
    let mut i_hd=0;
    //Draw background
    unsafe {
        Rectangle::new(Point::new(0, 0), Size::new(width as u32, height as u32))
            .into_styled(PrimitiveStyle::with_fill(G_COLOR))
            .draw(&mut lcd)
            .unwrap();
    }

    let raw_image: ImageRaw<Rgb565, LittleEndian> = ImageRaw::new(&FERRIS, 86);


    Image::new(&raw_image, Point::new(0, 0))
        .draw(&mut lcd)
        .unwrap();

    let mut i1 = 0;

    let rx = gpioa.pa8.into_floating_input();
    let mut pstate = rx.is_high();
    let mut local_c2: usize = 0;
    let mut pstate_bool: bool;

    loop {
        i1 += 1;

        pstate = rx.is_high();
        pstate_bool = pstate.unwrap();

        //Draw text box according to value of ip

        //Toggle button
        // basic idea (revised): when button is down, adelay fills up. When button is not down,
        //check if adelay is 'full', then process according to 'fullness'. Except holding
        //is partially processed while button is still down.
       if pstate_bool{

        //fill adelay
        adelay+=1;
        hdelay[i_hd]+=1;
        if adelay>500{
            
            unsafe { G_C2 = 0 };
            unsafe {
                G_START = false;
            }
            i_style = 0;
            style = styles[(i_style) % 2 as usize];
        }

       }
       else{
        //button is not down
        if adelay>2 && adelay <500{
            unsafe {
                G_START = !G_START;
            }
            i_style += 1;
            style = styles[(i_style) % 2 as usize];

            adelay=0;
        }

        if adelay>500{
            unsafe {
                G_START = false;
            }
           

            adelay=0;
        }
        
       }

       unsafe {
        local_c2 = G_C2;
    }
        Text::new(
            digit[(((pstate_bool as usize) / 1) % 10)],
            Point::new(40, 50),
            style,
        )
        .draw(&mut lcd)
        .unwrap();

        Text::new(
            digit[((local_c2) / 60 / 60 % 24) % 10],
            Point::new(90, 50),
            style,
        )
        .draw(&mut lcd)
        .unwrap();

        Text::new(
            digit[((local_c2) / 60 / 60 % 24) / 10 % 10],
            Point::new(85, 50),
            style,
        )
        .draw(&mut lcd)
        .unwrap();

        Text::new(
            digit[((local_c2) / 60 % 60) % 10],
            Point::new(105, 50),
            style,
        )
        .draw(&mut lcd)
        .unwrap();

        Text::new(
            digit[((local_c2) / 60 % 60) / 10 % 10],
            Point::new(100, 50),
            style,
        )
        .draw(&mut lcd)
        .unwrap();

        Text::new(digit[((local_c2) % 60) % 10], Point::new(120, 50), style)
            .draw(&mut lcd)
            .unwrap();

        Text::new(
            digit[((local_c2) % 60) / 10 % 10],
            Point::new(115, 50),
            style,
        )
        .draw(&mut lcd)
        .unwrap();
    }
}

#[allow(non_snake_case)]
#[no_mangle]
fn TIMER1() {
    unsafe {
        if (G_START) {
            G_C2 += 1;
        }

        if let Some(timer1) = G_TIMER1.as_mut() {
            timer1.clear_update_interrupt_flag();
        }

        if let Some(led) = R_LED.as_mut() {
            if (G_START) {
                if led.is_on() {
                    led.off();
                } else {
                    led.on();
                }
            }
        }
    }
}
