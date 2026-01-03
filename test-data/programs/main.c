#include <stdio.h>
#include <stdlib.h>
#include <time.h>

#include "chameleon.h"

#define OUTPUT_LENGTH (4*4096)

int main (void) {
    ChameleonWalk walk;
    size_t length;
    unsigned char* output = malloc(OUTPUT_LENGTH);
    
    chameleon_init(walk, 4096);
    chameleon_seed(time(NULL));
    
    length = chameleon_generate(walk, output, OUTPUT_LENGTH);
    printf("%.*s\n\n", (int) length, output);
    
    length = chameleon_mutate(walk, output, OUTPUT_LENGTH);
    printf("%.*s\n\n", (int) length, output);
    
    length = chameleon_mutate(walk, output, OUTPUT_LENGTH);
    printf("%.*s\n\n", (int) length, output);
    
    length = chameleon_mutate(walk, output, OUTPUT_LENGTH);
    printf("%.*s\n\n", (int) length, output);
    
    chameleon_destroy(walk);
    free(output);
}
