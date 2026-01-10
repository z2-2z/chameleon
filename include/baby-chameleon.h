#ifndef _BABY_CHAMELEON_H
#define _BABY_CHAMELEON_H

#include <stddef.h>

void chameleon_seed (size_t new_seed);
size_t chameleon_generate (unsigned char* output, size_t output_capacity);

#endif /* _BABY_CHAMELEON_H */
