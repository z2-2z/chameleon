#ifndef _CHAMELEON_{{ prefix }}_H
#define _CHAMELEON_{{ prefix }}_H

#ifndef _HAVE_CHAMELEON_WALK
#define _HAVE_CHAMELEON_WALK
// Details of ChameleonWalk are private to the generated code
typedef unsigned char ChameleonWalk[32];
#endif

void {{ prefix }}_seed (size_t new_seed);
void {{ prefix }}_init (ChameleonWalk walk, unsigned long capacity);
void {{ prefix }}_destroy (ChameleonWalk walk);
size_t {{ prefix }}_mutate (ChameleonWalk walk, unsigned char* output, unsigned long output_length);
size_t {{ prefix }}_generate (ChameleonWalk walk, unsigned char* output, unsigned long output_length);
int  {{ prefix }}_parse (ChameleonWalk walk, unsigned char* input, unsigned long input_length);

#endif /* _CHAMELEON_{{ prefix }}_H */
