//! General-purpose I/Os (GPIO), see Section 15 of RM0486.

use core::{convert::Infallible, marker::PhantomData};

use embedded_hal::digital::v2::{InputPin, OutputPin};
use stm32n6::stm32n657::{GPIOA_S, GPIOB_S, GPIOC_S, GPIOD_S, GPIOE_S, GPIOG_S, GPIOH_S};

use crate::rcc::{Peripheral, Rcc};

pub struct Input<M> {
    _marker: PhantomData<M>,
}

pub struct Floating;

pub struct PullDown;
pub struct PullUp;

pub struct Output<M> {
    _marker: PhantomData<M>,
}

pub struct Alternate<M, const F: u8> {
    _marker: PhantomData<M>,
}

pub struct PushPull;

macro_rules! impl_gpio {
    ($gpio:ident, $GPIO:ident, [
        $($pin:ident: $Pin:ident = ($mode:ident, $ot:ident, $pupd:ident, $id:ident, $bs:ident, $br:ident, $afr:ident, $afsel:ident),)*
    ]) => {
        pub struct $gpio {
            $(
                pub $pin: $Pin<Input<Floating>>,
            )*
        }

        impl $gpio {
            pub fn new(gpio: $GPIO, rcc: &Rcc) -> Self {
                let _ = gpio;
                rcc.enable(Peripheral::$gpio);
                Self {
                    $(
                        $pin: $Pin {
                            _marker: Default::default(),
                        },
                    )*
                }
            }
        }

        $(
            impl_pin!($GPIO, $Pin, $mode, $ot, $pupd, $id, $bs, $br, $afr, $afsel);
        )*
    }
}

macro_rules! impl_pin {
    ($GPIO:ident, $pin:ident, $mode:ident, $ot:ident, $pupd:ident, $id:ident, $bs:ident, $br:ident, $afr:ident, $afsel:ident) => {
        pub struct $pin<M> {
            _marker: PhantomData<M>,
        }

        impl<M> $pin<M> {
            fn gpio(&self) -> $GPIO {
                // SAFETY: This struct can only be constructed by consuming the peripheral, so
                // there can be no other instances accessing the same pin.
                unsafe { $GPIO::steal() }
            }

            pub fn into_pull_down_input(self) -> $pin<Input<PullDown>> {
                // mode: 00 = general-purpose input mode
                self.gpio()
                    .moder()
                    .modify(|_, w| unsafe { w.$mode().bits(0b00) });
                // pupd: 10 = pull-down
                self.gpio()
                    .pupdr()
                    .modify(|_, w| unsafe { w.$pupd().bits(0b10) });
                $pin {
                    _marker: Default::default(),
                }
            }

            pub fn into_pull_up_input(self) -> $pin<Input<PullUp>> {
                // mode: 00 = general-purpose input mode
                self.gpio()
                    .moder()
                    .modify(|_, w| unsafe { w.$mode().bits(0b00) });
                // pupd: 01 = pull-up
                self.gpio()
                    .pupdr()
                    .modify(|_, w| unsafe { w.$pupd().bits(0b01) });
                $pin {
                    _marker: Default::default(),
                }
            }

            pub fn into_push_pull_output(self) -> $pin<Output<PushPull>> {
                // mode: 01 = general-purpose output mode
                self.gpio()
                    .moder()
                    .modify(|_, w| unsafe { w.$mode().bits(0b01) });
                // ot: 0 = output push-pull
                self.gpio().otyper().modify(|_, w| w.$ot().clear_bit());
                // pp: 00 = no pull-up, pull-down
                self.gpio()
                    .pupdr()
                    .modify(|_, w| unsafe { w.$pupd().bits(0b00) });
                $pin {
                    _marker: Default::default(),
                }
            }

            pub fn into_alternate_pull_up<const F: u8>(self) -> $pin<Alternate<PullUp, F>> {
                const {
                    assert!(F <= 0xF);
                };
                // mode: 10 = alternate function
                self.gpio()
                    .moder()
                    .modify(|_, w| unsafe { w.$mode().bits(0b10) });
                // pupd: 01 = pull-up
                self.gpio()
                    .pupdr()
                    .modify(|_, w| unsafe { w.$pupd().bits(0b10) });
                self.gpio()
                    .$afr()
                    .modify(|_, w| unsafe { w.$afsel().bits(F) });

                $pin {
                    _marker: Default::default(),
                }
            }
        }

        impl<M> $pin<Input<M>> {
            fn input(&self) -> bool {
                self.gpio().idr().read().$id().bit()
            }
        }

        impl<M> InputPin for $pin<Input<M>> {
            type Error = Infallible;

            fn is_high(&self) -> Result<bool, Self::Error> {
                Ok(self.input())
            }

            fn is_low(&self) -> Result<bool, Self::Error> {
                Ok(!self.input())
            }
        }

        impl<M> OutputPin for $pin<Output<M>> {
            type Error = Infallible;

            fn set_high(&mut self) -> Result<(), Self::Error> {
                self.gpio().bsrr().write(|w| w.$bs().set_bit());
                Ok(())
            }

            fn set_low(&mut self) -> Result<(), Self::Error> {
                self.gpio().bsrr().write(|w| w.$br().set_bit());
                Ok(())
            }
        }
    };
}

impl_gpio!(GpioA, GPIOA_S, [
    a0: PinA0 = (mode0, ot0, pupd0, id0, bs0, br0, afrl,afsel0),
    a4: PinA4 = (mode4, ot4, pupd4, id4, bs4, br4, afrl,afsel4),
]);
impl_gpio!(GpioB, GPIOB_S, [
    b4: PinB4 = (mode4, ot4, pupd4, id4, bs4, br4, afrl,afsel4),
    b5: PinB5 = (mode5, ot5, pupd5, id5, bs5, br5, afrl,afsel5),
    b8: PinB8 = (mode8, ot8, pupd8, id8, bs8, br8, afrh,afsel8),
    b9: PinB9 = (mode9, ot9, pupd9, id9, bs9, br9, afrh,afsel9),
    b13: PinB13 = (mode13, ot13, pupd13, id13, bs13, br13, afrh,afsel13),
]);
impl_gpio!(GpioC, GPIOC_S, [
    c0: PinC0 = (mode0, ot0, pupd0, id0, bs0, br0, afrl,afsel0),
    c1: PinC1 = (mode1, ot1, pupd1, id1, bs1, br1, afrl,afsel1),
    c2: PinC2 = (mode2, ot2, pupd2, id2, bs2, br2, afrl,afsel2),
    c3: PinC3 = (mode3, ot3, pupd3, id3, bs3, br3, afrl,afsel3),
    c4: PinC4 = (mode4, ot4, pupd4, id4, bs4, br4, afrl,afsel4),
    c5: PinC5 = (mode5, ot5, pupd5, id5, bs5, br5, afrl,afsel5),
    c6: PinC6 = (mode6, ot6, pupd6, id6, bs6, br6, afrl,afsel6),
    c7: PinC7 = (mode7, ot7, pupd7, id7, bs7, br7, afrl,afsel7),
    c13: PinC13 = (mode13, ot13, pupd13, id13, bs13, br13, afrh,afsel13),
]);
impl_gpio!(GpioD, GPIOD_S, [
    d0: PinD0 = (mode0, ot0, pupd0, id0, bs0, br0, afrl,afsel0),
    d2: PinD2 = (mode2, ot2, pupd2, id2, bs2, br2, afrl,afsel2),
    d5: PinD5 = (mode5, ot5, pupd5, id5, bs5, br5, afrl,afsel5),
]);
impl_gpio!(GpioE, GPIOE_S, [
    e0: PinE0 = (mode0, ot0, pupd0, id0, bs0, br0, afrl,afsel0),
    e4: PinE4 = (mode4, ot4, pupd4, id4, bs4, br4, afrl,afsel4),
]);
impl_gpio!(GpioG, GPIOG_S, [
    g0: PinG0 = (mode0, ot0, pupd0, id0, bs0, br0, afrl,afsel0),
    g8: PinG8 = (mode8, ot8, pupd8, id8, bs8, br8, afrh,afsel8),
    g10: PinG10 = (mode10, ot10, pupd10, id10, bs10, br10, afrh,afsel10),
]);
impl_gpio!(GpioH, GPIOH_S, [
    h0: PinH0 = (mode0, ot0, pupd0, id0, bs0, br0, afrl,afsel0),
    h9: PinH9 = (mode9, ot9, pupd9, id9, bs9, br9, afrh,afsel9),
]);

pub const GPIO_ALTERNATE_FUNCTION_0: u8 = 0x0000000;
pub const GPIO_ALTERNATE_FUNCTION_1: u8 = 0x0000001;
pub const GPIO_ALTERNATE_FUNCTION_2: u8 = 0x0000002;
pub const GPIO_ALTERNATE_FUNCTION_3: u8 = 0x0000003;
pub const GPIO_ALTERNATE_FUNCTION_4: u8 = 0x0000004;
pub const GPIO_ALTERNATE_FUNCTION_5: u8 = 0x0000005;
pub const GPIO_ALTERNATE_FUNCTION_6: u8 = 0x0000006;
pub const GPIO_ALTERNATE_FUNCTION_7: u8 = 0x0000007;
pub const GPIO_ALTERNATE_FUNCTION_8: u8 = 0x0000008;
pub const GPIO_ALTERNATE_FUNCTION_9: u8 = 0x0000009;
pub const GPIO_AFLTERNATE_FUNCTION10: u8 = 0x000000A;
pub const GPIO_AFLTERNATE_FUNCTION11: u8 = 0x000000B;
pub const GPIO_AFLTERNATE_FUNCTION12: u8 = 0x000000C;
pub const GPIO_AFLTERNATE_FUNCTION13: u8 = 0x000000D;
pub const GPIO_AFLTERNATE_FUNCTION14: u8 = 0x000000E;
pub const GPIO_AFLTERNATE_FUNCTION15: u8 = 0x000000F;
