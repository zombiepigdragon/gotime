//! test executable for this project

use std::{
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll, Waker},
    thread,
    time::Duration,
};

fn main() {
    // println!("Allocating Boxed i32");
    // let boxed = gotime::GoBox::new(6_i32);
    // println!("Allocated Boxed i32");
    // dbg!(&*boxed);
    // println!("Cloning allocation");
    // let boxed2 = boxed.clone();
    // dbg!(&*boxed, &*boxed2);
    // drop(boxed);
    // println!("Freed Boxed i32");
    // dbg!(&*boxed2);
    // drop(boxed2);
    // println!("Freed boxed2");

    #[derive(Debug)]
    struct MyUselessFuture {
        has_run: bool,
    }
    impl Future for MyUselessFuture {
        type Output = ();

        fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Self::Output> {
            eprintln!("fut: start of poll (should be false)");
            dbg!(&self);
            assert!(!self.has_run);
            eprintln!("fut: post assert (should be false or assert failed)");
            dbg!(&self);
            eprintln!("fut: about to do assignment");
            let this = self.get_mut();
            this.has_run = true;
            eprintln!("fut: post assignment (should be true)");
            dbg!(&this);
            eprintln!("fut: ready");
            Poll::Ready(())
        }
    }

    let fut = MyUselessFuture { has_run: false };
    eprintln!("fut: before block_on (should be false)");
    dbg!(&fut);
    let () = gotime::block_on(fut);

    // let () = gotime::block_on(async {});

    // dbg!(std::time::Instant::now());
    // let now = gotime::block_on(async {
    //     println!("howdy!");
    //     // Wait for our timer future to complete after 0.5 seconds.
    //     // TimerFuture::new(Duration::from_millis(5000)).await;
    //     println!("done!");
    //     std::time::Instant::now()
    // });
    // dbg!(now);
}

pub struct TimerFuture {
    shared_state: Arc<Mutex<SharedState>>,
}

/// Shared state between the future and the waiting thread
struct SharedState {
    /// Whether or not the sleep time has elapsed
    completed: bool,

    /// The waker for the task that `TimerFuture` is running on.
    /// The thread can use this after setting `completed = true` to tell
    /// `TimerFuture`'s task to wake up, see that `completed = true`, and
    /// move forward.
    waker: Option<Waker>,
}

impl Future for TimerFuture {
    type Output = ();
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Look at the shared state to see if the timer has already completed.
        let mut shared_state = self.shared_state.lock().unwrap();
        if shared_state.completed {
            Poll::Ready(())
        } else {
            // Set waker so that the thread can wake up the current task
            // when the timer has completed, ensuring that the future is polled
            // again and sees that `completed = true`.
            //
            // It's tempting to do this once rather than repeatedly cloning
            // the waker each time. However, the `TimerFuture` can move between
            // tasks on the executor, which could cause a stale waker pointing
            // to the wrong task, preventing `TimerFuture` from waking up
            // correctly.
            //
            // N.B. it's possible to check for this using the `Waker::will_wake`
            // function, but we omit that here to keep things simple.
            shared_state.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}

impl TimerFuture {
    /// Create a new `TimerFuture` which will complete after the provided
    /// timeout.
    pub fn new(duration: Duration) -> Self {
        let shared_state = Arc::new(Mutex::new(SharedState {
            completed: false,
            waker: None,
        }));

        // Spawn the new thread
        let thread_shared_state = shared_state.clone();
        thread::spawn(move || {
            thread::sleep(duration);
            let mut shared_state = thread_shared_state.lock().unwrap();
            // Signal that the timer has completed and wake up the last
            // task on which the future was polled, if one exists.
            shared_state.completed = true;
            if let Some(waker) = shared_state.waker.take() {
                waker.wake()
            }
        });

        TimerFuture { shared_state }
    }
}
