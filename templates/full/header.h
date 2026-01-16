#ifndef _CHAMELEON_{{ prefix }}_H
#define _CHAMELEON_{{ prefix }}_H

#include <stddef.h>

// Details of ChameleonWalk are private to the generated code
typedef unsigned char ChameleonWalk[32];

void {{ prefix }}_seed (size_t new_seed);
void {{ prefix }}_init (ChameleonWalk walk, size_t capacity);
void {{ prefix }}_destroy (ChameleonWalk walk);
size_t {{ prefix }}_mutate (ChameleonWalk walk, unsigned char* output, size_t output_capacity);
size_t {{ prefix }}_generate (ChameleonWalk walk, unsigned char* output, size_t output_capacity);

#endif /* _CHAMELEON_{{ prefix }}_H */
