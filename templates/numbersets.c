{% for (id, numberset) in grammar.numbersets() %}
{%- if numberset.set().len() <= 1 %}
{%- let range = numberset.set().iter().next().unwrap() %}
static inline void _numberset_{{ id }} (unsigned char* output) {
    uint64_t value = {{ range.start() }}ULL + (internal_random() % ({{ range.end() }}ULL - {{ range.start() }}ULL + 1));
    __builtin_memcpy(output, (unsigned char*) &value, sizeof({{ numberset.typ().c_type() }}));
}
{%- else %}
static void _numberset_{{ id }} (unsigned char* output) {
    uint64_t value;
    
    switch (LINEAR_RANDOM({{ numberset.set().len() }})) {
        {%- for (i, range) in numberset.set().iter().enumerate() %}
        case {{ i }}: {
            value = {{ range.start() }}ULL + (internal_random() % ({{ range.end() }}ULL - {{ range.start() }}ULL + 1));
            break;
        }
        {%- endfor %}
        default: {
            __builtin_unreachable();
        }
    }
    
    __builtin_memcpy(output, (unsigned char*) &value, sizeof({{ numberset.typ().c_type() }}));
}
{%- endif %}
{%- endfor %}
