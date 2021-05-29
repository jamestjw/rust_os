use crate::{print, println};
use conquer_once::spin::OnceCell;
use core::{
    pin::Pin,
    task::{Context, Poll},
};
use crossbeam_queue::ArrayQueue;
use futures_util::stream::Stream;
use futures_util::stream::StreamExt;
use futures_util::task::AtomicWaker;
use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};

// Use OnceCell to perform safe one-time initialization of
// static values (since heap allocation is not possible at compile time).
//
// We use this over lazy_static since OnceCell has the advantage
// of ensuring that the initialization does not happen in the
// interrupt handler.
static SCANCODE_QUEUE: OnceCell<ArrayQueue<u8>> = OnceCell::uninit();

// Can be safely stored in a static and modifier concurrently
static WAKER: AtomicWaker = AtomicWaker::new();

// pub(crate) to only limit visibility of this function to `lib.rs`
pub(crate) fn add_scancode(scancode: u8) {
    // We take care not to initialise the queue in this function
    // since it is the interrupt handler that calls this function.
    // Heap allocations should not occur while handling interrupts
    if let Ok(queue) = SCANCODE_QUEUE.try_get() {
        if let Err(_) = queue.push(scancode) {
            println!("WARNING: scancode queue full; dropping keyboard input");
        } else {
            // Inform waker that there are available scancodes
            WAKER.wake();
        }
    } else {
        println!("WARNING: scancode queue uninitialised");
    }
}

pub struct ScancodeStream {
    // Prevent construction from outside of the module
    _private: (),
}

impl ScancodeStream {
    pub fn new() -> Self {
        // Ensure that only one ScancodeStream can be created
        SCANCODE_QUEUE
            .try_init_once(|| ArrayQueue::new(100))
            .expect("ScancodeStream::new should only be called once");
        ScancodeStream { _private: () }
    }
}

// To make the scancodes available to async tasks.
impl Stream for ScancodeStream {
    type Item = u8;

    // Allows us to keep calling the method until a None is returned
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<u8>> {
        let queue = SCANCODE_QUEUE.try_get().expect("not initialised");

        if let Ok(scancode) = queue.pop() {
            return Poll::Ready(Some(scancode));
        }

        WAKER.register(&cx.waker());

        // Check again as there might be a value pushed to
        // the queue while we were registering the waker
        match queue.pop() {
            Ok(scancode) => {
                WAKER.take();
                Poll::Ready(Some(scancode))
            }
            Err(crossbeam_queue::PopError) => Poll::Pending,
        }
    }
}

pub async fn print_keypresses() {
    let mut scancodes = ScancodeStream::new();
    // HandleControl allows the mapping of ctrl+[a-z] to Unicode characters,
    // which we do not allow
    let mut keyboard = Keyboard::new(layouts::Us104Key, ScancodeSet1, HandleControl::Ignore);

    // Since `poll_next` of the stream never returns `None`,
    // this is effectively an endless loop.
    while let Some(scancode) = scancodes.next().await {
        if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
            // Translates key event to a char if possible, e.g. translates
            // a press event of the 'A' key to either a lowercase or uppercase 'A'
            // depending on whether or not the shift key was pressed.
            if let Some(key) = keyboard.process_keyevent(key_event) {
                match key {
                    DecodedKey::Unicode(character) => print!("{}", character),
                    DecodedKey::RawKey(key) => print!("{:?}", key),
                }
            }
        }
    }
}
