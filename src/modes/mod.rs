//! One headless state machine per fundamental mechanic. Each owns its rules and
//! interaction loop; the UI only forwards input and draws state.
pub mod experiment;
pub mod position;
pub mod predict;
pub mod shape;
pub mod steer;
pub mod synchronize;
pub mod survey;
pub mod tempo;

pub use experiment::{Conclusion, ExperimentGame, Run, Verdict};
pub use position::{Attempt, PositionGame};
pub use predict::{PredictGame, Round, RoundResult};
pub use shape::{Attempt as ShapeAttempt, PlaceError as ShapePlaceError, ShapeGame, ShapeSpec, Source as ShapeSource, MIN_SEPARATION as SHAPE_MIN_SEPARATION};
pub use synchronize::{entry_time, CondResult as SyncCondResult, Flight as SyncFlight, SyncGame, SyncSpec, DELAY_STEP as SYNC_DELAY_STEP};
pub use steer::{Action, ChooseError, End, FailReason, Phase, SteerGame, SteerSpec};
pub use survey::{PAIR_RAY_LEN, CLEARED_EVERY, CLEARED_MARGIN, EstimateTest, TestError, MIN_SCOUT_SEPARATION, PUSH_EVERY, PUSH_MIN, TESTS_PER_ROUND, CommitError, PlaceError, RetryError, Round as SurveyRound, Scout, SurveyGame, SurveySpec, Verdict as SurveyVerdict};
pub use tempo::{TempoAttempt, TempoGame};

/// Small deterministic generator so Predict rounds are reproducible per (level, attempt).
pub struct Lcg(pub u64);
impl Lcg {
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        self.0 >> 33
    }
    pub fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
}
