/* GENERATED. DO NOT EDIT. */
/* generator: l2-role-contract-generator@1 */
/* descriptor-sha256: 2c3b3d9f374172435ec505358e9fc084b844e0914fd2bd77b12f6aacaebd12e2 */
#pragma once
#include <stddef.h>
#include <stdint.h>
typedef struct SgAdapterContext SgAdapterContext;
typedef struct { const uint8_t *ptr; size_t len; } SgAdapterBytes;
typedef struct { uint8_t *ptr; size_t capacity; size_t *written; } SgAdapterOut;

#define SG_ADAPTER_OP_HOST_SURFACE_VERSION "host.surface_version"
#define SG_ADAPTER_ROLE_HOST_SURFACE_VERSION "HOST"
int32_t sg_adapter_op_host_surface_version(SgAdapterContext *ctx, SgAdapterBytes request, SgAdapterOut response);

