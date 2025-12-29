#ifndef _CHAMELEON_H
#define _CHAMELEON_H

// Details of ChameleonWalk are private to the generated code
typedef unsigned char ChameleonWalk[32];

void chameleon_seed (size_t new_seed);
void chameleon_init (ChameleonWalk walk, unsigned long capacity);
void chameleon_destroy (ChameleonWalk walk);
size_t chameleon_mutate (ChameleonWalk walk, unsigned char* output, unsigned long output_length);
size_t chameleon_generate (ChameleonWalk walk, unsigned char* output, unsigned long output_length);
int  chameleon_parse (ChameleonWalk walk, unsigned char* input, unsigned long input_length);

#endif /* _CHAMELEON_H */
