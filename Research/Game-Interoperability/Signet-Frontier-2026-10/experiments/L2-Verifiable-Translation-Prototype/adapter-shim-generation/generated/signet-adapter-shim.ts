// GENERATED. DO NOT EDIT.
// generator: l2-adapter-shim-generator@0
// descriptor-sha256: 29663431523e65018681c1644020750af03edc4d82bf478c97966960da0981a5

export interface SignetAdapterShim {
  /** Version of the game/engine integration surface exposed by this shim. */
  surface_version(): number;
  /** Read one local control by adapter-defined identifier. */
  capture_control(control_id: string): number | null;
  /** Read one calibration metric when the legitimate integration surface exposes it. */
  observe_metric(metric_id: string): number | null;
  /** Present one already-resolved local appearance choice. */
  present_choice(choice_id: string, payload_json: string): boolean;
}
