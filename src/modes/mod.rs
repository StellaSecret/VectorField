//! One headless state machine per fundamental mechanic. Each owns its rules and
//! interaction loop; the UI only forwards input and draws state.
pub mod experiment;
pub mod position;
pub mod predict;
pub mod scout;
pub mod shape;
pub mod steer;
pub mod survey;
pub mod synchronize;
pub mod tempo;

pub use experiment::{Conclusion, ExperimentGame, Run, Verdict};
pub use position::{Attempt, PositionGame};
pub use predict::{PredictGame, Round, RoundResult};
pub use scout::{
    Call as ScoutCall, Scout as ScoutFlight, ScoutError, ScoutGame, ScoutPhase, ScoutSpec,
};
pub use shape::{
    influence_radius_of, Attempt as ShapeAttempt, PlaceError as ShapePlaceError, ShapeGame,
    ShapeSpec, Source as ShapeSource, MIN_SEPARATION as SHAPE_MIN_SEPARATION,
};
pub use steer::{Action, ChooseError, End, FailReason, Phase, SteerGame, SteerSpec};
pub use survey::{
    CommitError, EstimateTest, PlaceError, RetryError, Round as SurveyRound, Scout, SurveyGame,
    SurveySpec, TestError, Verdict as SurveyVerdict, CLEARED_EVERY, CLEARED_MARGIN,
    MIN_SCOUT_SEPARATION, PAIR_RAY_LEN, PUSH_EVERY, PUSH_MIN, TESTS_PER_ROUND,
};
pub use synchronize::{
    entry_time, CondResult as SyncCondResult, Flight as SyncFlight, SyncGame, SyncSpec,
    DELAY_STEP as SYNC_DELAY_STEP,
};
pub use tempo::{TempoAttempt, TempoGame};

/// Small deterministic generator so Predict rounds are reproducible per (level, attempt).
pub struct Lcg(pub u64);
impl Lcg {
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.0 >> 33
    }
    pub fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
}
