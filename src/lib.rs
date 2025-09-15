//! [OpenTracing][opentracing] API for Rust
//!
//! # Examples
//!
//! ```
//! use cf_rustracing::sampler::AllSampler;
//! use cf_rustracing::tag::Tag;
//! use cf_rustracing::Tracer;
//! use std::thread;
//! use std::time::Duration;
//!
//! # #[tokio::main]
//! # async fn main() {
//! // Creates a tracer
//! let (tracer, mut span_rx) = Tracer::new(AllSampler);
//! {
//!     // Starts "parent" span
//!     let parent_span = tracer.span("parent").start_with_state(());
//!     thread::sleep(Duration::from_millis(10));
//!     {
//!         // Starts "child" span
//!         let mut child_span = tracer
//!             .span("child_span")
//!             .child_of(&parent_span)
//!             .tag(Tag::new("key", "value"))
//!             .start_with_state(());
//!
//!         child_span.log(|log| {
//!             log.error().message("a log message");
//!         });
//!     } // The "child" span dropped and will be sent to `span_rx`
//! } // The "parent" span dropped and will be sent to `span_rx`
//!
//! println!("# SPAN: {:?}", span_rx.recv().await);
//! println!("# SPAN: {:?}", span_rx.recv().await);
//! # }
//! ```
//!
//! As an actual usage example of the crate and an implmentation of the [OpenTracing] API,
//! it may be helpful to looking at [rustracing_jaeger] crate.
//!
//! # References
//!
//! - [The OpenTracing Semantic Specification (v1.1)][specification]
//!
//! [opentracing]: http://opentracing.io/
//! [specification]: https://github.com/opentracing/specification/blob/master/specification.md
//! [rustracing_jaeger]: https://github.com/sile/rustracing_jaeger
#![warn(missing_docs)]
#![allow(clippy::new_ret_no_self)]
#[macro_use]
extern crate trackable;

pub use crate::error::{Error, ErrorKind};
pub use crate::tracer::Tracer;

pub mod carrier;
pub mod convert;
pub mod log;
pub mod sampler;
pub mod span;
pub mod tag;

mod error;
mod tracer;

/// This crate specific `Result` type.
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sampler::AllSampler;
    use crate::span::{FinishedSpan, Span};
    use crate::tag::{StdTag, Tag, TagValue};
    use std::sync::atomic::{AtomicI64, Ordering};
    use std::thread;
    use std::time::Duration;

    #[tokio::test]
    async fn it_works() {
        let (tracer, mut span_rx) = Tracer::new(AllSampler);
        {
            let span = tracer.span("it_works").start_with_state(());
            let mut child = span.child("child", |options| options.start_with_state(()));
            child.set_tags(|| StdTag::peer_addr("127.0.0.1:80".parse().unwrap()));
        }

        let span = span_rx.recv().await.unwrap();
        assert_eq!(span.operation_name(), "child");

        let span = span_rx.recv().await.unwrap();
        assert_eq!(span.operation_name(), "it_works");
    }

    #[tokio::test]
    async fn example_code_works() {
        // Creates a tracer
        let (tracer, mut span_rx) = Tracer::new(AllSampler);
        {
            // Starts "parent" span
            let parent_span = tracer.span("parent").start_with_state(());
            thread::sleep(Duration::from_millis(10));
            {
                // Starts "child" span
                let mut child_span = tracer
                    .span("child_span")
                    .child_of(&parent_span)
                    .tag(Tag::new("key", "value"))
                    .start_with_state(());

                child_span.log(|log| {
                    log.error().message("a log message");
                });
            } // The "child" span dropped and will be sent to `span_rx`
        } // The "parent" span dropped and will be sent to `span_rx`

        let span = span_rx.recv().await.unwrap();
        assert_eq!(span.operation_name(), "child_span");

        let span = span_rx.recv().await.unwrap();
        assert_eq!(span.operation_name(), "parent");
    }

    #[test]
    fn finish_callback() {
        let span_counter = AtomicI64::new(1);
        let finish_cb = move |span: &mut Span<()>| {
            let call_num = span_counter.fetch_add(1, Ordering::Relaxed);
            span.set_tag(|| Tag::new("callback-call", call_num));
        };

        let (tracer, mut span_rx) = Tracer::new(AllSampler);
        {
            let parent_span = tracer
                .span("parent")
                .finish_callback(finish_cb)
                .start_with_state(());

            let child_with_cb = parent_span.child("child_with_cb", |s| s.start_with_state(()));
            let mut child_no_cb = parent_span.child("child_no_cb", |s| s.start_with_state(()));
            child_no_cb.take_finish_callback();

            drop(child_with_cb);
            drop(child_no_cb);
            drop(parent_span);
        }

        let find_span_counter = |s: &FinishedSpan<()>| match s
            .tags()
            .iter()
            .find(|t| t.name() == "callback-call")?
            .value()
        {
            TagValue::Integer(v) => Some(*v),
            v => panic!("invalid `callback-call` tag value: {v:?}"),
        };

        let child_with_cb = span_rx.try_recv().unwrap();
        assert_eq!(child_with_cb.operation_name(), "child_with_cb");
        assert_eq!(find_span_counter(&child_with_cb), Some(1));

        let child_no_cb = span_rx.try_recv().unwrap();
        assert_eq!(child_no_cb.operation_name(), "child_no_cb");
        assert_eq!(find_span_counter(&child_no_cb), None);

        let parent_span = span_rx.try_recv().unwrap();
        assert_eq!(parent_span.operation_name(), "parent");
        assert_eq!(find_span_counter(&parent_span), Some(2));
    }

    #[allow(dead_code)]
    fn span_can_be_shared() {
        fn trait_check<T: Send + Sync>() {}
        trait_check::<Span<()>>();
    }
}
