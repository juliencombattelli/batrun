use std::collections::VecDeque;
use std::future::Future;
use std::pin::Pin;
use std::ptr;
use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

pub fn wait_until_next_poll() -> impl Future<Output = ()> {
    WaitUntilNextPoll {
        already_polled: false,
    }
}

struct WaitUntilNextPoll {
    already_polled: bool,
}

impl Future for WaitUntilNextPoll {
    type Output = ();

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        _: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        if self.already_polled {
            std::task::Poll::Ready(())
        } else {
            self.already_polled = true;
            std::task::Poll::Pending
        }
    }
}

pub fn execute_many<'a>(fs: Vec<Pin<Box<dyn Future<Output = ()> + 'a>>>) {
    let waker = noop_waker();
    let mut ctx = Context::from_waker(&waker);

    let mut queue = VecDeque::from(fs);
    while let Some(mut f) = queue.pop_front() {
        let f2 = f.as_mut();
        if f2.poll(&mut ctx).is_pending() {
            // Enqueue back, so it gets polled again in a future iteration (since we don't support
            // IO or waiting in any form, the underlying future will make progress every time we
            // poll)
            queue.push_back(f);
        }
    }
}

fn noop_waker() -> Waker {
    const NOOP: RawWaker = {
        const VTABLE: RawWakerVTable = RawWakerVTable::new(
            // Cloning just returns a new no-op raw waker
            |_| NOOP,
            // `wake` does nothing
            |_| {},
            // `wake_by_ref` does nothing
            |_| {},
            // Dropping does nothing as we don't allocate anything
            |_| {},
        );
        RawWaker::new(ptr::null(), &VTABLE)
    };

    unsafe { Waker::from_raw(NOOP) }
}

// async fn print_squares1_async(i: u64) {
//     println!("1. sqr({i}) = {}", i * i);
//     wait_until_next_poll().await;
// }

// async fn print_squares2_async(i: u64) {
//     println!("2. sqr({i}) = {}", i * i);
//     wait_until_next_poll().await;
// }

// fn main() {
//     let f1 = async || {
//         for i in 1..10 {
//             print_squares1_async(i).await;
//         }
//     };
//     let f2 = async || {
//         for i in 1..3 {
//             print_squares2_async(i).await;
//         }
//     };
//     execute_many(vec![Box::pin(f1()), Box::pin(f2())]);
// }
