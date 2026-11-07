//! Debouncing for concurrent alerts.
//!
//! A concurrent alert should describe a real, lasting condition, not a single
//! bad sample. Whatever evaluates the condition (the device-service watchdog, a
//! smart device checking its sensor value, a script) feeds every sample into a
//! [`Debouncer`], which tells it what to send:
//!
//! - [`Signal::Fire`] once the condition held for `start_after` consecutive
//!   samples, and on every sample after that while it lasts. Each fire extends
//!   the alert's keep-alive, so if the sender dies the alert expires on its own.
//! - [`Signal::Stop`] once the condition was clear for `stop_after` consecutive
//!   samples. Stopping ends the alert right away instead of waiting for the
//!   keep-alive to run out.
//! - [`Signal::Idle`] otherwise.
//!
//! A failed fire needs no special handling: the next sample fires again. A
//! failed stop leaves the alert open until its keep-alive runs out.
//!
//! For numeric limits, [`ThresholdDebouncer`] adds hysteresis on top, so a
//! value hovering around the limit doesn't start and stop alerts.
//!
//! The types are plain state machines with no I/O or clock, so they work the
//! same on a server and on a microcontroller. With a fixed sampling interval,
//! "N samples" equals "N × interval".

/// How many consecutive samples are needed to change state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DebounceConfig {
    /// Consecutive samples with the condition present before the alert starts.
    /// `0` is treated as `1`.
    pub start_after: u32,
    /// Consecutive samples with the condition clear before the alert stops.
    /// `0` is treated as `1`.
    pub stop_after: u32,
}

impl Default for DebounceConfig {
    fn default() -> Self {
        Self {
            start_after: 3,
            stop_after: 2,
        }
    }
}

/// What the sender should do after a sample.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Signal {
    /// No alert is open: send nothing.
    Idle,
    /// The alert is open: fire it (as `AlertKind::Concurrent`) to start or keep
    /// it alive. `started` is `true` on the sample that started it.
    Fire { started: bool },
    /// The condition is over: stop the alert.
    Stop,
}

/// Turns a stream of "is the condition present?" samples into [`Signal`]s.
#[derive(Debug, Clone)]
pub struct Debouncer {
    config: DebounceConfig,
    active: bool,
    /// Consecutive samples that disagree with `active`.
    streak: u32,
}

impl Debouncer {
    pub fn new(config: DebounceConfig) -> Self {
        Self {
            config,
            active: false,
            streak: 0,
        }
    }

    /// Feeds one sample. `condition` is `true` while the alert condition is
    /// present (device unreachable, value too low, ...).
    pub fn update(&mut self, condition: bool) -> Signal {
        if condition == self.active {
            self.streak = 0;
        } else {
            self.streak += 1;
            let needed = if self.active {
                self.config.stop_after
            } else {
                self.config.start_after
            };
            if self.streak >= needed.max(1) {
                self.active = condition;
                self.streak = 0;
                if !condition {
                    return Signal::Stop;
                }
                return Signal::Fire { started: true };
            }
        }
        if self.active {
            Signal::Fire { started: false }
        } else {
            Signal::Idle
        }
    }

    /// Whether the alert is currently open.
    pub fn is_active(&self) -> bool {
        self.active
    }
}

/// Which side of the threshold is bad.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThresholdDirection {
    /// Alert when the value drops below `trigger` (e.g. low temperature).
    Below,
    /// Alert when the value rises above `trigger` (e.g. high temperature).
    Above,
}

/// A numeric limit with hysteresis.
///
/// The condition starts when the value crosses `trigger` and only ends once it
/// is back past `clear`. Values in between keep the current state. Example for
/// low temperature: `trigger = 5.0`, `clear = 6.0`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Threshold {
    pub direction: ThresholdDirection,
    pub trigger: f64,
    pub clear: f64,
}

impl Threshold {
    /// Alert below `trigger`, clear at or above `clear` (`clear >= trigger`).
    pub fn below(trigger: f64, clear: f64) -> Self {
        Self {
            direction: ThresholdDirection::Below,
            trigger,
            clear,
        }
    }

    /// Alert above `trigger`, clear at or below `clear` (`clear <= trigger`).
    pub fn above(trigger: f64, clear: f64) -> Self {
        Self {
            direction: ThresholdDirection::Above,
            trigger,
            clear,
        }
    }

    /// Whether `value` counts as "condition present", given whether the alert
    /// is currently open.
    pub fn condition(&self, value: f64, active: bool) -> bool {
        match (self.direction, active) {
            (ThresholdDirection::Below, false) => value < self.trigger,
            (ThresholdDirection::Below, true) => value < self.clear,
            (ThresholdDirection::Above, false) => value > self.trigger,
            (ThresholdDirection::Above, true) => value > self.clear,
        }
    }
}

/// [`Threshold`] plus [`Debouncer`]: feed raw values, get [`Signal`]s.
#[derive(Debug, Clone)]
pub struct ThresholdDebouncer {
    threshold: Threshold,
    debouncer: Debouncer,
}

impl ThresholdDebouncer {
    pub fn new(threshold: Threshold, config: DebounceConfig) -> Self {
        Self {
            threshold,
            debouncer: Debouncer::new(config),
        }
    }

    pub fn update(&mut self, value: f64) -> Signal {
        let condition = self.threshold.condition(value, self.debouncer.is_active());
        self.debouncer.update(condition)
    }

    pub fn is_active(&self) -> bool {
        self.debouncer.is_active()
    }

    /// Replaces the limits, keeping the current state (e.g. after a config change).
    pub fn set_threshold(&mut self, threshold: Threshold) {
        self.threshold = threshold;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const IDLE: Signal = Signal::Idle;
    const START: Signal = Signal::Fire { started: true };
    const KEEP: Signal = Signal::Fire { started: false };
    const STOP: Signal = Signal::Stop;

    fn feed(d: &mut Debouncer, samples: &[bool]) -> Vec<Signal> {
        samples.iter().map(|s| d.update(*s)).collect()
    }

    #[test]
    fn single_failure_does_not_start() {
        let mut d = Debouncer::new(DebounceConfig {
            start_after: 3,
            stop_after: 2,
        });
        assert_eq!(
            feed(&mut d, &[true, false, true, true, false]),
            vec![IDLE; 5]
        );
        assert!(!d.is_active());
    }

    #[test]
    fn starts_keeps_alive_and_stops_once() {
        let mut d = Debouncer::new(DebounceConfig {
            start_after: 3,
            stop_after: 2,
        });
        let out = feed(
            &mut d,
            &[true, true, true, true, false, true, false, false, false],
        );
        assert_eq!(
            out,
            vec![
                IDLE, IDLE, START, // starts after 3 bad samples
                KEEP,  // and is kept alive while bad
                KEEP,  // one good sample is not enough to stop
                KEEP,  // and a bad one resets the streak
                KEEP, STOP, IDLE,
            ]
        );
        assert!(!d.is_active());
    }

    #[test]
    fn zero_thresholds_act_immediately() {
        let mut d = Debouncer::new(DebounceConfig {
            start_after: 0,
            stop_after: 0,
        });
        assert_eq!(d.update(true), START);
        assert_eq!(d.update(false), STOP);
    }

    #[test]
    fn restarts_after_stop() {
        let mut d = Debouncer::new(DebounceConfig {
            start_after: 1,
            stop_after: 1,
        });
        assert_eq!(feed(&mut d, &[true, false, true]), vec![START, STOP, START]);
    }

    #[test]
    fn hysteresis_below() {
        let mut d = ThresholdDebouncer::new(
            Threshold::below(5.0, 6.0),
            DebounceConfig {
                start_after: 2,
                stop_after: 2,
            },
        );
        // 5.5 is between the limits: not enough to start an alert.
        assert_eq!(d.update(5.5), IDLE);
        assert_eq!(d.update(4.9), IDLE);
        assert_eq!(d.update(4.8), START);
        // Back above trigger but below clear: still an alert.
        assert_eq!(d.update(5.5), KEEP);
        assert_eq!(d.update(5.9), KEEP);
        assert_eq!(d.update(6.0), KEEP);
        assert_eq!(d.update(6.1), STOP);
    }

    #[test]
    fn hysteresis_above() {
        let mut d = ThresholdDebouncer::new(
            Threshold::above(30.0, 28.0),
            DebounceConfig {
                start_after: 1,
                stop_after: 1,
            },
        );
        assert_eq!(d.update(30.0), IDLE);
        assert_eq!(d.update(30.5), START);
        assert_eq!(d.update(29.0), KEEP);
        assert_eq!(d.update(28.0), STOP);
    }
}
