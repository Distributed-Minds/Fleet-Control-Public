// GENERATED. DO NOT EDIT.
// generator: l2-role-contract-generator@1
// descriptor-sha256: 026c4d2dc9b3f4686c9e7ea67685535ead8c25e0791c6bd0e947618428b21312

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdapterOperation {
    AuthorityDriftCorrection,
    InputPositionRotation,
    InputRightClick,
    PresentationActorMove,
    PresentationDamage,
    PresentationDeath,
    WorldFill,
}

pub trait SignetAdapterRoleOps {
    /// role=AUTHORITY; authority=AUTHORITY_CORRECTION; evidence=DOCS
    fn authority_drift_correction(&mut self, request: &[u8], response: &mut Vec<u8>) -> Result<(), i32>;
    /// role=INPUT; authority=LOCAL_OBSERVATION; evidence=DOCS
    fn input_position_rotation(&mut self, request: &[u8], response: &mut Vec<u8>) -> Result<(), i32>;
    /// role=INPUT; authority=LOCAL_OBSERVATION; evidence=DOCS
    fn input_right_click(&mut self, request: &[u8], response: &mut Vec<u8>) -> Result<(), i32>;
    /// role=PRESENTATION; authority=LOCAL_PRESENTATION; evidence=DOCS
    fn presentation_actor_move(&mut self, request: &[u8], response: &mut Vec<u8>) -> Result<(), i32>;
    /// role=PRESENTATION; authority=LOCAL_PRESENTATION; evidence=DOCS
    fn presentation_damage(&mut self, request: &[u8], response: &mut Vec<u8>) -> Result<(), i32>;
    /// role=PRESENTATION; authority=LOCAL_PRESENTATION; evidence=DOCS
    fn presentation_death(&mut self, request: &[u8], response: &mut Vec<u8>) -> Result<(), i32>;
    /// role=WORLD; authority=SEMANTIC_MAPPING; evidence=DOCS
    fn world_fill(&mut self, request: &[u8], response: &mut Vec<u8>) -> Result<(), i32>;
}
