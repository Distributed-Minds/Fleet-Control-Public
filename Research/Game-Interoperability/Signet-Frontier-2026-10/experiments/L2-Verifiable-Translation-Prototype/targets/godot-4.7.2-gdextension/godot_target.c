#include "godot_abi_subset.h"
#include "signet_adapter_shim.h"
#include <string.h>
struct SgAdapterShim { GDExtensionGodotVersion2 version; int initialized; };
static struct SgAdapterShim g_shim;
static void init_cb(void *u,GDExtensionInitializationLevel l){(void)u;if(l>=GDEXTENSION_INITIALIZATION_CORE)g_shim.initialized=1;}
static void deinit_cb(void *u,GDExtensionInitializationLevel l){(void)u;if(l==GDEXTENSION_INITIALIZATION_CORE)g_shim.initialized=0;}
L2_EXPORT GDExtensionBool signet_l2_godot_library_init(GDExtensionInterfaceGetProcAddress gp,GDExtensionClassLibraryPtr lib,GDExtensionInitialization *init){
 (void)lib;if(!gp||!init)return 0;
 GDExtensionInterfaceGetGodotVersion2 gv=(GDExtensionInterfaceGetGodotVersion2)(void*)gp("get_godot_version2");
 if(!gv)return 0;memset(&g_shim,0,sizeof(g_shim));gv(&g_shim.version);
 init->minimum_initialization_level=GDEXTENSION_INITIALIZATION_CORE;init->userdata=&g_shim;init->initialize=init_cb;init->deinitialize=deinit_cb;return 1;
}
L2_EXPORT SgAdapterShim *signet_l2_godot_shim(void){return &g_shim;}
uint32_t sg_adapter_surface_version(SgAdapterShim *s){return s?(s->version.major<<16)|(s->version.minor<<8)|s->version.patch:0;}
int32_t sg_adapter_capture_control(SgAdapterShim*s,const char*i,float*o){(void)s;(void)i;(void)o;return 0;}
int32_t sg_adapter_observe_metric(SgAdapterShim*s,const char*i,double*o){(void)s;(void)i;(void)o;return 0;}
int32_t sg_adapter_present_choice(SgAdapterShim*s,const char*i,const char*p){(void)s;(void)i;(void)p;return 0;}
