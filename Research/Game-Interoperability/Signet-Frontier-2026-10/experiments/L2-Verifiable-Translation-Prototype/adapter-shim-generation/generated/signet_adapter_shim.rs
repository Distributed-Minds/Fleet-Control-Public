// GENERATED. DO NOT EDIT.
// generator: l2-adapter-shim-generator@0
// descriptor-sha256: 29663431523e65018681c1644020750af03edc4d82bf478c97966960da0981a5

pub trait SignetAdapterShim {
    /// Version of the game/engine integration surface exposed by this shim.
    fn surface_version(&mut self) -> u32;
    /// Read one local control by adapter-defined identifier.
    fn capture_control(&mut self, control_id: &str) -> Option<f32>;
    /// Read one calibration metric when the legitimate integration surface exposes it.
    fn observe_metric(&mut self, metric_id: &str) -> Option<f64>;
    /// Present one already-resolved local appearance choice.
    fn present_choice(&mut self, choice_id: &str, payload_json: &str) -> bool;
}
