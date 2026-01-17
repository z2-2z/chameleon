#ifndef _CHAMELEON_H
#define _CHAMELEON_H

#include <stddef.h>


/* A ChameleonWalk is a walk through the rules of the grammar.
 * Every step of the walk is the expansion of a grammar rule
 * that produces at least one Terminal since Chameleon operates
 * on grammars in Greibach-Normal-Form.
 * A walk and its corresponding output are in this re-implementation
 * of Gramatron tied together, so a walk and its output are always
 * passed together as arguments to the functions below.
 *
 * The implemenation details are private to the generated code. */
typedef unsigned char ChameleonWalk[32];


/* Supply a seed to the internal PRNG of Chameleon's mutator/generator.
 * If the passed value is zero, CHAMELEON_SEED is used instead. */
void chameleon_seed (size_t new_seed);


/* Initialize a new ChameleonWalk that can hold capacity amount of steps.
 * This uses malloc(), so you need to call chameleon_destroy() later on. */
void chameleon_init (ChameleonWalk walk, size_t capacity);


/* Free the resources of a ChameleonWalk. */
void chameleon_destroy (ChameleonWalk walk);


/* Given an already existing ChameleonWalk, create a mutant of the walk and write the
 * generated output to output, which can hold at most output_capacity bytes.
 * If the walk is empty / freshly initialized, this functions acts just like chameleon_generate().
 * Returns the amount of bytes written to output, which is at most output_capacity.
 * If the function returns exactly output_capacity bytes, then the output was truncated and the
 * capacity of ChameleonWalk or output_capacity are too small. */
size_t chameleon_mutate (ChameleonWalk walk, unsigned char* output, size_t output_capacity);


/* Generate a completely new walk starting from the entrypoint of the grammar.
 * Write the generated output to output, which can hold at most output_capacity bytes.
 * Returns the amount of bytes written to output, which is at most output_capacity.
 * If the function returns exactly output_capacity bytes, then the output was truncated and the
 * capacity of ChameleonWalk or output_capacity are too small. */
size_t chameleon_generate (ChameleonWalk walk, unsigned char* output, size_t output_capacity);


#endif /* _CHAMELEON_H */
