#[rustversion::since(1.39)]
#[multiversion::multiversion]
#[clone(target = "[x86|x86_64]+avx")]
#[clone(target = "[x86|x86_64]+sse")]
#[clone(target = "[arm|aarch64]+neon")]
async fn async_add(a: &mut [f32], b: &[f32]) {
    a.iter_mut().zip(b.iter()).for_each(|(a, b)| *a += b);
}

#[rustversion::since(1.39)]
struct Adder(f32);

#[rustversion::since(1.39)]
impl Adder {
    #[multiversion::multiversion]
    #[clone(target = "[x86|x86_64]+avx")]
    #[clone(target = "[x86|x86_64]+sse")]
    #[clone(target = "[arm|aarch64]+neon")]
    async fn async_add(&self, a: &mut [f32]) {
        a.iter_mut().for_each(|a| *a += self.0);
    }
}

#[rustversion::since(1.39)]
mod test {
    // Adapted from David Tolnay's async-trait.
    // Provided under Apache License, Version 2.0 or MIT license.
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

    #[test]
    fn async_fn() {
        let mut a = vec![0f32, 2f32, 4f32];
        let b = vec![1f32, 1f32, 1f32];
        let fut = super::async_add(&mut a, &b);
        block_on(fut);
        assert_eq!(a, vec![1f32, 3f32, 5f32]);
    }

    #[test]
    fn async_associated_fn() {
        let mut a = vec![0f32, 2f32, 4f32];
        let fut = super::Adder(1.).async_add(&mut a);
        block_on(fut);
        assert_eq!(a, vec![1f32, 3f32, 5f32]);
    }
}
