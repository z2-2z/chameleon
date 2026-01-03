#include <stdio.h>
#include <stdlib.h>
#include <time.h>
#include <json.h>

#include "chameleon.h"

#define OUTPUT_LENGTH (16*4096)

int main (void) {
    ChameleonWalk walk;
    size_t length;
    unsigned char* output = malloc(OUTPUT_LENGTH);
    struct json_tokener* tokener = json_tokener_new_ex(256);
    enum json_tokener_error error;
    struct json_object* object;
    
    chameleon_init(walk, 4096);
    chameleon_seed(time(NULL));
    
    while (1) {
        length = chameleon_mutate(walk, output, OUTPUT_LENGTH);
        
        if (length == OUTPUT_LENGTH) {
            length = chameleon_generate(walk, output, OUTPUT_LENGTH);
            continue;
        }
        output[length] = 0;
        
        //printf("Testing: %s\n", output);
        
        json_tokener_reset(tokener);
        json_tokener_set_flags(tokener, JSON_TOKENER_STRICT /*| JSON_TOKENER_VALIDATE_UTF8*/);
        object = json_tokener_parse_ex(tokener, (const char*) output, (int) length + 1);
        error = json_tokener_get_error(tokener);
        
        if (error == json_tokener_success) {
            if (object) {
                while (json_object_put(object) == 0);
            }
        } else {
            printf("INVALID JSON: %s\n", json_tokener_error_desc(error));
            printf("%s\n", output);
            break;
        }
    }
    
    json_tokener_free(tokener);
    chameleon_destroy(walk);
    free(output);
}
