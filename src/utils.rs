pub trait AsStatic<T> {
    fn as_static(&self) -> &'static T;
    fn as_static_mut(&mut self) -> &'static mut T;
}

impl<T> AsStatic<T> for T {
    fn as_static(&self) -> &'static T {
        unsafe { &*(self as *const T) }
    }
    fn as_static_mut(&mut self) -> &'static mut T {
        unsafe { &mut *(self as *mut T) }
    }
}
