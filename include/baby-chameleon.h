#ifndef _CHAMELEON_H
#define _CHAMELEON_H

#include <stddef.h>

void chameleon_seed (size_t new_seed);
size_t chameleon_generate (unsigned char* output, size_t output_capacity);

#endif /* _CHAMELEON_H */
