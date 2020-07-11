// This file is taken from amethyst_core
use std::time::{Duration, Instant};

/// A stopwatch which accurately measures elapsed time.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Stopwatch {
    /// Initial state with an elapsed time value of 0 seconds.
    Waiting,
    /// Stopwatch has started counting the elapsed time since this `Instant`
    /// and accumuluated time from previous start/stop cycles `Duration`.
    Started(Duration, Instant),
    /// Stopwatch has been stopped and reports the elapsed time `Duration`.
    Ended(Duration),
}

impl Default for Stopwatch {
    fn default() -> Stopwatch {
        Stopwatch::Waiting
    }
}

impl Stopwatch {
    /// Creates a new stopwatch.
    pub fn new() -> Stopwatch {
        Default::default()
    }

    /// Retrieves the elapsed time.
    pub fn elapsed(&self) -> Duration {
        match *self {
            Stopwatch::Waiting => Duration::new(0, 0),
            Stopwatch::Started(dur, start) => dur + start.elapsed(),
            Stopwatch::Ended(dur) => dur,
        }
    }

    /// Stops, resets, and starts the stopwatch again.
    pub fn restart(&mut self) {
        *self = Stopwatch::Started(Duration::new(0, 0), Instant::now());
    }

    /// Starts, or resumes, measuring elapsed time. If the stopwatch has been
    /// started and stopped before, the new results are compounded onto the
    /// existing elapsed time value.
    ///
    /// Note: Starting an already running stopwatch will do nothing.
    pub fn start(&mut self) {
        match *self {
            Stopwatch::Waiting => self.restart(),
            Stopwatch::Ended(dur) => {
                *self = Stopwatch::Started(dur, Instant::now());
            }
            _ => {}
        }
    }

    /// Stops measuring elapsed time.
    ///
    /// Note: Stopping a stopwatch that isn't running will do nothing.
    pub fn stop(&mut self) {
        if let Stopwatch::Started(dur, start) = *self {
            *self = Stopwatch::Ended(dur + start.elapsed());
        }
    }

    /// Clears the current elapsed time value.
    pub fn reset(&mut self) {
        *self = Stopwatch::Waiting;
    }
}
