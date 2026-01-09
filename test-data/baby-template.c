// gcc -shared -o /dev/null -fPIC template.c -Wall -Wextra -Wpedantic -DNUM_RULES=3

#include <stdlib.h>

#undef UNLIKELY
#define UNLIKELY(x) __builtin_expect(!!(x), 0)
#undef LIKELY
#define LIKELY(x) __builtin_expect(!!(x), 1)

const unsigned char TERMINAL[1] = {0};

// Multiple production rules
static size_t _generate_nonterm_X (unsigned char* output, size_t output_length)  {
    size_t r;
    unsigned char* original_output = output;
    
    switch (random() % NUM_RULES) {
        case 0: {
            /* Terminals */
            if (UNLIKELY(sizeof(TERMINAL) > output_length)) {
                return output_length;
            }
            __builtin_memcpy(output, TERMINAL, sizeof(TERMINAL));
            output += sizeof(TERMINAL);
            output_length -= sizeof(TERMINAL);
            
            /* Non-terminals */
            r = _mutate_nonterm_Y(steps, length, capacity, step, output, output_length);
            output += r;
            output_length -= r;
            
            break;
        }
        
        default: {
            __builtin_unreachable();
        }
    }
    
    return (size_t) (output - original_output);
}

void chameleon_seed (size_t seed) {
    
}

size_t chameleon_generate (unsigned char* output, size_t output_length) {
    return _generate_nonterm_X(output, output_length);
}

