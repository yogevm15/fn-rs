#include <stdint.h>

typedef struct FName
{
    uint32_t ComparisonIndex;
    uint32_t Number;

} FName;


typedef struct UObject UObject;
struct UObject
{
    void** VFTable;
    int32_t ObjectFlags;
    int32_t InternalIndex;
    UObject* ClassPrivate;
    FName NamePrivate;
    UObject* OuterPrivate;

};

typedef UObject* (*SpawnActorO)(UObject* World, UObject* Class, void* Position, void* Rotation, void* SpawnParameters);

