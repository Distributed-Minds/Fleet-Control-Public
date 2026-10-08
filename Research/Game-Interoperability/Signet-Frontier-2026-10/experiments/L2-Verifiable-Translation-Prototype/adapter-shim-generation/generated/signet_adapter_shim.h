/* GENERATED. DO NOT EDIT. */
/* generator: l2-adapter-shim-generator@0 */
/* descriptor-sha256: 29663431523e65018681c1644020750af03edc4d82bf478c97966960da0981a5 */
#ifndef SIGNET_ADAPTER_SHIM_RESEARCH_V0_H
#define SIGNET_ADAPTER_SHIM_RESEARCH_V0_H
#include <stdint.h>
#ifdef __cplusplus
extern "C" {
#endif

typedef struct SgAdapterShim SgAdapterShim;

/* Version of the game/engine integration surface exposed by this shim. */
uint32_t sg_adapter_surface_version(SgAdapterShim *shim);

/* Read one local control by adapter-defined identifier. */
int32_t sg_adapter_capture_control(SgAdapterShim *shim, const char *control_id, float *out_value);

/* Read one calibration metric when the legitimate integration surface exposes it. */
int32_t sg_adapter_observe_metric(SgAdapterShim *shim, const char *metric_id, double *out_value);

/* Present one already-resolved local appearance choice. */
int32_t sg_adapter_present_choice(SgAdapterShim *shim, const char *choice_id, const char *payload_json);

#ifdef __cplusplus
}
#endif
#endif
