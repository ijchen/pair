/// A simple struct that runs a closure on drop. Used to clean up resources
/// during panic unwinding within [`Pair`](crate::pair).
pub struct DropGuard<F: FnMut()>(pub F);

impl<F: FnMut()> Drop for DropGuard<F> {
    fn drop(&mut self) {
        (self.0)();
    }
}
