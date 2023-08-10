#[rustversion::since(1.39)]
#[multiversion::multiversion(targets("x86_64+avx", "x86_64+sse", "aarch64+neon",))]
async fn async_add(a: &mut [f32], b: &[f32]) {
    a.iter_mut().zip(b.iter()).for_each(|(a, b)| *a += b);
}

mod test {

    // Adapted from David Tolnay's async-trait.
    // Provided under Apache License, Version 2.0 or MIT license.
    #[rustversion::since(1.39)]
    pub fn block_on<F: std::future::Future>(mut fut: F) -> F::Output {
        use std::pin::Pin;
        use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

        unsafe fn clone(_null: *const ()) -> RawWaker {
            unimplemented!()
        }

        unsafe fn wake(_null: *const ()) {
            unimplemented!()
        }

        unsafe fn wake_by_ref(_null: *const ()) {
            unimplemented!()
        }

        unsafe fn drop(_null: *const ()) {}

        let data = std::ptr::null();
        let vtable = &RawWakerVTable::new(clone, wake, wake_by_ref, drop);
        let raw_waker = RawWaker::new(data, vtable);
        let waker = unsafe { Waker::from_raw(raw_waker) };
        let mut cx = Context::from_waker(&waker);

        // fut does not move until it gets dropped.
        let fut = unsafe { Pin::new_unchecked(&mut fut) };

        match fut.poll(&mut cx) {
            Poll::Ready(output) => output,
            Poll::Pending => panic!("future did not resolve immediately"),
        }
    }

    #[rustversion::since(1.39)]
    #[test]
    fn async_fn() {
        let mut a = vec![0f32, 2f32, 4f32];
        let b = vec![1f32, 1f32, 1f32];
        let fut = super::async_add(&mut a, &b);
        block_on(fut);
        assert_eq!(a, vec![1f32, 3f32, 5f32]);
    }
}
