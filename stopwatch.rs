#![no_std]
#![no_main]

use panic_halt as _;

use embedded_graphics::mono_font::{
    ascii::FONT_5X8,
    MonoTextStyleBuilder,
};
use core::arch::asm;
use riscv::asm::wfi;
use embedded_graphics::image::{Image, ImageRaw};
use embedded_graphics::pixelcolor::raw::LittleEndian;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{Rectangle, PrimitiveStyle};
use embedded_graphics::text::Text;
use longan_nano::hal::{pac, prelude::*};
use longan_nano::{lcd, lcd_pins};

use riscv_rt::entry;
use longan_nano::hal::delay::McycleDelay;


use gd32vf103xx_hal::pac::Interrupt;
use longan_nano::hal::{ pac::*, eclic::*};
use gd32vf103xx_hal::timer;
use gd32vf103xx_hal::timer::Timer;
use longan_nano::led::{rgb, Led, RED};
use embedded_hal::digital::v2::InputPin;

static mut R_LED: Option<RED> = None;
static mut G_TIMER1: Option<Timer<TIMER1>> = None;
static mut glb_stylecolor:Rgb565= Rgb565::BLACK;
static mut G_START:bool=false;
 static mut i2:usize=0;


const FERRIS: &[u8] = include_bytes!("ferris.raw");

#[entry]
fn main() -> ! {
    //let digit: [char; 10] = ['0','1','2','3','4','5','6','7','8','9'];
    let digit: [&str; 10] = ["0","1","2","3","4","5","6","7","8","9"];
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
    //let mut bkp: BackupDomain = dp.BKP.configure(&mut rcu, &mut pmu);

    let usart0: USART0= dp.USART0;
    
    let gpioa = dp.GPIOA.split(&mut rcu);
    let gpiob = dp.GPIOB.split(&mut rcu);
    let gpioc = dp.GPIOC.split(&mut rcu);

    let lcd_pins = lcd_pins!(gpioa, gpiob);
    let mut lcd = lcd::configure(dp.SPI0, lcd_pins, &mut afio, &mut rcu);
    let (width, height) = (lcd.size().width as i32, lcd.size().height as i32);

//interrupt
    let (mut red, mut green, mut blue) = rgb(gpioc.pc13, gpioa.pa1, gpioa.pa2);


    red.off();
    green.off();
    blue.off();
    unsafe { R_LED = Some(red); };

    ECLIC::reset();
    ECLIC::set_threshold_level(Level::L0);
    ECLIC::set_level_priority_bits(LevelPriorityBits::L3P1);
 // timer
 let mut timer =  Timer::timer1(dp.TIMER1, 1.hz(), &mut rcu);
 timer.listen(timer::Event::Update);
 unsafe {G_TIMER1 = Some(timer)};

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

    // Clear screen
 
       

        
    let style = MonoTextStyleBuilder::new()
        .font(&FONT_5X8)
        .text_color(Rgb565::BLACK)
        .background_color(Rgb565::GREEN)
        .build();


     let mut delay = McycleDelay::new(&rcu.clocks);
     let mut ip=0;

     

        //Draw background
        unsafe{Rectangle::new(Point::new(0, 0), Size::new(width as u32, height as u32))
            .into_styled(PrimitiveStyle::with_fill(glb_stylecolor))
            .draw(&mut lcd)
            .unwrap();
        }
       
            
        let raw_image: ImageRaw<Rgb565, LittleEndian> = ImageRaw::new(&FERRIS, 86);
       // Image::new(&raw_image, Point::new(width / 2 - 43, height / 2 - 32))
         //   .draw(&mut lcd)
           // .unwrap();   

            Image::new(&raw_image, Point::new(0, 0))
            .draw(&mut lcd)
            .unwrap();   
        
  
       let mut i1=0; 
     
    let rx=gpioa.pa8.into_floating_input();
    let mut pstate=rx.is_high();

    
    loop {
    i1+=1;
  
        pstate=rx.is_high();


        //Draw text box according to value of ip
unsafe{  
    if pstate.unwrap(){
            G_START=!G_START;
        }
 
        Text::new(digit[(((pstate.unwrap() as usize)/1)%10)], Point::new(40, 50), style)
        .draw(&mut lcd)
        .unwrap();
        
       




        Text::new(digit[((i2)/60/60%24)%10], Point::new(100, 50), style)
        .draw(&mut lcd)
        .unwrap();

        Text::new(digit[((i2)/60/60%24)/10%10], Point::new(95, 50), style)
        .draw(&mut lcd)
        .unwrap();



        Text::new(digit[((i2)/60%60)%10], Point::new(115, 50), style)
        .draw(&mut lcd)
        .unwrap();

        Text::new(digit[((i2)/60%60)/10%10], Point::new(110, 50), style)
        .draw(&mut lcd)
        .unwrap();



        Text::new(digit[((i2)%60)%10], Point::new(130, 50), style)
        .draw(&mut lcd)
        .unwrap();

        Text::new(digit[((i2)%60)/10%10], Point::new(125, 50), style)
        .draw(&mut lcd)
        .unwrap();


}












          //  delay.delay_ms(1);
            //sleep
    }
}


#[allow(non_snake_case)]
#[no_mangle]
fn TIMER1() {
    unsafe {

        if(G_START)
        {i2+=1;}

        if let Some(timer1) = G_TIMER1.as_mut() {
            timer1.clear_update_interrupt_flag();
        }
        if let Some(led) = R_LED.as_mut() {
            if led.is_on() {
                led.off();
            } else {
                led.on();
            }
       
        }
    }
}
