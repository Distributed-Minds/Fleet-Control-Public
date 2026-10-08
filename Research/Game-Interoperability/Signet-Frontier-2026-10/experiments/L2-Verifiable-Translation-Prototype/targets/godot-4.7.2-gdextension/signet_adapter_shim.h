#pragma once
#include <stdint.h>
typedef struct SgAdapterShim SgAdapterShim;
uint32_t sg_adapter_surface_version(SgAdapterShim *shim);
int32_t sg_adapter_capture_control(SgAdapterShim *shim,const char *control_id,float *out_value);
int32_t sg_adapter_observe_metric(SgAdapterShim *shim,const char *metric_id,double *out_value);
int32_t sg_adapter_present_choice(SgAdapterShim *shim,const char *choice_id,const char *payload_json);
