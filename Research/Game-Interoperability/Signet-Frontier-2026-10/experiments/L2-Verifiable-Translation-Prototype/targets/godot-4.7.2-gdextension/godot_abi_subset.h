#pragma once
#include <stdint.h>
typedef uint8_t GDExtensionBool;
typedef struct GDExtensionClassLibraryPtr_opaque *GDExtensionClassLibraryPtr;
typedef enum { GDEXTENSION_INITIALIZATION_CORE=0, GDEXTENSION_INITIALIZATION_SERVERS=1, GDEXTENSION_INITIALIZATION_SCENE=2, GDEXTENSION_INITIALIZATION_EDITOR=3, GDEXTENSION_MAX_INITIALIZATION_LEVEL=4 } GDExtensionInitializationLevel;
typedef void (*GDExtensionInitializeCallback)(void*,GDExtensionInitializationLevel);
typedef void (*GDExtensionDeinitializeCallback)(void*,GDExtensionInitializationLevel);
typedef struct { GDExtensionInitializationLevel minimum_initialization_level; void *userdata; GDExtensionInitializeCallback initialize; GDExtensionDeinitializeCallback deinitialize; } GDExtensionInitialization;
typedef void (*GDExtensionInterfaceFunctionPtr)(void);
typedef GDExtensionInterfaceFunctionPtr (*GDExtensionInterfaceGetProcAddress)(const char*);
typedef struct { uint32_t major,minor,patch,hex; const char *status,*build,*hash; uint64_t timestamp; const char *string; } GDExtensionGodotVersion2;
typedef void (*GDExtensionInterfaceGetGodotVersion2)(GDExtensionGodotVersion2*);
#if defined(_WIN32)
#define L2_EXPORT __declspec(dllexport)
#elif defined(__GNUC__)
#define L2_EXPORT __attribute__((visibility("default")))
#else
#define L2_EXPORT
#endif
