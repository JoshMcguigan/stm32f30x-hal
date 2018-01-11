//! General Purpose Input / Output

// TODO the pins here currently correspond to the LQFP-100 package. There should be Cargo features
// that let you select different microcontroller packages

use core::marker::PhantomData;

use rcc::AHB;

/// Extension trait to split a GPIO peripheral in independent pins
pub trait GpioExt {
    /// GPIO pins
    type Parts;

    /// Splits the GPIO block into independent pins
    fn split(self, ahb: &mut AHB) -> Self::Parts;
}

// States
/// Input mode
pub struct Input<MODE> {
    _mode: PhantomData<MODE>,
}

/// Floating input
pub struct Floating;
/// Pulled down input
pub struct PullDown;
/// Pulled up input
pub struct PullUp;

/// Output mode
pub struct Output<MODE> {
    _mode: PhantomData<MODE>,
}

/// Push pull output
pub struct PushPull;
/// Open drain output
pub struct OpenDrain;

/// Alternate function
pub struct AF0;

/// Alternate function
pub struct AF1;

/// Alternate function
pub struct AF2;

/// Alternate function
pub struct AF3;

/// Alternate function
pub struct AF4;

/// Alternate function
pub struct AF5;

/// Alternate function
pub struct AF6;

/// Alternate function
pub struct AF7;

/// Alternate function
pub struct AF8;

/// Alternate function
pub struct AF9;

/// Alternate function
pub struct AF10;

/// Alternate function
pub struct AF11;

/// Alternate function
pub struct AF12;

/// Alternate function
pub struct AF13;

/// Alternate function
pub struct AF14;

/// Alternate function
pub struct AF15;

macro_rules! gpio {
    ($GPIOX:ident, $gpiox:ident, $iopxenr:ident, $iopxrst:ident, $PXx:ident, [
        $($PXi:ident: ($i:expr, $MODE:ty, $AFR:ident),)+
    ]) => {
        /// GPIO
        #[allow(non_snake_case)]
        pub mod $GPIOX {
            use core::marker::PhantomData;

            use hal::digital::OutputPin;
            use stm32f30x::{$gpiox, $GPIOX};

            use rcc::AHB;
            use super::{
                AF4, AF5, AF6, AF7, Floating, GpioExt, Input, OpenDrain, Output,
                PullDown, PullUp, PushPull,
            };

            /// GPIO parts
            #[allow(non_snake_case)]
            pub struct Parts {
                /// Opaque AFRH register
                pub AFRH: AFRH,
                /// Opaque AFRL register
                pub AFRL: AFRL,
                /// Opaque MODER register
                pub MODER: MODER,
                /// Opaque OTYPER register
                pub OTYPER: OTYPER,
                /// Opaque PUPDR register
                pub PUPDR: PUPDR,
                $(
                    /// Pin
                    pub $PXi: $PXi<$MODE>,
                )+
            }

            impl GpioExt for $GPIOX {
                type Parts = Parts;

                fn split(self, ahb: &mut AHB) -> Parts {
                    ahb.enr().modify(|_, w| w.$iopxenr().enabled());
                    ahb.rstr().modify(|_, w| w.$iopxrst().set_bit());
                    ahb.rstr().modify(|_, w| w.$iopxrst().clear_bit());

                    Parts {
                        AFRH: AFRH { _0: () },
                        AFRL: AFRL { _0: () },
                        MODER: MODER { _0: () },
                        OTYPER: OTYPER { _0: () },
                        PUPDR: PUPDR { _0: () },
                        $(
                            $PXi: $PXi { _mode: PhantomData },
                        )+
                    }
                }
            }

            /// Opaque AFRL register
            pub struct AFRL {
                _0: (),
            }

            impl AFRL {
                pub(crate) fn afr(&mut self) -> &$gpiox::AFRL {
                    unsafe { &(*$GPIOX::ptr()).afrl }
                }
            }

            /// Opaque AFRH register
            pub struct AFRH {
                _0: (),
            }

            impl AFRH {
                pub(crate) fn afr(&mut self) -> &$gpiox::AFRH {
                    unsafe { &(*$GPIOX::ptr()).afrh }
                }
            }

            /// Opaque MODER register
            pub struct MODER {
                _0: (),
            }

            impl MODER {
                pub(crate) fn moder(&mut self) -> &$gpiox::MODER {
                    unsafe { &(*$GPIOX::ptr()).moder }
                }
            }

            /// Opaque OTYPER register
            pub struct OTYPER {
                _0: (),
            }

            impl OTYPER {
                pub(crate) fn otyper(&mut self) -> &$gpiox::OTYPER {
                    unsafe { &(*$GPIOX::ptr()).otyper }
                }
            }

            /// Opaque PUPDR register
            pub struct PUPDR {
                _0: (),
            }

            impl PUPDR {
                pub(crate) fn pupdr(&mut self) -> &$gpiox::PUPDR {
                    unsafe { &(*$GPIOX::ptr()).pupdr }
                }
            }

            /// Partially erased pin
            pub struct $PXx<MODE> {
                i: u8,
                _mode: PhantomData<MODE>,
            }

            impl<MODE> OutputPin for $PXx<Output<MODE>> {
                fn is_high(&self) -> bool {
                    !self.is_low()
                }

                fn is_low(&self) -> bool {
                    // NOTE(unsafe) atomic read with no side effects
                    unsafe { (*$GPIOX::ptr()).odr.read().bits() & (1 << self.i) == 0 }
                }

                fn set_high(&mut self) {
                    // NOTE(unsafe) atomic write to a stateless register
                    unsafe { (*$GPIOX::ptr()).bsrr.write(|w| w.bits(1 << self.i)) }
                }

                fn set_low(&mut self) {
                    // NOTE(unsafe) atomic write to a stateless register
                    unsafe { (*$GPIOX::ptr()).bsrr.write(|w| w.bits(1 << (16 + self.i))) }
                }
            }

            $(
                /// Pin
                pub struct $PXi<MODE> {
                    _mode: PhantomData<MODE>,
                }

                impl<MODE> $PXi<MODE> {
                    /// Puts the pin in alternate function 4 (AF4)
                    pub fn as_af4(
                        self,
                        moder: &mut MODER,
                        afr: &mut $AFR,
                    ) -> $PXi<AF4> {
                        let offset = 2 * $i;

                        // alternate function mode
                        let mode = 0b10;
                        moder.moder().modify(|r, w| unsafe {
                            w.bits((r.bits() & !(0b11 << offset)) | (mode << offset))
                        });

                        let af = 4;
                        let offset = 4 * ($i % 8);
                        afr.afr().modify(|r, w| unsafe {
                            w.bits((r.bits() & !(0b1111 << offset)) | (af << offset))
                        });

                        $PXi { _mode: PhantomData }
                    }

                    /// Puts the pin in alternate function 5 (AF5)
                    pub fn as_af5(
                        self,
                        moder: &mut MODER,
                        afr: &mut $AFR,
                    ) -> $PXi<AF5> {
                        let offset = 2 * $i;

                        // alternate function mode
                        let mode = 0b10;
                        moder.moder().modify(|r, w| unsafe {
                            w.bits((r.bits() & !(0b11 << offset)) | (mode << offset))
                        });

                        let af = 5;
                        let offset = 4 * ($i % 8);
                        afr.afr().modify(|r, w| unsafe {
                            w.bits((r.bits() & !(0b1111 << offset)) | (af << offset))
                        });

                        $PXi { _mode: PhantomData }
                    }

                    /// Puts the pin in alternate function 6 (AF6)
                    pub fn as_af6(
                        self,
                        moder: &mut MODER,
                        afr: &mut $AFR,
                    ) -> $PXi<AF6> {
                        let offset = 2 * $i;

                        // alternate function mode
                        let mode = 0b10;
                        moder.moder().modify(|r, w| unsafe {
                            w.bits((r.bits() & !(0b11 << offset)) | (mode << offset))
                        });

                        let af = 6;
                        let offset = 4 * ($i % 8);
                        afr.afr().modify(|r, w| unsafe {
                            w.bits((r.bits() & !(0b1111 << offset)) | (af << offset))
                        });

                        $PXi { _mode: PhantomData }
                    }

                    /// Puts the pin in alternate function 7 (AF7)
                    pub fn as_af7(
                        self,
                        moder: &mut MODER,
                        afr: &mut $AFR,
                    ) -> $PXi<AF7> {
                        let offset = 2 * $i;

                        // alternate function mode
                        let mode = 0b10;
                        moder.moder().modify(|r, w| unsafe {
                            w.bits((r.bits() & !(0b11 << offset)) | (mode << offset))
                        });

                        let af = 7;
                        let offset = 4 * ($i % 8);

                        afr.afr().modify(|r, w| unsafe {
                            w.bits((r.bits() & !(0b1111 << offset)) | (af << offset))
                        });

                        $PXi { _mode: PhantomData }
                    }

                    /// Puts the pin in floating input mode
                    pub fn as_floating_input(
                        self,
                        moder: &mut MODER,
                        pupdr: &mut PUPDR,
                    ) -> $PXi<Input<Floating>> {
                        let offset = 2 * $i;

                        // input mode
                        moder
                            .moder()
                            .modify(|r, w| unsafe { w.bits(r.bits() & !(0b11 << offset)) });

                        // no pull-up or pull-down
                        pupdr
                            .pupdr()
                            .modify(|r, w| unsafe { w.bits(r.bits() & !(0b11 << offset)) });

                        $PXi { _mode: PhantomData }
                    }

                    /// Puts the pin in pull down input mode
                    pub fn as_pull_down_input(
                        self,
                        moder: &mut MODER,
                        pupdr: &mut PUPDR,
                    ) -> $PXi<Input<PullDown>> {
                        let offset = 2 * $i;

                        // input mode
                        moder
                            .moder()
                            .modify(|r, w| unsafe { w.bits(r.bits() & !(0b11 << offset)) });

                        // pull-down
                        pupdr.pupdr().modify(|r, w| unsafe {
                            w.bits((r.bits() & !(0b11 << offset)) | (0b10 << offset))
                        });

                        $PXi { _mode: PhantomData }
                    }

                    /// Puts the pin in pull up input mode
                    pub fn as_pull_up_input(
                        self,
                        moder: &mut MODER,
                        pupdr: &mut PUPDR,
                    ) -> $PXi<Input<PullUp>> {
                        let offset = 2 * $i;

                        // input mode
                        moder
                            .moder()
                            .modify(|r, w| unsafe { w.bits(r.bits() & !(0b11 << offset)) });

                        // pull-up
                        pupdr.pupdr().modify(|r, w| unsafe {
                            w.bits((r.bits() & !(0b11 << offset)) | (0b01 << offset))
                        });

                        $PXi { _mode: PhantomData }
                    }

                    /// Puts the pin in open drain output mode
                    pub fn as_open_drain_output(
                        self,
                        moder: &mut MODER,
                        otyper: &mut OTYPER,
                    ) -> $PXi<Output<OpenDrain>> {
                        let offset = 2 * $i;

                        // general purpose output mode
                        let mode = 0b01;
                        moder.moder().modify(|r, w| unsafe {
                            w.bits((r.bits() & !(0b11 << offset)) | (mode << offset))
                        });

                        // open drain output
                        otyper
                            .otyper()
                            .modify(|r, w| unsafe { w.bits(r.bits() | (0b1 << $i)) });

                        $PXi { _mode: PhantomData }
                    }

                    /// Puts the pin in push pull output mode
                    pub fn as_push_pull_output(
                        self,
                        moder: &mut MODER,
                        otyper: &mut OTYPER,
                    ) -> $PXi<Output<PushPull>> {
                        let offset = 2 * $i;

                        // general purpose output mode
                        let mode = 0b01;
                        moder.moder().modify(|r, w| unsafe {
                            w.bits((r.bits() & !(0b11 << offset)) | (mode << offset))
                        });

                        // push pull output
                        otyper
                            .otyper()
                            .modify(|r, w| unsafe { w.bits(r.bits() & !(0b1 << $i)) });

                        $PXi { _mode: PhantomData }
                    }
                }

                impl $PXi<Output<OpenDrain>> {
                    /// Enables / disables the internal pull up
                    pub fn internal_pull_up(&mut self, pupdr: &mut PUPDR, on: bool) {
                        let offset = 2 * $i;

                        pupdr.pupdr().modify(|r, w| unsafe {
                            w.bits(
                                (r.bits() & !(0b11 << offset)) | if on {
                                    0b01 << offset
                                } else {
                                    0
                                },
                            )
                        });
                    }
                }

                impl<MODE> $PXi<Output<MODE>> {
                    /// Erases the pin number from the type
                    ///
                    /// This is useful when you want to collect the pins into an array where you
                    /// need all the elements to have the same type
                    pub fn downgrade(self) -> $PXx<Output<MODE>> {
                        $PXx {
                            i: $i,
                            _mode: self._mode,
                        }
                    }
                }

                impl<MODE> OutputPin for $PXi<Output<MODE>> {
                    fn is_high(&self) -> bool {
                        !self.is_low()
                    }

                    fn is_low(&self) -> bool {
                        // NOTE(unsafe) atomic read with no side effects
                        unsafe { (*$GPIOX::ptr()).odr.read().bits() & (1 << $i) == 0 }
                    }

                    fn set_high(&mut self) {
                        // NOTE(unsafe) atomic write to a stateless register
                        unsafe { (*$GPIOX::ptr()).bsrr.write(|w| w.bits(1 << $i)) }
                    }

                    fn set_low(&mut self) {
                        // NOTE(unsafe) atomic write to a stateless register
                        unsafe { (*$GPIOX::ptr()).bsrr.write(|w| w.bits(1 << (16 + $i))) }
                    }
                }
            )+
        }
    }
}

gpio!(GPIOA, gpioa, iopaen, ioparst, PAx, [
    PA0: (0, Input<Floating>, AFRL),
    PA1: (1, Input<Floating>, AFRL),
    PA2: (2, Input<Floating>, AFRL),
    PA3: (3, Input<Floating>, AFRL),
    PA4: (4, Input<Floating>, AFRL),
    PA5: (5, Input<Floating>, AFRL),
    PA6: (6, Input<Floating>, AFRL),
    PA7: (7, Input<Floating>, AFRL),
    PA8: (8, Input<Floating>, AFRH),
    PA9: (9, Input<Floating>, AFRH),
    PA10: (10, Input<Floating>, AFRH),
    PA11: (11, Input<Floating>, AFRH),
    PA12: (12, Input<Floating>, AFRH),
    // TODO these are configured as JTAG pins
    // PA13: (13, Input<Floating>),
    // PA14: (14, Input<Floating>),
    // PA15: (15, Input<Floating>),
]);

gpio!(GPIOB, gpiob, iopben, iopbrst, PBx, [
    PB0: (0, Input<Floating>, AFRL),
    PB1: (1, Input<Floating>, AFRL),
    PB2: (2, Input<Floating>, AFRL),
    // TODO these are configured as JTAG pins
    // PB3: (3, Input<Floating>),
    // PB4: (4, Input<Floating>),
    PB5: (5, Input<Floating>, AFRL),
    PB6: (6, Input<Floating>, AFRL),
    PB7: (7, Input<Floating>, AFRL),
    PB8: (8, Input<Floating>, AFRH),
    PB9: (9, Input<Floating>, AFRH),
    PB10: (10, Input<Floating>, AFRH),
    PB11: (11, Input<Floating>, AFRH),
    PB12: (12, Input<Floating>, AFRH),
    PB13: (13, Input<Floating>, AFRH),
    PB14: (14, Input<Floating>, AFRH),
    PB15: (15, Input<Floating>, AFRH),
]);

gpio!(GPIOC, gpioc, iopcen, iopcrst, PCx, [
    PC0: (0, Input<Floating>, AFRL),
    PC1: (1, Input<Floating>, AFRL),
    PC2: (2, Input<Floating>, AFRL),
    PC3: (3, Input<Floating>, AFRL),
    PC4: (4, Input<Floating>, AFRL),
    PC5: (5, Input<Floating>, AFRL),
    PC6: (6, Input<Floating>, AFRL),
    PC7: (7, Input<Floating>, AFRL),
    PC8: (8, Input<Floating>, AFRH),
    PC9: (9, Input<Floating>, AFRH),
    PC10: (10, Input<Floating>, AFRH),
    PC11: (11, Input<Floating>, AFRH),
    PC12: (12, Input<Floating>, AFRH),
    PC13: (13, Input<Floating>, AFRH),
    PC14: (14, Input<Floating>, AFRH),
    PC15: (15, Input<Floating>, AFRH),
]);

gpio!(GPIOD, gpioc, iopden, iopdrst, PDx, [
    PD0: (0, Input<Floating>, AFRL),
    PD1: (1, Input<Floating>, AFRL),
    PD2: (2, Input<Floating>, AFRL),
    PD3: (3, Input<Floating>, AFRL),
    PD4: (4, Input<Floating>, AFRL),
    PD5: (5, Input<Floating>, AFRL),
    PD6: (6, Input<Floating>, AFRL),
    PD7: (7, Input<Floating>, AFRL),
    PD8: (8, Input<Floating>, AFRH),
    PD9: (9, Input<Floating>, AFRH),
    PD10: (10, Input<Floating>, AFRH),
    PD11: (11, Input<Floating>, AFRH),
    PD12: (12, Input<Floating>, AFRH),
    PD13: (13, Input<Floating>, AFRH),
    PD14: (14, Input<Floating>, AFRH),
    PD15: (15, Input<Floating>, AFRH),
]);

gpio!(GPIOE, gpioc, iopeen, ioperst, PEx, [
    PE0: (0, Input<Floating>, AFRL),
    PE1: (1, Input<Floating>, AFRL),
    PE2: (2, Input<Floating>, AFRL),
    PE3: (3, Input<Floating>, AFRL),
    PE4: (4, Input<Floating>, AFRL),
    PE5: (5, Input<Floating>, AFRL),
    PE6: (6, Input<Floating>, AFRL),
    PE7: (7, Input<Floating>, AFRL),
    PE8: (8, Input<Floating>, AFRH),
    PE9: (9, Input<Floating>, AFRH),
    PE10: (10, Input<Floating>, AFRH),
    PE11: (11, Input<Floating>, AFRH),
    PE12: (12, Input<Floating>, AFRH),
    PE13: (13, Input<Floating>, AFRH),
    PE14: (14, Input<Floating>, AFRH),
    PE15: (15, Input<Floating>, AFRH),
]);

gpio!(GPIOF, gpioc, iopfen, iopfrst, PFx, [
    PF0: (0, Input<Floating>, AFRL),
    PF1: (1, Input<Floating>, AFRL),
    PF2: (2, Input<Floating>, AFRL),
    PF4: (4, Input<Floating>, AFRL),
    PF6: (6, Input<Floating>, AFRL),
    PF9: (9, Input<Floating>, AFRH),
    PF10: (10, Input<Floating>, AFRH),
]);
