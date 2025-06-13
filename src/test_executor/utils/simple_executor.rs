use std::collections::VecDeque;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll, Waker};

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

    fn poll(mut self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Self::Output> {
        if self.already_polled {
            Poll::Ready(())
        } else {
            self.already_polled = true;
            Poll::Pending
        }
    }
}

pub fn execute_many<'a>(fs: Vec<Pin<Box<dyn Future<Output = ()> + 'a>>>) {
    let waker = Waker::noop();
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

// pub fn block_on<F: IntoFuture>(fut: F) -> F::Output {
//     execute_many(vec![Box::pin(task())])
// }
