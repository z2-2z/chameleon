#ifndef _BABY_CHAMELEON_H
#define _BABY_CHAMELEON_H

#include <stddef.h>


/* Supply a seed to the internal PRNG of Chameleon's generator.
 * If the passed value is zero, CHAMELEON_SEED is used instead. */
void chameleon_seed (size_t new_seed);


/* Generate an output that adheres to the specified grammar and write it
 * to output, which can hold at most output_capacity bytes.
 * Returns the number of bytes written to output. */
size_t chameleon_generate (unsigned char* output, size_t output_capacity);


#endif /* _BABY_CHAMELEON_H */
