//! Panic handling abstracted to work with and without `#[cfg(feature = "std")]`

#[cfg(feature = "std")]
use std::boxed::Box;

#[cfg(feature = "std")]
type PanicPayloadInner = Box<dyn core::any::Any + Send + 'static>;
#[cfg(not(feature = "std"))]
type PanicPayloadInner = core::convert::Infallible;

/// A panic payload, abstracted to work with and without
/// `#[cfg(feature = "std")]`.
///
/// With `std`, this will contain a normal panic payload
/// (`Box<dyn Any + Send + 'static>`).
///
/// Without `std`, this will never be constructed, and contains
/// `std::convert::Infallible`.
pub struct PanicPayload(PanicPayloadInner);

/// [`std::panic::catch_unwind`], abstracted to work with and without `std`.
///
/// With `std`, this function just delegates to [`std::panic::catch_unwind`].
///
/// Without `std`, this function will call the provided closure without
/// attempting to catch panics at all - it will therefore always either return
/// [`Ok`] or diverge.
///
/// Note that this function additionally does not require the closure is
/// [`UnwindSafe`](core::panic::UnwindSafe) - our usage within this crate would
/// be wrapping all calls in [`AssertUnwindSafe`](core::panic::AssertUnwindSafe)
/// anyway. It would be difficult for an API consumer to observe violated
/// invariants through unwind unsafety, and the API burden on normal use cases
/// would be too heavy if we didn't assert unwind safety on their behalf.
pub fn catch_unwind<F: FnOnce() -> R, R>(f: F) -> Result<R, PanicPayload> {
    // If we have `std`, delegate to `catch_unwind`
    #[cfg(feature = "std")]
    let output = std::panic::catch_unwind(core::panic::AssertUnwindSafe(f)).map_err(PanicPayload);

    // If we don't have `std`, just call the function and let panics go uncaught
    #[cfg(not(feature = "std"))]
    let output = Ok(f());

    output
}

/// [`std::panic::resume_unwind`], abstracted to work with and without `std`.
///
/// With `std`, this function just delegates to [`std::panic::resume_unwind`].
///
/// Without `std`, this function is impossible to call - a [`PanicPayload`] is
/// never produced by [`catch_unwind`] without `std`.
pub fn resume_unwind(payload: PanicPayload) -> ! {
    // If we have `std`, delegate to `resume_unwind`
    #[cfg(feature = "std")]
    std::panic::resume_unwind(payload.0);

    // If we don't have `std`, a PanicPayload can never be produced, so this
    // function can't be called in the first place
    #[cfg(not(feature = "std"))]
    match payload {}
}
