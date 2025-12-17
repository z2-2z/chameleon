// gcc -shared -o /dev/null -fPIC template.c -Wall -Wextra -Wpedantic -DNUM_RULES=3

#include <stdlib.h>

#undef UNLIKELY
#define UNLIKELY(x) __builtin_expect(!!(x), 0)
#undef LIKELY
#define LIKELY(x) __builtin_expect(!!(x), 1)

typedef struct {
    unsigned int* steps;
    size_t length;
    size_t capacity;
} ChameleonWalk;

const unsigned char TERMINAL[1] = {0};

#ifndef OMIT_CHAMELEON_INIT
void chameleon_init (ChameleonWalk* walk, size_t capacity) {
    walk->steps = malloc(capacity * sizeof(unsigned int));
    walk->length = 0;
    walk->capacity = capacity;
}
#endif /* OMIT_CHAMELEON_INIT */

#ifndef OMIT_CHAMELEON_DESTROY
void chameleon_destroy (ChameleonWalk* walk) {
    free(walk->steps);
    __builtin_memset(walk, 0, sizeof(ChameleonWalk));
}
#endif /* OMIT_CHAMELEON_DESTROY */

#if !defined(OMIT_CHAMELEON_MUTATE) || !defined(OMIT_CHAMELEON_GENERATE)
// One production rule
static size_t _mutate_nonterm_Y (unsigned int* steps, const size_t length, const size_t capacity, size_t* step, unsigned char* output, size_t output_length)  {
    unsigned int hit_limit = 0; size_t r;
    size_t s = (*step)++;
    unsigned char* original_output = output;
    
    if (UNLIKELY(s >= capacity)) {
        return 0;
    }
    
    if (s >= length) {
        /* Terminal: */
        if (UNLIKELY(sizeof(TERMINAL) > output_length)) {
            return 0;
        }
        __builtin_memcpy(output, TERMINAL, sizeof(TERMINAL));
        output += sizeof(TERMINAL);
        output_length -= sizeof(TERMINAL);
    }
    
    /* Non-terminals */
    r = _mutate_nonterm_Y(steps, length, capacity, step, output, output_length);
    output += r;
    output_length -= r;
    hit_limit |= !r;
    
    if (UNLIKELY(hit_limit)) {
        return 0;
    }
    
    return (size_t) (output - original_output);
}

// Multiple production rules
static size_t _mutate_nonterm_X (unsigned int* steps, const size_t length, const size_t capacity, size_t* step, unsigned char* output, size_t output_length)  {
    unsigned int hit_limit = 0; size_t r;
    unsigned int mutate, rule;
    size_t s = (*step)++;
    unsigned char* original_output = output;
    
    if (UNLIKELY(s >= capacity)) {
        return 0;
    }
    
    mutate = (s >= length);
    
    if (mutate) {
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
            r = _mutate_nonterm_Y(steps, length, capacity, step, output, output_length);
            output += r;
            output_length -= r;
            hit_limit |= !r;
            
            if (UNLIKELY(hit_limit)) {
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
#endif

#ifndef OMIT_CHAMELEON_MUTATE
size_t chameleon_mutate (ChameleonWalk* walk, unsigned char* output, size_t output_length) {
    size_t length = 0;
    if (LIKELY(walk->length > 0)) {
        length = random() % walk->length;
    }
    walk->length = 0;
    return _mutate_nonterm_X(walk->steps, length, walk->capacity, &walk->length, output, output_length);
}
#endif /* OMIT_CHAMELEON_MUTATE */

#ifndef OMIT_CHAMELEON_GENERATE
size_t chameleon_generate (ChameleonWalk* walk, unsigned char* output, size_t output_length) {
    walk->length = 0;
    return _mutate_nonterm_X(walk->steps, 0, walk->capacity, &walk->length, output, output_length);
}
#endif /* OMIT_CHAMELEON_GENERATE */

//TODO: chameleon_parse
