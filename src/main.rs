use nonprofit_dlq_worker::domain::{decide, JobDecision, JobKind, NonprofitJob};
use nonprofit_dlq_worker::infrai_client::InfraiClient;

fn block_on<T>(future: impl std::future::Future<Output = T>) -> T {
    use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};
    fn noop(_: *const ()) {}
    fn clone(_: *const ()) -> RawWaker { RawWaker::new(std::ptr::null(), &VTABLE) }
    static VTABLE: RawWakerVTable = RawWakerVTable::new(clone, noop, noop, noop);
    let waker = unsafe { Waker::from_raw(RawWaker::new(std::ptr::null(), &VTABLE)) };
    let mut context = Context::from_waker(&waker);
    let mut future = std::pin::pin!(future);
    loop {
        match future.as_mut().poll(&mut context) {
            Poll::Ready(value) => return value,
            Poll::Pending => std::thread::yield_now(),
        }
    }
}

fn main() {
    let job = NonprofitJob { kind: JobKind::DonorReceipt, recipient: "donor-42".into(), attempts: 3 };
    match decide(&job, 3) {
        JobDecision::DeadLetter { reason } => println!("dead-letter: {}", reason),
        JobDecision::Process => println!("process: {}", job.recipient),
    }
    if let (Ok(client), Ok(queue)) = (InfraiClient::from_env(), std::env::var("INFRAI_QUEUE")) {
        let _ = block_on(client.queue_dlq_list(&queue));
        println!("queue: {}", queue);
    }
}
