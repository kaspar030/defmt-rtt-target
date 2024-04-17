//! `defmt` global logger over RTT using `rtt-target`

#![no_std]

use critical_section::RestoreState;

/// Global logger lock.
static mut TAKEN: bool = false;
static mut CS_RESTORE: RestoreState = RestoreState::invalid();
static mut ENCODER: defmt::Encoder = defmt::Encoder::new();

use rtt_target::UpChannel;

static mut CHANNEL: Option<UpChannel> = None;

#[defmt::global_logger]
struct Logger;

pub fn init(channel: UpChannel) {
    unsafe { CHANNEL = Some(channel) }
}

unsafe impl defmt::Logger for Logger {
    fn acquire() {
        unsafe {
            // safety: Must be paired with corresponding call to release(), see below
            let restore = critical_section::acquire();

            // safety: accessing the `static mut` is OK because we have acquired a critical
            // section.
            if TAKEN {
                panic!("defmt logger taken reentrantly")
            }

            // safety: accessing the `static mut` is OK because we have acquired a critical
            // section.
            TAKEN = true;

            // safety: accessing the `static mut` is OK because we have acquired a critical
            // section.
            CS_RESTORE = restore;
        }

        // safety: accessing the `static mut` is OK because we have disabled interrupts.
        unsafe { ENCODER.start_frame(do_write) }
    }

    unsafe fn flush() {}

    unsafe fn release() {
        ENCODER.end_frame(do_write);

        // safety: accessing the `static mut` is OK because we have acquired a critical
        // section.
        TAKEN = false;

        // safety: accessing the `static mut` is OK because we have acquired a critical
        // section.
        let restore = CS_RESTORE;

        // safety: Must be paired with corresponding call to acquire(), see above
        critical_section::release(restore);
    }

    unsafe fn write(bytes: &[u8]) {
        // safety: accessing the `static mut` is OK because we have disabled interrupts.
        ENCODER.write(bytes, do_write);
    }
}

fn do_write(bytes: &[u8]) {
    unsafe {
        if let Some(c) = &mut CHANNEL {
            c.write(bytes);
        }
    };
}
