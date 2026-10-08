/* GENERATED. DO NOT EDIT. */
/* generator: l2-role-contract-generator@1 */
/* descriptor-sha256: 026c4d2dc9b3f4686c9e7ea67685535ead8c25e0791c6bd0e947618428b21312 */
#pragma once
#include <stddef.h>
#include <stdint.h>
typedef struct SgAdapterContext SgAdapterContext;
typedef struct { const uint8_t *ptr; size_t len; } SgAdapterBytes;
typedef struct { uint8_t *ptr; size_t capacity; size_t *written; } SgAdapterOut;

#define SG_ADAPTER_OP_AUTHORITY_DRIFT_CORRECTION "authority.drift_correction"
#define SG_ADAPTER_ROLE_AUTHORITY_DRIFT_CORRECTION "AUTHORITY"
int32_t sg_adapter_op_authority_drift_correction(SgAdapterContext *ctx, SgAdapterBytes request, SgAdapterOut response);

#define SG_ADAPTER_OP_INPUT_POSITION_ROTATION "input.position_rotation"
#define SG_ADAPTER_ROLE_INPUT_POSITION_ROTATION "INPUT"
int32_t sg_adapter_op_input_position_rotation(SgAdapterContext *ctx, SgAdapterBytes request, SgAdapterOut response);

#define SG_ADAPTER_OP_INPUT_RIGHT_CLICK "input.right_click"
#define SG_ADAPTER_ROLE_INPUT_RIGHT_CLICK "INPUT"
int32_t sg_adapter_op_input_right_click(SgAdapterContext *ctx, SgAdapterBytes request, SgAdapterOut response);

#define SG_ADAPTER_OP_PRESENTATION_ACTOR_MOVE "presentation.actor_move"
#define SG_ADAPTER_ROLE_PRESENTATION_ACTOR_MOVE "PRESENTATION"
int32_t sg_adapter_op_presentation_actor_move(SgAdapterContext *ctx, SgAdapterBytes request, SgAdapterOut response);

#define SG_ADAPTER_OP_PRESENTATION_DAMAGE "presentation.damage"
#define SG_ADAPTER_ROLE_PRESENTATION_DAMAGE "PRESENTATION"
int32_t sg_adapter_op_presentation_damage(SgAdapterContext *ctx, SgAdapterBytes request, SgAdapterOut response);

#define SG_ADAPTER_OP_PRESENTATION_DEATH "presentation.death"
#define SG_ADAPTER_ROLE_PRESENTATION_DEATH "PRESENTATION"
int32_t sg_adapter_op_presentation_death(SgAdapterContext *ctx, SgAdapterBytes request, SgAdapterOut response);

#define SG_ADAPTER_OP_WORLD_FILL "world.fill"
#define SG_ADAPTER_ROLE_WORLD_FILL "WORLD"
int32_t sg_adapter_op_world_fill(SgAdapterContext *ctx, SgAdapterBytes request, SgAdapterOut response);

