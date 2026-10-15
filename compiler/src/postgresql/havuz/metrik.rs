//! Pool counters: event callbacks never perform IO or reenter the pool lock.
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
const MAX: u64 = 9_007_199_254_740_991;
pub(super) type Gozlem = Arc<dyn Fn([u64; 8]) + Send + Sync>;
pub(super) struct Metrik {
    sayac: Mutex<[u64; 5]>,
    gozlem: Gozlem,
}
impl Metrik {
    pub(super) fn new(gozlem: Gozlem) -> Arc<Self> {
        Arc::new(Self {
            sayac: Mutex::new([0; 5]),
            gozlem,
        })
    }
    fn change(&self, f: impl FnOnce(&mut [u64; 5])) {
        if let Ok(mut values) = self.sayac.lock() {
            f(&mut values);
        }
    }
    pub(super) fn begin(&self) {
        self.change(|s| add(&mut s[0], 1));
    }
    pub(super) fn end(&self, start: Instant) {
        self.change(|s| {
            s[0] = s[0].saturating_sub(1);
            add(&mut s[3], us(start.elapsed()));
        });
    }
    pub(super) fn sample(&self, state: r2d2::State) {
        // state() was read by the caller outside r2d2's HandleEvent callbacks.
        let values = {
            let Ok(values) = self.sayac.lock() else {
                return;
            };
            *values
        };
        (self.gozlem)([
            state.connections as u64,
            state.idle_connections as u64,
            state.connections.saturating_sub(state.idle_connections) as u64,
            values[0],
            values[1],
            values[2],
            values[3],
            values[4],
        ]);
    }
}
fn add(n: &mut u64, delta: u64) {
    *n = n.saturating_add(delta).min(MAX);
}
fn us(duration: Duration) -> u64 {
    duration.as_micros().min(MAX as u128) as u64
}
pub(super) struct Olaylar(pub(super) Arc<Metrik>);
impl std::fmt::Debug for Olaylar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("MetadataOnlyPoolEvents")
    }
}
impl r2d2::HandleEvent for Olaylar {
    fn handle_checkout(&self, _: r2d2::event::CheckoutEvent) {
        self.0.change(|s| add(&mut s[1], 1));
    }
    fn handle_timeout(&self, _: r2d2::event::TimeoutEvent) {
        self.0.change(|s| add(&mut s[2], 1));
    }
    fn handle_checkin(&self, event: r2d2::event::CheckinEvent) {
        self.0.change(|s| add(&mut s[4], us(event.duration())));
    }
}
