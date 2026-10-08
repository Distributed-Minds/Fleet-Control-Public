#include "godot_abi_subset.h"
#include "signet_adapter_shim.h"
#include <dlfcn.h>
#include <stdio.h>
#include <string.h>
typedef GDExtensionBool(*InitFn)(GDExtensionInterfaceGetProcAddress,GDExtensionClassLibraryPtr,GDExtensionInitialization*);
typedef SgAdapterShim*(*ShimFn)(void);
static void get_version(GDExtensionGodotVersion2*v){memset(v,0,sizeof(*v));v->major=4;v->minor=7;v->patch=2;v->hex=0x040702;v->status="stable";v->hash="ed1daf0bf001b61586d9930840f2f1394092c079";}
static GDExtensionInterfaceFunctionPtr gp(const char*n){return strcmp(n,"get_godot_version2")==0?(GDExtensionInterfaceFunctionPtr)(void*)get_version:NULL;}
int main(void){void*l=dlopen("./libsignet_l2_godot.so",RTLD_NOW);if(!l)return 2;InitFn f=(InitFn)dlsym(l,"signet_l2_godot_library_init");ShimFn s=(ShimFn)dlsym(l,"signet_l2_godot_shim");uint32_t(*v)(SgAdapterShim*)=(uint32_t(*)(SgAdapterShim*))dlsym(l,"sg_adapter_surface_version");GDExtensionInitialization i={0};if(!f||!s||!v||!f(gp,NULL,&i))return 3;i.initialize(i.userdata,GDEXTENSION_INITIALIZATION_CORE);uint32_t got=v(s());printf("godot-entry-init: PASS\ngodot-version-discovery: 0x%06x\n",got);if(got!=0x040702)return 4;puts("generated-shim-surface: PASS");return 0;}
