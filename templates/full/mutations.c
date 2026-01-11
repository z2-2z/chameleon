{% for set in grammar.rules() %}
// This is the mutation function for non-terminal '{{ grammar.nonterminal(set.nonterm().id()) }}'
{%- if set.rules().len() <= 1 %}
static size_t _mutate_nonterm_{{ set.nonterm().id() }} (unsigned int* steps, const size_t length, const size_t capacity, size_t* step, unsigned char* output, size_t output_length)  {
    {%- if set.has_no_symbols() %}
    (void) steps;
    (void) length;
    (void) output;
    (void) output_length;
    size_t s = *step;
    
    if (LIKELY(s < capacity)) {
        *step = s + 1;
    }
    
    return 0;
    {%- else %}
    (void) steps;
    {%- if set.has_nonterms() %}
    size_t r;
    {%- endif %}
    {%- if set.has_terms() %}
    unsigned int mutate;
    {%- endif %}
    unsigned char* original_output = output;
    size_t s = *step;
    
    if (UNLIKELY(s >= capacity)) {
        return 0;
    }
    *step = s + 1;
    {%- if set.has_terms() %}
    mutate = (s >= length);
    {% endif %}
    {%- let rule = set.rules().iter().next().unwrap() %}
    {% for symbol in rule %}
    {%- match symbol %}
    {%- when crate::translator::Symbol::Terminal(term) %}
    {%- match term %}
    {%- when crate::translator::Terminal::Numberset(id) %}
    {%- let numberset = grammar.numberset(**id) %}
    if (mutate) {
        if (UNLIKELY(sizeof({{ numberset.typ().c_type() }}) > output_length)) {
            return output_length;
        }
        _numberset_{{ id }}(output);
    }
    output += sizeof({{ numberset.typ().c_type() }});
    {%- if !loop.last %}
    output_length -= sizeof({{ numberset.typ().c_type() }});
    {%- endif %}
    {%- when crate::translator::Terminal::Bytes(id) %}
    if (mutate) {
        if (UNLIKELY(sizeof(TERMINAL_{{ id }}) > output_length)) {
            return output_length;
        }
        __builtin_memcpy(output, TERMINAL_{{ id }}, sizeof(TERMINAL_{{ id }}));
    }
    output += sizeof(TERMINAL_{{ id }});
    {%- if !loop.last %}
    output_length -= sizeof(TERMINAL_{{ id }});
    {%- endif %}
    {%- endmatch %}
    {%- when crate::translator::Symbol::NonTerminal(nonterm) %}
    r = _mutate_nonterm_{{ nonterm.id() }}(steps, length, capacity, step, output, output_length);
    output += r;
    {%- if !loop.last %}
    output_length -= r;
    {%- endif %}
    {%- endmatch %}
    {%- endfor %}
    
    return (size_t) (output - original_output);
    {%- endif %}
}
{%- else %}
static size_t _mutate_nonterm_{{ set.nonterm().id() }} (unsigned int* steps, const size_t length, const size_t capacity, size_t* step, unsigned char* output, size_t output_length)  {
    {%- if set.has_no_symbols() %}
    (void) output_length;
    {%- endif %}
    {%- if set.has_nonterms() %}
    size_t r;
    {%- endif %}
    unsigned int mutate, rule;
    unsigned char* original_output = output;
    size_t s = *step;
    
    if (UNLIKELY(s >= capacity)) {
        return 0;
    }
    *step = s + 1;
    
    mutate = (s >= length);
    
    if (mutate) {
        {% if set.is_triangular() -%}
        rule = TRIANGULAR_RANDOM({{ (set.rules().len() * (set.rules().len() + 1)) / 2 }});
        {%- else -%}
        rule = internal_random() % {{ set.rules().len() }};
        {%- endif %}
        steps[s] = rule;
    } else {
        rule = steps[s];
    }
    
    switch (rule) {
        {%- for (i, rule) in set.rules().iter().enumerate() %}
        case {{ i }}: {
            {%- for symbol in rule %}
            {%- match symbol %}
            {%- when crate::translator::Symbol::Terminal(term) %}
            {%- match term %}
            {%- when crate::translator::Terminal::Numberset(id) %}
            {%- let numberset = grammar.numberset(**id) %}
            if (mutate) {
                if (UNLIKELY(sizeof({{ numberset.typ().c_type() }}) > output_length)) {
                    return output_length;
                }
                _numberset_{{ id }}(output);
            }
            output += sizeof({{ numberset.typ().c_type() }});
            {%- if !loop.last %}
            output_length -= sizeof({{ numberset.typ().c_type() }});
            {%- endif %}
            {%- when crate::translator::Terminal::Bytes(id) %}
            if (mutate) {
                if (UNLIKELY(sizeof(TERMINAL_{{ id }}) > output_length)) {
                    return output_length;
                }
                __builtin_memcpy(output, TERMINAL_{{ id }}, sizeof(TERMINAL_{{ id }}));
            }
            output += sizeof(TERMINAL_{{ id }});
            {%- if !loop.last %}
            output_length -= sizeof(TERMINAL_{{ id }});
            {%- endif %}
            {%- endmatch %}
            {%- when crate::translator::Symbol::NonTerminal(nonterm) %}
            r = _mutate_nonterm_{{ nonterm.id() }}(steps, length, capacity, step, output, output_length);
            output += r;
            {%- if !loop.last %}
            output_length -= r;
            {%- endif %}
            {%- endmatch %}
            {%- endfor %}
            break;
        }
        {% endfor %}
        default: {
            __builtin_unreachable();
        }
    }
    
    return (size_t) (output - original_output);
}
{%- endif %}
{% endfor %}
