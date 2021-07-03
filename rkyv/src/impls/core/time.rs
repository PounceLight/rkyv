//! [`Archive`] implementations for times.

use crate::{Archive, Archived, Deserialize, Fallible, Serialize};
use core::time::Duration;

/// An archived [`Duration`](core::time::Duration).
#[derive(Clone, Copy, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "validation", derive(bytecheck::CheckBytes))]
#[cfg_attr(feature = "strict", repr(C))]
pub struct ArchivedDuration {
    secs: Archived<u64>,
    nanos: Archived<u32>,
}

const NANOS_PER_SEC: u32 = 1_000_000_000;
const NANOS_PER_MILLI: u32 = 1_000_000;
const NANOS_PER_MICRO: u32 = 1_000;
const MILLIS_PER_SEC: u64 = 1_000;
const MICROS_PER_SEC: u64 = 1_000_000;

impl ArchivedDuration {
    /// Returns the number of _whole_ seconds contained by this
    /// `ArchivedDuration`.
    ///
    /// The returned value does not include the fractional (nanosecond) part of the duration, which
    /// can be obtained using [`subsec_nanos`].
    ///
    /// [`subsec_nanos`]: ArchivedDuration::subsec_nanos
    #[inline]
    pub const fn as_secs(&self) -> u64 {
        from_archived!(self.secs)
    }

    /// Returns the fractional part of this `ArchivedDuration`, in whole milliseconds.
    ///
    /// This method does **not** return the length of the duration when represented by milliseconds.
    /// The returned number always represents a fractional portion of a second (i.e., it is less
    /// than one thousand).
    #[inline]
    pub const fn subsec_millis(&self) -> u32 {
        from_archived!(self.nanos) / NANOS_PER_MILLI
    }

    /// Returns the fractional part of this `ArchivedDuration`, in whole microseconds.
    ///
    /// This method does **not** return the length of the duration when represented by microseconds.
    /// The returned number always represents a fractional portion of a second (i.e., it is less
    /// than one million).
    #[inline]
    pub const fn subsec_micros(&self) -> u32 {
        from_archived!(self.nanos) / NANOS_PER_MICRO
    }

    /// Returns the fractional part of this `Duration`, in nanoseconds.
    ///
    /// This method does **not** return the length of the duration when represented by nanoseconds.
    /// The returned number always represents a fractional portion of a second (i.e., it is less
    /// than one billion).
    #[inline]
    pub const fn subsec_nanos(&self) -> u32 {
        from_archived!(self.nanos)
    }

    /// Returns the total number of whole milliseconds contained by this `ArchivedDuration`.
    #[inline]
    pub const fn as_millis(&self) -> u128 {
        self.as_secs() as u128 * MILLIS_PER_SEC as u128
            + (self.subsec_nanos() / NANOS_PER_MILLI) as u128
    }

    /// Returns the total number of whole microseconds contained by this `ArchivedDuration`.
    #[inline]
    pub const fn as_micros(&self) -> u128 {
        self.as_secs() as u128 * MICROS_PER_SEC as u128
            + (self.subsec_nanos() / NANOS_PER_MICRO) as u128
    }

    /// Returns the total number of nanoseconds contained by this `ArchivedDuration`.
    #[inline]
    pub const fn as_nanos(&self) -> u128 {
        self.as_secs() as u128 * NANOS_PER_SEC as u128 + self.subsec_nanos() as u128
    }

    /// Returns the number of seconds contained by this `ArchivedDuration` as `f64`.
    ///
    /// The returned value does include the fractional (nanosecond) part of the duration.
    #[inline]
    pub fn as_secs_f64(&self) -> f64 {
        (self.as_secs() as f64) + (self.subsec_nanos() as f64) / (NANOS_PER_SEC as f64)
    }

    /// Returns the number of seconds contained by this `ArchivedDuration` as `f32`.
    ///
    /// The returned value does include the fractional (nanosecond) part of the duration.
    #[inline]
    pub fn as_secs_f32(&self) -> f32 {
        (self.as_secs() as f32) + (self.subsec_nanos() as f32) / (NANOS_PER_SEC as f32)
    }
}

impl Archive for Duration {
    type Archived = ArchivedDuration;
    type Resolver = ();

    #[inline]
    unsafe fn resolve(&self, pos: usize, _: Self::Resolver, out: *mut Self::Archived) {
        let (fp, fo) = out_field!(out.secs);
        self.as_secs().resolve(pos + fp, (), fo);
        let (fp, fo) = out_field!(out.nanos);
        self.subsec_nanos().resolve(pos + fp, (), fo);
    }
}

impl<S: Fallible + ?Sized> Serialize<S> for Duration {
    #[inline]
    fn serialize(&self, _: &mut S) -> Result<Self::Resolver, S::Error> {
        Ok(())
    }
}

impl<D: Fallible + ?Sized> Deserialize<Duration, D> for ArchivedDuration {
    #[inline]
    fn deserialize(&self, _: &mut D) -> Result<Duration, D::Error> {
        Ok(Duration::new(self.as_secs(), self.subsec_nanos()))
    }
}