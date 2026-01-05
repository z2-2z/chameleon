#ifndef _CHAMELEON_{{ prefix }}_H
#define _CHAMELEON_{{ prefix }}_H

#include <stddef.h>

#ifndef _HAVE_CHAMELEON_WALK
#define _HAVE_CHAMELEON_WALK
// Details of ChameleonWalk are private to the generated code
typedef unsigned char ChameleonWalk[32];
#endif

void {{ prefix }}_seed (size_t new_seed);
void {{ prefix }}_init (ChameleonWalk walk, size_t capacity);
void {{ prefix }}_destroy (ChameleonWalk walk);
size_t {{ prefix }}_mutate (ChameleonWalk walk, unsigned char* output, size_t output_length);
size_t {{ prefix }}_generate (ChameleonWalk walk, unsigned char* output, size_t output_length);
int  {{ prefix }}_parse (ChameleonWalk walk, unsigned char* input, size_t input_length);

#endif /* _CHAMELEON_{{ prefix }}_H */
