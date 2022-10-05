use std::{convert::Infallible, time::Duration};

use drv8825::{SignalError, DRV8825};
use embedded_hal::timer::{CountDown, Periodic};
use embedded_svc::timer::TimerService;
use esp_idf_svc::timer::{EspTimerService, Task};
use esp_idf_sys::EspError;
use void::Void;


#[derive(Debug, Clone)]
struct CurtainError;

impl CurtainError {
    // type DriverType = Driver;
    fn from_signal(s: SignalError<Infallible,EspError, Infallible>) -> CurtainError {
        CurtainError{}
    }
    // fn from_error(e: Error<Infallible,EspError,Infallible>) {
        
    // }
}

struct CBTimer{
    service: EspTimerService<Task>
}

impl CountDown for CBTimer{
    type Time = Duration;

    fn start<T>(&mut self, count: T)
    where
        T: Into<Self::Time> {
        // self.service.timer(callback)
        todo!()
    }

    fn wait(&mut self) -> nb::Result<(), Void> {
        todo!()
    }
}

impl fugit_timer::Timer for CBTimer {
    type Error = CurtainError;

    fn now(&mut self) -> fugit::TimerInstantU32<TIMER_HZ> {
        todo!()
    }

    fn start(&mut self, duration: fugit::TimerDurationU32<TIMER_HZ>) -> Result<(), Self::Error> {
        todo!()
    }

    fn cancel(&mut self) -> Result<(), Self::Error> {
        todo!()
    }

    fn wait(&mut self) -> nb::Result<(), Self::Error> {
        todo!()
    }
}


mod curtain_stepper {

    use embedded_svc::timer::TimerService;
    use esp_idf_svc::timer::{EspTimerService, EspTimer};
    use fugit_timer::Timer;
    use stepper::{
        fugit::NanosDurationU32 as Nanoseconds,
        motion_control, ramp_maker,
        Direction, Stepper
    };

    
    use esp_idf_hal::gpio::{Gpio1,Gpio2,Gpio3,Gpio4,Gpio5,Output};

    use crate::curtain_stepper::CBTimer;

    use super::CurtainError;
    
    // use embedded_svc::timer::Timer;
    
    pub fn stepper_driver(step: &mut Gpio2<Output>, dir: &mut Gpio3<Output>)->Result<(),CurtainError> {

        // We need some `embedded_hal::digital::OutputPin` implementations connected
        // to the STEP and DIR signals of our driver chip. How you acquire those
        // depends on the platform you run on. Here, we'll use a mock implementation
        // for the sake of demonstration.

        
        
        // We also need a timer (that implements `embedded_hal::timer::CountDown`),
        // since there are time-critical aspects to communicating with the driver
        // chip. Again, how you acquire one depends on your target platform, and
        // again, we'll use a mock here for the sake of demonstration.
        // let mut timer = Timer::<1_000_000>::new();
        let service = EspTimerService::new().unwrap();
        let task = service.timer(|| {
            println!("One-shot timer triggered");
        }).unwrap();
        
        let mut once_timer = EspTimerService::new().unwrap().timer(|| {
            println!("One-shot timer triggered");
        }).unwrap();
    
        
        // embedded_hal::timer::CountDown
        // let mut timer = Timer::<1_000_000>::new();
        let mut timer = CBTimer{service};
        // let mut timer = <dyn Timer::<1_000_000,Error=CurtainError>>::new();
        
        // Define the numeric type we're going to use. We'll use a fixed-point type
        // here, as that's the most widely supported. If your target hardware has
        // support for floating point, it might be more convenient (and possibly
        // efficient) to use that instead.
        type Num = fixed::FixedI64<typenum::U32>;
        
        // Define the target acceleration and maximum speed using timer ticks as the
        // unit of time. We could also use seconds or any other unit of time
        // (Stepper doesn't care), but then we'd need to provide a conversion from
        // seconds to timer ticks. This way, we save that conversion.
        //
        // These values assume a 1 MHz timer, but that depends on the timer you're
        // using, of course.
        let target_accel = Num::from_num(0.001); // steps / tick^2; 1000 steps / s^2
        let max_speed = Num::from_num(0.001); // steps / tick; 1000 steps / s
        
        // We want to use the high-level motion control API (see below), but let's
        // assume the driver we use for this example doesn't provide hardware
        // support for that. Let's instantiate a motion profile from the RampMaker
        // library to provide a software fallback.
        let profile = ramp_maker::Trapezoidal::new(target_accel);
        
        // Now we need to initialize the stepper API. We do this by initializing a
        // driver (`MyDriver`), then wrapping that into the generic API (`Stepper`).
        // `MyDriver` is a placeholder. In a real use-case, you'd typically use one
        // of the drivers from the `stepper::drivers` module, but any driver that
        // implements the traits from `stepper::traits` will do.
        //
        // By default, drivers can't do anything after being initialized. This means
        // they also don't require any hardware resources, which makes them easier
        // to use when you don't need all features.
        use stepper::drivers::drv8825::DRV8825;
        let driver = DRV8825::new();
        let mut stepper = Stepper::from_driver(driver)
            // Enable direction control
            .enable_direction_control(dir, Direction::Forward, &mut timer)
            .map_err(|x| CurtainError::from_signal(x))?
            // Enable step control
            .enable_step_control(step)
            // Enable motion control using the software fallback
            .enable_motion_control((timer, profile, DelayToTicks));
        
        // Tell the motor to move 2000 steps (10 revolutions on a typical stepper
        // motor), while respecting the maximum speed. Since we selected a
        // trapezoidal motion profile above, this will result in a controlled
        // acceleration to the maximum speed, and a controlled deceleration after.
        let target_step = 2000;
        stepper
            .move_to_position(max_speed, target_step)
            
            .wait()
            .unwrap()
            
            ;
        
        // Here's the converter that Stepper is going to use internally, to convert
        // from the computed delay value to timer ticks. Since we chose to use timer
        // ticks as the unit of time for velocity and acceleration, this conversion
        // is pretty simple (and cheap).
        use num_traits::cast::ToPrimitive;
        pub struct DelayToTicks;
        impl<const TIMER_HZ: u32> motion_control::DelayToTicks<Num, TIMER_HZ> for DelayToTicks {
            type Error = core::convert::Infallible;
        
            fn delay_to_ticks(&self, delay: Num)
                -> Result<fugit::TimerDurationU32<TIMER_HZ>, Self::Error>
            {
                Ok(fugit::TimerDurationU32::<TIMER_HZ>::from_ticks(Num::to_u32(&delay).expect("the delay to convert")))
            }
        }
        Ok(())
       
    }


}



