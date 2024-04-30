cf-rustracing
==========

[![Crates.io: cf-rustracing](https://img.shields.io/crates/v/cf-rustracing.svg)](https://crates.io/crates/cf-rustracing)
[![Documentation](https://docs.rs/cf-rustracing/badge.svg)](https://docs.rs/cf-rustracing)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

[OpenTracing] API for Rust.

[Documentation](https://docs.rs/rustracing)

Examples
--------

```rust
use cf_rustracing::sampler::AllSampler;
use cf_rustracing::tag::Tag;
use cf_rustracing::Tracer;
use std::thread;
use std::time::Duration;

#[tokio::main]
async fn main() {
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
    
    println!("# SPAN: {:?}", span_rx.recv().await);
    println!("# SPAN: {:?}", span_rx.recv().await);
}
```

As an actual usage example of the crate and an implementation of the [OpenTracing] API,
it may be helpful to looking at [rustracing_jaeger] crate.

References
----------

- [The OpenTracing Semantic Specification (v1.1)][specification]

[OpenTracing]: http://opentracing.io/
[specification]: https://github.com/opentracing/specification/blob/master/specification.md
[rustracing_jaeger]: https://github.com/cloudflare/rustracing_jaeger
