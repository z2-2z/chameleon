/***** DEPENDENCIES *****/

#include <stdlib.h>
#include <stdint.h>

/***** MACROS *****/

#undef UNLIKELY
#define UNLIKELY(x) __builtin_expect(!!(x), 0)
#undef LIKELY
#define LIKELY(x) __builtin_expect(!!(x), 1)

#undef THREAD_LOCAL
#ifdef CHAMELEON_THREAD_SAFE
#define THREAD_LOCAL __thread
#else
#define THREAD_LOCAL
#endif

#undef EXPORT_FUNCTION
#ifdef CHAMELEON_VISIBLE
#define EXPORT_FUNCTION __attribute__((visibility ("default")))
#else
#define EXPORT_FUNCTION
#endif

#ifndef CHAMELEON_SEED
 #define CHAMELEON_SEED 1739639165216539016ULL
#endif

#define TRIANGULAR_RANDOM(n) (TRIANGULAR_LOOKUP_TABLE[internal_random() % ((n * (n + 1)) >> 1)])
#define LINEAR_RANDOM(n) (internal_random() % n)

/***** TYPES *****/

typedef {{ grammar.step_type() }} step_t;

typedef struct {
    step_t* steps;
    size_t length;
    size_t capacity;
} ChameleonWalk;

/***** PRNG *****/

static THREAD_LOCAL size_t rand_state = CHAMELEON_SEED;

static inline size_t internal_random (void) {
    size_t x = rand_state;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    return rand_state = x;
}
{% if grammar.max_num_of_rules() > 0 %}
static const step_t TRIANGULAR_LOOKUP_TABLE[] = {
{% for i in 1..=grammar.max_num_of_rules() %}
{%- for j in 0..i -%}
{{ i - 1 }},
{%- endfor %}
{% endfor -%}
};
{% endif %}

/***** TERMINALS *****/

{% for (id, content) in grammar.terminals() -%}
static const unsigned char TERMINAL_{{ id }}[{{ content.len() }}] = {
    {% for byte in content -%}
        {{ "{:#02x}" | format(byte) }}
        {%- if !loop.last %},{% endif %}
    {%- endfor %}
};
{% endfor %}
