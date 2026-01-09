{% for set in grammar.rules() %}
// This is the mutation function for non-terminal '{{ grammar.nonterminal(set.nonterm().id()) }}'
{%- if set.rules().len() <= 1 %}
static size_t _generate_nonterm_{{ set.nonterm().id() }} (unsigned char* output, size_t output_length)  {
    {%- if set.has_no_symbols() %}
    (void) output;
    (void) output_length;
    
    return 0;
    {%- else %}
    {%- if set.has_nonterms() %}
    size_t r;
    {%- endif %}
    unsigned char* original_output = output;
    
    {%- let rule = set.rules().iter().next().unwrap() %}
    {% for symbol in rule %}
    {%- match symbol %}
    {%- when crate::translator::Symbol::Terminal(term) %}
    {%- match term %}
    {%- when crate::translator::Terminal::Numberset(id) %}
    {%- let numberset = grammar.numberset(**id) %}
    if (UNLIKELY(sizeof({{ numberset.typ().c_type() }}) > output_length)) {
        return output_length;
    }
    _mutate_numberset_{{ id }}(output);
    output += sizeof({{ numberset.typ().c_type() }});
    {%- if !loop.last %}
    output_length -= sizeof({{ numberset.typ().c_type() }});
    {%- endif %}
    {%- when crate::translator::Terminal::Bytes(id) %}
    if (UNLIKELY(sizeof(TERMINAL_{{ id }}) > output_length)) {
        return output_length;
    }
    __builtin_memcpy(output, TERMINAL_{{ id }}, sizeof(TERMINAL_{{ id }}));
    output += sizeof(TERMINAL_{{ id }});
    {%- if !loop.last %}
    output_length -= sizeof(TERMINAL_{{ id }});
    {%- endif %}
    {%- endmatch %}
    {%- when crate::translator::Symbol::NonTerminal(nonterm) %}
    r = _generate_nonterm_{{ nonterm.id() }}(output, output_length);
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
static size_t _generate_nonterm_{{ set.nonterm().id() }} (unsigned char* output, size_t output_length)  {
    {%- if set.has_no_symbols() %}
    (void) output_length;
    {%- endif %}
    {%- if set.has_nonterms() %}
    size_t r;
    {%- endif %}
    unsigned char* original_output = output;
    
    switch (internal_random() % {{ set.rules().len() }}) {
        {%- for (i, rule) in set.rules().iter().enumerate() %}
        case {{ i }}: {
            {%- for symbol in rule %}
            {%- match symbol %}
            {%- when crate::translator::Symbol::Terminal(term) %}
            {%- match term %}
            {%- when crate::translator::Terminal::Numberset(id) %}
            {%- let numberset = grammar.numberset(**id) %}
            if (UNLIKELY(sizeof({{ numberset.typ().c_type() }}) > output_length)) {
                return output_length;
            }
            _mutate_numberset_{{ id }}(output);
            output += sizeof({{ numberset.typ().c_type() }});
            {%- if !loop.last %}
            output_length -= sizeof({{ numberset.typ().c_type() }});
            {%- endif %}
            {%- when crate::translator::Terminal::Bytes(id) %}
            if (UNLIKELY(sizeof(TERMINAL_{{ id }}) > output_length)) {
                return output_length;
            }
            __builtin_memcpy(output, TERMINAL_{{ id }}, sizeof(TERMINAL_{{ id }}));
            output += sizeof(TERMINAL_{{ id }});
            {%- if !loop.last %}
            output_length -= sizeof(TERMINAL_{{ id }});
            {%- endif %}
            {%- endmatch %}
            {%- when crate::translator::Symbol::NonTerminal(nonterm) %}
            r = _generate_nonterm_{{ nonterm.id() }}(output, output_length);
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
