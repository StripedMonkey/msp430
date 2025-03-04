//! Single-core critical section implementation using the [`critical_section`]
//! crate. Only the [`with`] function is publicly exposed.

#[cfg(all(target_arch = "msp430", feature = "critical-section-single-core"))]
mod critical_section_single_core {
    use crate::{interrupt, register};
    use critical_section::RawRestoreState;

    struct CriticalSection;
    critical_section::set_impl!(CriticalSection);

    unsafe impl critical_section::Impl for CriticalSection {
        // Without #[inline(never)] attribute, rustc tends to misoptimize for
        // size on functions that use interrupt::free() or
        // critical_section::with().
        // I don't believe #[inline] or #[inline(always)] hints work for
        // functions marked as "extern" in another crate (which is the case
        // for the acquire()/release() here- see critical_section crate).
        #[cfg_attr(feature = "outline-cs-acq", inline(never))]
        unsafe fn acquire() -> RawRestoreState {
            // Disable interrupts and make sure we know whether they were
            // enabled or not before entering this function.
            let sr = register::sr::read().bits();
            interrupt::disable();
            // Safety: Sr is repr(transparent), RawRestoreState is u16, and Sr
            // contains only a single u16. This should be fine.
            core::mem::transmute(sr)
        }

        #[cfg_attr(feature = "outline-cs-rel", inline(never))]
        unsafe fn release(sr: RawRestoreState) {
            // Safety: Must be called w/ acquire, otherwise we could receive
            // an invalid Sr (even though internally it's a u16, not all bits
            // are actually used). It would be better to pass Sr as
            // RawRestoreState, but since we can't, this will be acceptable,
            // See acquire() for why this is safe.
            let sr: register::sr::Sr = core::mem::transmute(sr);

            // If the interrupts were active before our `disable` call, then re-enable
            // them. Otherwise, keep them disabled.
            if sr.gie() {
                interrupt::enable();
            }
        }
    }
}

pub use ::critical_section::with;
