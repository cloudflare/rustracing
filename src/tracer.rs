use crate::sampler::Sampler;
use crate::span::{SharedSpanConsumer, SpanConsumer, SpanReceiver, StartSpanOptions};
use std::borrow::Cow;
use std::sync::Arc;
use tokio::sync::mpsc;

/// Tracer.
///
/// # Examples
///
/// ```
/// use cf_rustracing::Tracer;
/// use cf_rustracing::sampler::AllSampler;
///
/// # #[tokio::main]
/// # async fn main(){
/// let (tracer, mut span_rx) = Tracer::new(AllSampler);
/// {
///    let _span = tracer.span("foo").start_with_state(());
/// }
/// let span = span_rx.recv().await.unwrap();
/// assert_eq!(span.operation_name(), "foo");
/// # }
/// ```
#[derive(Debug)]
pub struct Tracer<S, T> {
    sampler: Arc<S>,
    span_tx: SharedSpanConsumer<T>,
}
impl<S: Sampler<T>, T: Send + 'static> Tracer<S, T> {
    /// Makes a new `Tracer` instance with an unbounded [`tokio::sync::mpsc`] channel.
    pub fn new(sampler: S) -> (Self, SpanReceiver<T>) {
        let (span_tx, span_rx) = mpsc::unbounded_channel();
        (Self::with_consumer(sampler, span_tx), span_rx)
    }
}
impl<S: Sampler<T>, T> Tracer<S, T> {
    /// Makes a new `Tracer` instance with a custom consumer implementation.
    pub fn with_consumer<C: SpanConsumer<T> + 'static>(sampler: S, consumer: C) -> Self {
        Self {
            sampler: Arc::new(sampler),
            span_tx: SharedSpanConsumer::new(consumer),
        }
    }

    /// Returns `StartSpanOptions` for starting a span which has the name `operation_name`.
    pub fn span<N>(&self, operation_name: N) -> StartSpanOptions<'_, S, T>
    where
        N: Into<Cow<'static, str>>,
    {
        StartSpanOptions::new(operation_name, &self.span_tx, &self.sampler)
    }
}
impl<S, T> Tracer<S, T> {
    /// Clone with the given `sampler`.
    pub fn clone_with_sampler<U: Sampler<T>>(&self, sampler: U) -> Tracer<U, T> {
        Tracer {
            sampler: Arc::new(sampler),
            span_tx: self.span_tx.clone(),
        }
    }
}
impl<S, T> Clone for Tracer<S, T> {
    fn clone(&self) -> Self {
        Tracer {
            sampler: Arc::clone(&self.sampler),
            span_tx: self.span_tx.clone(),
        }
    }
}
