#![no_std]
#![no_main]

mod mmc;
mod nucleo;

use core::panic::PanicInfo;

use boards::ui::rgb_led::RgbLed as _;
use cortex_m_rt::{exception, ExceptionFrame};
use cortex_m_semihosting::hprintln;

use self::nucleo::Led;

#[rtic::app(device = stm32n6::stm32n657)]
mod app {
    use boards::ui::{
        buttons::UserPresence as _,
        rgb_led::{Intensities, RgbLed as _},
    };
    use cortex_m_semihosting::hprintln;
    use stm32n657_hal::{
        bsec::Bsec,
        gpio::{GpioA, GpioC, GpioE, GpioG},
        mmc::MmcMaster,
        rcc::{ClockConfig, Rcc},
        timer::{MillisecondsCounter, Tim6, Tim7, Timer},
        Rate,
    };
    use systick_monotonic::Systick;
    use trussed::platform::consent;

    use crate::mmc::Mmc;
    use crate::nucleo::{Button, Led};

    #[monotonic(binds = SysTick, default = true)]
    type Monotonic = Systick<100>;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        led: Led,
        button: Button,
        timer: Timer<Tim6>,
        counter: MillisecondsCounter<Tim7>,
        // mmc: Mmc,
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local, init::Monotonics) {
        let bsec = Bsec::new(cx.device.BSEC);
        let uid = bsec.uid();

        hprintln!("nkso3 firmware is running on {:x?}", uid).ok();

        let rcc = Rcc::new(cx.device.RCC);
        let clock_config = rcc.clock_config();
        assert_eq!(clock_config, ClockConfig::DEFAULT);

        let monotonic = Systick::new(cx.core.SYST, clock_config.sys_bus_ck().to_Hz());

        let gpiog = GpioG::new(cx.device.GPIOG_S, &rcc);
        let led = Led::init(gpiog.g10, gpiog.g0, gpiog.g8);

        let gpioc = GpioC::new(cx.device.GPIOC_S, &rcc);
        let button = Button::init(gpioc.c13);

        let gpioa = GpioA::new(cx.device.GPIOA_S, &rcc);
        let gpioe = GpioE::new(cx.device.GPIOE_S, &rcc);
        hprintln!("Before pins").ok();
        let pins = (
            gpioa.a0.into_sdmmc2_cmd(),
            gpioc.c2.into_sdmmc2_ck(),
            gpioc.c4.into_sdmmc2_d0(),
            // gpioc.c5.into_sdmmc2_d1(),
            // gpioc.c0.into_sdmmc2_d2(),
            // gpioe.e4.into_sdmmc2_d3(),
        );
        hprintln!("after pins").ok();
        let mmc = MmcMaster::new(cx.device.SDMMC2_S, pins);

        hprintln!("before enable").ok();
        let mmc = mmc.enable(&rcc).expect("Enabling mmc");
        hprintln!("after enable").ok();

        let tim7 = Tim7::new(cx.device.TIM7_S, &rcc);
        let counter = MillisecondsCounter::new(tim7, clock_config);

        let tim6 = Tim6::new(cx.device.TIM6_S, &rcc);
        let mut timer = Timer::new(tim6, clock_config);
        timer.start(Rate::Hz(100));

        (
            Shared {},
            Local {
                counter,
                led,
                button,
                timer,
                // mmc,
            },
            init::Monotonics(monotonic),
        )
    }

    #[idle(local = [led, button, timer, counter])]
    fn idle(cx: idle::Context) -> ! {
        let idle::LocalResources {
            led,
            button,
            timer,
            counter,
        } = cx.local;

        let start = counter.now();
        let mut cycle_start = start;
        loop {
            hprintln!("idle").ok();
            let user_presence = button.check_user_presence();
            let is_user_present = user_presence != consent::Level::None;

            let now = counter.now();
            let elapsed = now.checked_duration_since(cycle_start).unwrap().to_millis();
            if elapsed >= 1_000 {
                cycle_start = now;

                let total_elapsed = now.checked_duration_since(start).unwrap();
                hprintln!("{}", total_elapsed).ok();
            }

            let mut intensities = Intensities::from(0);
            if elapsed < 500 {
                intensities.green = u8::MAX;
            }
            if is_user_present {
                intensities.blue = u8::MAX;
            }
            led.set(intensities);

            nb::block!(timer.wait()).ok();
        }
    }
}

#[inline(never)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    Led::set_panic_led();
    hprintln!("{}", info).ok();
    loop {
        cortex_m::asm::wfi();
    }
}

#[exception]
unsafe fn HardFault(ef: &ExceptionFrame) -> ! {
    hprintln!("HardFault: {:?}", ef).ok();
    loop {
        cortex_m::asm::wfi();
    }
}
