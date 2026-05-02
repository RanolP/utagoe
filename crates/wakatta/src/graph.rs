use std::time::Duration;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::track::TrackId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Segment {
    pub start: Duration,
    pub end: Option<Duration>,
}

impl Segment {
    pub fn new(start: Duration, end: Option<Duration>) -> Result<Self, SegmentError> {
        if let Some(end) = end {
            if end < start {
                return Err(SegmentError::EndBeforeStart { start, end });
            }
        }

        Ok(Self { start, end })
    }

    pub fn contains(&self, position: Duration) -> bool {
        position >= self.start && self.end.is_none_or(|end| position <= end)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum SegmentError {
    #[error("segment end {end:?} is before start {start:?}")]
    EndBeforeStart { start: Duration, end: Duration },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Relation {
    pub id: Uuid,
    pub from: TrackId,
    pub to: TrackId,
    pub kind: RelationKind,
    pub segment: Option<Segment>,
    pub note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationKind {
    RemixOf,
    CoverOf,
    MashupOf,
    AdaptationOf,
    ArrangementOf,
    SampleOf,
    TranslationOf,
    SameMelodyAs,
    References,
    PairsWellWith,
    UserDefined(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn segment_rejects_end_before_start() {
        let err = Segment::new(Duration::from_secs(20), Some(Duration::from_secs(10)))
            .expect_err("inverted segment must fail");

        assert_eq!(
            err,
            SegmentError::EndBeforeStart {
                start: Duration::from_secs(20),
                end: Duration::from_secs(10),
            }
        );
    }

    #[test]
    fn segment_contains_positions_inside_closed_range() {
        let segment = Segment::new(Duration::from_secs(10), Some(Duration::from_secs(20)))
            .expect("valid segment");

        assert!(!segment.contains(Duration::from_secs(9)));
        assert!(segment.contains(Duration::from_secs(10)));
        assert!(segment.contains(Duration::from_secs(15)));
        assert!(segment.contains(Duration::from_secs(20)));
        assert!(!segment.contains(Duration::from_secs(21)));
    }

    #[test]
    fn open_ended_segment_contains_any_position_after_start() {
        let segment = Segment::new(Duration::from_secs(10), None).expect("valid segment");

        assert!(!segment.contains(Duration::from_secs(9)));
        assert!(segment.contains(Duration::from_secs(10)));
        assert!(segment.contains(Duration::from_secs(60)));
    }
}
