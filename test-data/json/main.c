#include <stdio.h>
#include <stdlib.h>
#include <time.h>
#include <jq.h>
#include <jv.h>

#include "chameleon.h"

#define OUTPUT_LENGTH (16*4096)

int main (void) {
    ChameleonWalk walk;
    size_t length;
    unsigned char* output = malloc(OUTPUT_LENGTH);
    jq_state *jq = jq_init();
    
    chameleon_init(walk, 4096);
    chameleon_seed(time(NULL));
    
    while (1) {
        length = chameleon_mutate(walk, output, OUTPUT_LENGTH);
        
        if (length == OUTPUT_LENGTH) {
            continue;
        }
        output[length] = 0;
        
        //printf("Testing: %s\n", output);
        
        jv parsed = jv_parse(output);
        
        if (jv_is_valid(parsed)) {
            jv_free(parsed);
        } else {
            printf("INVALID JSON: %s\n", output);
            break;
        }
    }
    
    chameleon_destroy(walk);
    free(output);
}
