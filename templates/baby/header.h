#ifndef _BABY_CHAMELEON_{{ prefix }}_H
#define _BABY_CHAMELEON_{{ prefix }}_H

#include <stddef.h>

void {{ prefix }}_seed (size_t new_seed);
size_t {{ prefix }}_generate (unsigned char* output, size_t output_capacity);

#endif /* _BABY_CHAMELEON_{{ prefix }}_H */
