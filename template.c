#include <stdlib.h>

#undef UNLIKELY
#define UNLIKELY(x) __builtin_expect(!!(x), 0)
#undef LIKELY
#define LIKELY(x) __builtin_expect(!!(x), 1)

typedef struct {
    unsigned int* steps;
    size_t length;
    size_t capacity;
} PeacockWalk;

const unsigned char TERMINAL[1] = {0};

#ifndef OMIT_PEACOCK_INIT
void peacock_init (PeacockWalk* walk, size_t capacity) {
    walk->steps = malloc(capacity * sizeof(unsigned int));
    walk->length = 0;
    walk->capacity = capacity;
}
#endif /* OMIT_PEACOCK_INIT */

#ifndef OMIT_PEACOCK_MUTATE
static size_t _mutate_nonterm_X (unsigned int* steps, size_t* length, const size_t capacity, size_t* step, unsigned char* output, size_t output_length)  {
    unsigned int limited = 0, mutate, rule;
    size_t r, s = (*step)++;
    unsigned char* original_output = output;
    
    if (UNLIKELY(s >= capacity)) {
        return 0;
    }
    
    mutate = (s >= *length);
    
    if (mutate) {
        *length = s + 1;
        rule = random() % NUM_RULES;
        steps[s] = rule;
    } else {
        rule = steps[s];
    }
    
    switch (rule) {
        case 0: {
            /* Terminals */
            if (mutate) {
                if (UNLIKELY(sizeof(TERMINAL) > output_length)) {
                    return 0;
                }
                __builtin_memcpy(output, TERMINAL, sizeof(TERMINAL));
            }
            output += sizeof(TERMINAL);
            output_length -= sizeof(TERMINAL);
            
            /* Non-terminals */
            r = _mutate_nonterm_X(steps, length, capacity, step, output, output_length);
            output += r;
            output_length -= r;
            limited |= !r;
            
            /* end: */
            if (UNLIKELY(limited)) {
                return 0;
            }
            break;
        }
        
        default: {
            __builtin_unreachable();
        }
    }
    
    return (size_t) (output - original_output);
}

size_t peacock_mutate (PeacockWalk* walk, unsigned char* output, size_t output_length) {
    size_t step = 0;
    walk->length = random() % walk->length;
    return _mutate_nonterm_X(walk->steps, &walk->length, walk->capacity, &step, output, output_length);
}

size_t peacock_generate (PeacockWalk* walk, unsigned char* output, size_t output_length) {
    size_t step = 0;
    walk->length = 0;
    return _mutate_nonterm_X(walk->steps, &walk->length, walk->capacity, &step, output, output_length);
}

#endif /* OMIT_PEACOCK_MUTATE */
