#ifndef AVPLAYER_ABI_H
#define AVPLAYER_ABI_H

#include <stddef.h>
#include <stdint.h>

typedef struct {
    const uint8_t *data;
    size_t length;
} AVPBytes;

typedef struct AVPValue AVPValue;

struct AVPValue {
    uint32_t kind;
    uint32_t reserved;
    int64_t signed_value;
    uint64_t unsigned_value;
    double real_value;
    AVPBytes bytes;
    const AVPValue *values;
    const AVPBytes *keys;
    size_t count;
    void *object;
    void *owner;
};

enum {
    AVP_VALUE_NULL = 0,
    AVP_VALUE_BOOL = 1,
    AVP_VALUE_SIGNED = 2,
    AVP_VALUE_UNSIGNED = 3,
    AVP_VALUE_REAL = 4,
    AVP_VALUE_STRING = 5,
    AVP_VALUE_DATA = 6,
    AVP_VALUE_ARRAY = 7,
    AVP_VALUE_DICTIONARY = 8,
    AVP_VALUE_OBJECT = 9
};

_Static_assert(sizeof(AVPBytes) == 16, "AVPBytes size");
_Static_assert(sizeof(AVPValue) == 88, "AVPValue size");
_Static_assert(offsetof(AVPValue, bytes) == 32, "AVPValue bytes offset");
_Static_assert(offsetof(AVPValue, owner) == 80, "AVPValue owner offset");

#endif
