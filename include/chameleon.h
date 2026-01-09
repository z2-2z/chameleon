#ifndef _CHAMELEON_H
#define _CHAMELEON_H

#include <stddef.h>

// Details of ChameleonWalk are private to the generated code
typedef unsigned char ChameleonWalk[32];

void chameleon_seed (size_t new_seed);
void chameleon_init (ChameleonWalk walk, size_t capacity);
void chameleon_destroy (ChameleonWalk walk);
size_t chameleon_mutate (ChameleonWalk walk, unsigned char* output, size_t output_capacity);
size_t chameleon_generate (ChameleonWalk walk, unsigned char* output, size_t output_capacity);

#endif /* _CHAMELEON_H */
