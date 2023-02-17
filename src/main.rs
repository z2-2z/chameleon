#![feature(allow_internal_unstable)]

use termcolor;
use termcolor::WriteColor;
use std::io::Write;
use std::path::Path;
use clap::Parser;

mod grammar;
mod frontend;
mod backend;

#[derive(clap::Parser)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    #[clap(long, action, default_value_t = false)]
    forbid_cycles: bool,
    
    #[clap(long, action, default_value_t = false)]
    print_stats: bool,
    
    #[clap(long, action, default_value_t = false)]
    bench: bool,
    
    #[clap(short = 'o', value_parser)]
    outfile: Option<String>,
    
    #[clap(long, value_parser, default_value = "")]
    prefix: String,
    
    #[clap(value_parser)]
    grammar: String,
}

/// Given a number `n`, return how many decimal digits are
/// needed to represent this number
fn get_decimal_length(mut n: usize) -> usize {
    let mut len = 0;
    
    while n > 0 {
        n /= 10;
        len += 1;
    }
    
    len
}

/// Given an error location denoted by `lineno` and `col` print a context of the
/// file that may be helpful for debugging.
/// The context includes 3 lines around the invalid line.
fn print_line_context(stream: &mut termcolor::StandardStream, view: &frontend::SourceView, lineno: usize, col: usize, len: usize) -> Result<(), std::io::Error> {
    let line_length = std::cmp::max(
        get_decimal_length(lineno - 1),
        std::cmp::max(
            get_decimal_length(lineno),
            get_decimal_length(lineno + 1),
        )
    );
    let mut dimmed = termcolor::ColorSpec::new();
    dimmed.set_bg(None);
    dimmed.set_dimmed(true);
    let mut slim_red = termcolor::ColorSpec::new();
    slim_red.set_bg(None);
    slim_red.set_bold(false);
    slim_red.set_fg(Some(termcolor::Color::Red));
    
    if lineno > 1 {
        if let Some(line) = view.get_line(lineno - 1) {
            write!(stream, " {:width$} | ", lineno - 1, width = line_length)?;
            stream.set_color(&dimmed)?;
            writeln!(stream, "{}", line)?;
            stream.reset()?;
        }
    }
    
    if let Some(line) = view.get_line(lineno) {
        writeln!(stream, " {:width$} | {}", lineno, line, width = line_length)?;
        
        // Mark the affected columns
        write!(stream, " {0:width$} | {0:skip_cols$}", "",  width = line_length, skip_cols = col - 1)?;
        stream.set_color(&slim_red)?;
        writeln!(stream, "{0:^<input_len$}", "", input_len = len)?;
        stream.reset()?;
    } else {
        writeln!(stream, "<could not get line info>")?;
    }
    
    if let Some(line) = view.get_line(lineno + 1) {
        write!(stream, " {:width$} | ", lineno + 1, width = line_length)?;
        stream.set_color(&dimmed)?;
        writeln!(stream, "{}", line)?;
        stream.reset()?;
    }
    
    Ok(())
}

/// Print a neat error message for the given lexer error
fn print_lexing_error(view: &frontend::SourceView, error: &frontend::LexerError) -> Result<(), std::io::Error> {
    let mut bold_red = termcolor::ColorSpec::new();
    bold_red.set_bg(None);
    bold_red.set_bold(true);
    bold_red.set_fg(Some(termcolor::Color::Red));
    let mut slim_red = termcolor::ColorSpec::new();
    slim_red.set_bg(None);
    slim_red.set_bold(false);
    slim_red.set_fg(Some(termcolor::Color::Red));
    let mut stream = termcolor::StandardStream::stderr(termcolor::ColorChoice::Auto);
    
    stream.set_color(&slim_red)?;
    stream.write_all(b" ----=== ")?;
    stream.set_color(&bold_red)?;
    stream.write_all(b"Parsing Error")?;
    stream.set_color(&slim_red)?;
    stream.write_all(b" ===----")?;
    stream.reset()?;
    writeln!(&mut stream, "")?;
    
    match error {
        frontend::LexerError::EOF(message) => {
            writeln!(&mut stream, "Hit an unexpected EOF: {}", message)?;
        },
        frontend::LexerError::ExpectedIdentifier(pos) => {
            let (line, col) = view.lineinfo(*pos);
            writeln!(&mut stream, "In line {} column {}: Expected an identifier", line, col)?;
            print_line_context(&mut stream, view, line, col, 1)?;
        },
        frontend::LexerError::MissingWhitespace(pos) => {
            let (line, col) = view.lineinfo(*pos);
            writeln!(&mut stream, "In line {} column {}: A whitespace is missing", line, col)?;
            print_line_context(&mut stream, view, line, col, 1)?;
        },
        frontend::LexerError::ExpectedChar(pos, c) => {
            let (line, col) = view.lineinfo(*pos);
            writeln!(&mut stream, "In line {} column {}: Expected the character '{}'", line, col, c)?;
            print_line_context(&mut stream, view, line, col, 1)?;
        },
        frontend::LexerError::ExpectedKeyword(pos, keyword) =>  {
            let (line, col) = view.lineinfo(*pos);
            writeln!(&mut stream, "In line {} column {}: Expected the keyword '{}'", line, col, keyword)?;
            print_line_context(&mut stream, view, line, col, 1)?;
        },
        frontend::LexerError::InvalidNumber(pos) => {
            let (line, col) = view.lineinfo(*pos);
            writeln!(&mut stream, "In line {} column {}: Invalid number", line, col)?;
            print_line_context(&mut stream, view, line, col, 1)?;
        },
        frontend::LexerError::InvalidCharLiteral(pos) => {
            let (line, col) = view.lineinfo(*pos);
            writeln!(&mut stream, "In line {} column {}: Invalid character constant", line, col)?;
            print_line_context(&mut stream, view, line, col, 1)?;
        },
        frontend::LexerError::ExpectedLiteral(pos, literals) => {
            let (line, col) = view.lineinfo(*pos);
            writeln!(&mut stream, "In line {} column {}: Expected literal '{}'", line, col, literals)?;
            print_line_context(&mut stream, view, line, col, 1)?;
        },
    }
    
    writeln!(&mut stream, "")?;
    Ok(())
}

/// Print a neat error message for a given parser error
fn print_parsing_error(view: &frontend::SourceView, error: &frontend::ParserError) -> Result<(), std::io::Error> {
    let mut bold_red = termcolor::ColorSpec::new();
    bold_red.set_bg(None);
    bold_red.set_bold(true);
    bold_red.set_fg(Some(termcolor::Color::Red));
    let mut slim_red = termcolor::ColorSpec::new();
    slim_red.set_bg(None);
    slim_red.set_bold(false);
    slim_red.set_fg(Some(termcolor::Color::Red));
    let mut stream = termcolor::StandardStream::stderr(termcolor::ColorChoice::Auto);
    
    stream.set_color(&slim_red)?;
    stream.write_all(b" ----=== ")?;
    stream.set_color(&bold_red)?;
    stream.write_all(b"Parsing Error")?;
    stream.set_color(&slim_red)?;
    stream.write_all(b" ===----")?;
    stream.reset()?;
    writeln!(&mut stream, "")?;
    
    match error {
        frontend::ParserError::UnknownOptionValue(value) => {
            let (line, col) = view.lineinfo(value.start);
            writeln!(&mut stream, "In line {} column {}: Unknown option value", line, col)?;
            print_line_context(&mut stream, view, line, col, value.len())?;
        },
        frontend::ParserError::UnknownOptionName(name) => {
            let (line, col) = view.lineinfo(name.start);
            writeln!(&mut stream, "In line {} column {}: Unknown option name", line, col)?;
            print_line_context(&mut stream, view, line, col, name.len())?;
        },
        frontend::ParserError::DuplicateContainerName(name) => {
            let (line, col) = view.lineinfo(name.start);
            writeln!(&mut stream, "In line {} column {}: Name '{}' already exists", line, col, view.range(&name))?;
            print_line_context(&mut stream, view, line, col, name.len())?;
        },
        frontend::ParserError::EOF(message) => {
            writeln!(&mut stream, "Hit an unexpected EOF: {}", message)?;
        },
        frontend::ParserError::UnexpectedToken(pos, message) => {
            if let Some(pos) = pos {
                let (line, col) = view.lineinfo(*pos);
                writeln!(&mut stream, "In line {} column {}: {}", line, col, message)?;
                print_line_context(&mut stream, view, line, col, 1)?;
            } else {
                writeln!(&mut stream, "Invalid token sequence by lexer: {}", message)?;
                writeln!(&mut stream, "Unable to provide more info")?;
            }
        },
        frontend::ParserError::InvalidKeyword(range, message) => {
            let (line, col) = view.lineinfo(range.start);
            writeln!(&mut stream, "In line {} column {}: {}", line, col, message)?;
            print_line_context(&mut stream, view, line, col, range.len())?;
        },
        frontend::ParserError::InvalidNumber(base, num) => {
            let (line, col) = view.lineinfo(num.start);
            writeln!(&mut stream, "In line {} column {}: Invalid number of base {} given the type of the variable", line, col, base)?;
            print_line_context(&mut stream, view, line, col, num.len())?;
        },
        frontend::ParserError::InvalidRange(range) => {
            let (line, col) = view.lineinfo(range.start);
            writeln!(&mut stream, "In line {} column {}: Invalid bounds in range", line, col)?;
            print_line_context(&mut stream, view, line, col, range.len())?;
        },
        frontend::ParserError::CharacterNotAllowed(ch) => {
            let (line, col) = view.lineinfo(ch.start);
            writeln!(&mut stream, "In line {} column {}: Char literals are not allowed here", line, col)?;
            print_line_context(&mut stream, view, line, col, ch.len())?;
        },
        frontend::ParserError::InvalidCharacter(ch) => {
            let (line, col) = view.lineinfo(ch.start);
            writeln!(&mut stream, "In line {} column {}: Invalid char literal", line, col)?;
            print_line_context(&mut stream, view, line, col, ch.len())?;
        },
        frontend::ParserError::InvalidNumberset(pos) => {
            let (line, col) = view.lineinfo(*pos);
            writeln!(&mut stream, "In line {} column {}: Invalid numberset", line, col)?;
            print_line_context(&mut stream, view, line, col, 1)?;
        },
        frontend::ParserError::InvalidTypeName(name) => {
            let (line, col) = view.lineinfo(name.start);
            writeln!(&mut stream, "In line {} column {}: Specified type and value don't match", line, col)?;
            print_line_context(&mut stream, view, line, col, name.len())?;
        },
        frontend::ParserError::InvalidString(string, message) => {
            let (line, col) = view.lineinfo(string.start);
            writeln!(&mut stream, "In line {} column {}: {}", line, col, message)?;
            print_line_context(&mut stream, view, line, col, string.len())?;
        },
        frontend::ParserError::NoRoot => {
            writeln!(&mut stream, "No root {0} was found. Name the {0} where generation shall start '{1}'.", frontend::keywords::CONTAINER, frontend::keywords::ROOT_CONTAINER)?;
        },
        frontend::ParserError::UnresolvedRef(reference) => {
            let (line, col) = view.lineinfo(reference.start);
            writeln!(&mut stream, "In line {} column {}: Couldn't find a struct with the given name", line, col)?;
            print_line_context(&mut stream, view, line, col, reference.len())?;
        },
        frontend::ParserError::EmptyBlock(block) => {
            let (line, col) = view.lineinfo(*block);
            writeln!(&mut stream, "In line {} column {}: Blocks without variables are not allowed", line, col)?;
            print_line_context(&mut stream, view, line, col, 1)?;
        },
        frontend::ParserError::IllegalContainerName(name) => {
            let (line, col) = view.lineinfo(name.start);
            writeln!(&mut stream, "In line {} column {}: Illegal name for {}", line, col, frontend::keywords::CONTAINER)?;
            print_line_context(&mut stream, view, line, col, name.len())?;
        },
        frontend::ParserError::NonLocalOption(name) => {
            let (line, col) = view.lineinfo(name.start);
            writeln!(&mut stream, "In line {} column {}: This option is only allowed on a global level", line, col)?;
            print_line_context(&mut stream, view, line, col, name.len())?;
        },
        frontend::ParserError::IllegalOptionValue(value) => {
            let (line, col) = view.lineinfo(value.start);
            writeln!(&mut stream, "In line {} column {}: Invalid option value", line, col)?;
            print_line_context(&mut stream, view, line, col, value.len())?;
        },
    }
    
    writeln!(&mut stream, "")?;
    Ok(())
}

fn print_dead_containers(view: &frontend::SourceView, containers: Vec<grammar::ContainerId>, grammar: &grammar::Grammar) -> Result<(), std::io::Error> {
    let mut bold_red = termcolor::ColorSpec::new();
    bold_red.set_bg(None);
    bold_red.set_bold(true);
    bold_red.set_fg(Some(termcolor::Color::Red));
    let mut slim_red = termcolor::ColorSpec::new();
    slim_red.set_bg(None);
    slim_red.set_bold(false);
    slim_red.set_fg(Some(termcolor::Color::Red));
    let mut stream = termcolor::StandardStream::stderr(termcolor::ColorChoice::Auto);
    
    stream.set_color(&slim_red)?;
    stream.write_all(b" ----=== ")?;
    stream.set_color(&bold_red)?;
    stream.write_all(b"Grammar Error")?;
    stream.set_color(&slim_red)?;
    stream.write_all(b" ===----")?;
    stream.reset()?;
    writeln!(&mut stream, "")?;
    
    writeln!(&mut stream, "The following containers are never being used:")?;
    
    for id in containers {
        let name = grammar.container(id).unwrap().name().unwrap();
        let (line, col) = view.lineinfo(name.start);
        if name.len() == 0 {
            writeln!(&mut stream, " - anonymous struct in line {} column {}", line, col)?;
        } else {
            writeln!(&mut stream, " - '{}' in line {} column {}", view.range(&name), line, col)?;
        }
        
    }
    
    Ok(())
}

fn print_cycle(view: &frontend::SourceView, cycle: (grammar::ContainerId, grammar::ContainerId), grammar: &grammar::Grammar) -> Result<(), std::io::Error> {
    let mut bold_red = termcolor::ColorSpec::new();
    bold_red.set_bg(None);
    bold_red.set_bold(true);
    bold_red.set_fg(Some(termcolor::Color::Red));
    let mut slim_red = termcolor::ColorSpec::new();
    slim_red.set_bg(None);
    slim_red.set_bold(false);
    slim_red.set_fg(Some(termcolor::Color::Red));
    let mut stream = termcolor::StandardStream::stderr(termcolor::ColorChoice::Auto);
    
    stream.set_color(&slim_red)?;
    stream.write_all(b" ----=== ")?;
    stream.set_color(&bold_red)?;
    stream.write_all(b"Grammar Error")?;
    stream.set_color(&slim_red)?;
    stream.write_all(b" ===----")?;
    stream.reset()?;
    writeln!(&mut stream, "")?;
    
    writeln!(&mut stream, "There exists a cycle between the two containers")?;
    
    let name = grammar.container(cycle.0).unwrap().name().unwrap();
    let (line, col) = view.lineinfo(name.start);
    if name.len() == 0 {
        writeln!(&mut stream, "1. anonymous struct in line {} column {}", line, col)?;
    } else {
        writeln!(&mut stream, "1. '{}' in line {} column {}", view.range(&name), line, col)?;
    }
    
    let name = grammar.container(cycle.1).unwrap().name().unwrap();
    let (line, col) = view.lineinfo(name.start);
    if name.len() == 0 {
        writeln!(&mut stream, "2. inline struct in line {} column {}", line, col)?;
    } else {
        writeln!(&mut stream, "2. '{}' in line {} column {}", view.range(&name), line, col)?;
    }
    
    writeln!(&mut stream, "")?;
    writeln!(&mut stream, "If you want to allow cycles, pass --allow-cycles")?;
    
    Ok(())
}

fn warning(msg: &str) -> Result<(), std::io::Error> {
    let mut yellow = termcolor::ColorSpec::new();
    yellow.set_bg(None);
    yellow.set_bold(true);
    yellow.set_fg(Some(termcolor::Color::Yellow));
    let mut stream = termcolor::StandardStream::stderr(termcolor::ColorChoice::Auto);
    stream.set_color(&yellow)?;
    write!(&mut stream, "Warning: ")?;
    stream.reset()?;
    writeln!(&mut stream, "{}", msg)?;
    Ok(())
}

fn verify_grammar(view: &frontend::SourceView, grammar: &grammar::Grammar, args: &Args) {
    let graph = frontend::graph::GrammarGraph::minimal_graph(grammar);
    
    if let Some(cycle) = graph.cycle() {
        let _ = print_cycle(view, cycle, grammar);
        std::process::exit(1);
    }
    
    let graph = frontend::graph::GrammarGraph::full_graph(grammar);    

    let dead_containers = graph.unreachable_containers();
    if !dead_containers.is_empty() {
        let _ = print_dead_containers(view, dead_containers, grammar);
        std::process::exit(1);
    }
    
    let cycle = graph.cycle();
    
    if args.forbid_cycles {
        if let Some(cycle) = &cycle {
            let _ = print_cycle(view, *cycle, grammar);
            std::process::exit(1);
        }
    }
    
    if args.print_stats {
        if cycle.is_some() {
            let _ = warning("Graph contains cycles so no stats will be printed");
        } else {
            print_stats(grammar);
        }
    }
}

fn print_stats(grammar: &grammar::Grammar) {    
    let stats = frontend::stats::GrammarStats::from_grammar(grammar);
    
    println!("Grammar stats:");
    
    print!("  - number of paths = ");
    if let Some(num_paths) = stats.num_paths() {
        println!("{}", num_paths);
    } else {
        println!("too many");
    }
    
    print!("  - largest input = ");
    if let Some(max_input_size) = stats.max_input_size() {
        println!("{} bytes", max_input_size);
    } else {
        println!("too big");
    }
    
    print!("  - smallest input = ");
    if let Some(min_input_size) = stats.min_input_size() {
        println!("{} bytes", min_input_size);
    } else {
        println!("too big");
    }
}

fn run_benchmark(outfile: &str) {
    let path = Path::new(outfile);
    let mut file = std::fs::File::create("/tmp/chm-bench.c").expect("Could not create benchmark file");
    
    write!(
        &mut file,
"
#include <stdio.h>
#include <stdlib.h>
#include <stddef.h>
#include <unistd.h>
#include <pthread.h>
#include <time.h>

#include \"{}.h\"

typedef struct thread_data {{
    // Input for thread
    size_t limit;
    unsigned char* buffer;
    size_t buffer_size;
    size_t seed;
    
    // Output of thread
    char success;
    time_t elapsed;
    size_t iterations;
}} thread_data;

void* generator_thread (void* arg) {{
    thread_data* data = (thread_data*) arg;
    size_t generated = 0;
    size_t iterations = 0;
    struct timespec start, end;
    
    data->success = 0;
    seed(data->seed);
    
    if (clock_gettime(CLOCK_MONOTONIC, &start) != 0) {{
        return NULL;
    }}
    
    while (generated < data->limit) {{
        generated += generate(data->buffer, data->buffer_size);
        iterations++;
    }}
    
    if (clock_gettime(CLOCK_MONOTONIC, &end) != 0) {{
        return NULL;
    }}
    
    data->success = 1;
    data->elapsed = end.tv_sec - start.tv_sec;
    data->iterations = iterations;
    
    return NULL;
}}

void print_results (thread_data* results, int threads) {{
    // Calculate total MiB/s
    double total_per_sec = 0.0;
    
    for (int i = 0; i < threads; ++i) {{
        total_per_sec += ((double) results[i].limit / (double) results[i].elapsed);
    }}
    
    // Calculate the mean from the mean size per iteration
    size_t mean_size = 0;
    
    for (int i = 0; i < threads; ++i) {{
        mean_size += (results[i].limit / results[i].iterations);
    }}
    
    mean_size /= threads;
    
    printf(\" -> threads: %d | total generated: %.4lf MiB/s | mean size: %lu\\n\", threads, total_per_sec / 1024.0 / 1024.0, mean_size);
}}

size_t GiB (size_t n) {{
    return n * 1024 * 1024 * 1024;
}}

int main (int argc, char** argv) {{
    int num_threads = 1;
    
    if (argc > 1) {{
        num_threads = atoi(argv[1]);
        
        if (num_threads <= 0) {{
            return 1;
        }}
    }}
    
    thread_data results[num_threads];
    pthread_t threads[num_threads];
    size_t limit = 1;
    time_t min_time = 10;
    
    for (int i = 0; i < num_threads; ++i) {{
        results[i].buffer_size = GiB(1);
        
        if (!(results[i].buffer = malloc(GiB(1)))) {{
            return 1;
        }}
    }}
    
    printf(\"Measuring time to %lu GiB:\\n\", limit);
    
    for (int j = 0; ; ++j) {{
        for (int i = 0; i < num_threads; ++i) {{
            results[i].limit = GiB(limit);
            results[i].seed = (size_t)argv + j * num_threads + i;
            
            if (pthread_create(&threads[i], NULL, generator_thread, (void*) &results[i]) != 0) {{
                return 1;
            }}
        }}
        
        for (int i = 0; i < num_threads; ++i) {{
            if (pthread_join(threads[i], NULL) != 0) {{
                return 1;
            }}
            
            if (!results[i].success) {{
                return 1;
            }}
        }}
        
        // Check if we need to adjust the limit
        char adjust = 0;
        
        for (int i = 0; i < num_threads; ++i) {{
            if (results[i].elapsed < min_time) {{
                adjust = 1;
                break;
            }}
        }}
        
        if (adjust) {{
            limit <<= 1;
            
            if (limit == 0) {{
                return 1;
            }}
            
            printf(\"One run took less than %ld seconds, readjusting limit to %lu GiB\\n\", min_time, limit);
            printf(\"Measuring time to %lu GiB:\\n\", limit);
        }} else {{
            print_results(results, num_threads);
        }}
    }}
}}
",
        path.file_name().unwrap().to_str().unwrap(),
    ).expect("Could not write to benchmark file");
    
    let include_dir = if let Some(parent) = path.parent() {
        let parent = parent.to_str().unwrap();
        if parent.len() == 0 {
            format!("-I.")
        } else {
            format!("-I{}", parent)
        }
    } else {
        format!("-I.")
    };
    
    let status = std::process::Command::new("gcc")
        .arg("-o")
        .arg("/tmp/chm-bench")
        .arg("-O3")
        .arg("-flto")
        .arg("-lpthread")
        .arg("/tmp/chm-bench.c")
        .arg(outfile)
        .arg(&include_dir)
        .arg("-DMULTITHREADING")
        .arg("-Wall")
        .arg("-Wextra")
        .arg("-Wpedantic")
        .arg("-Wno-unused-function")
        .arg("-Wno-unused-parameter")
        .arg("-Werror")
        .arg("-g")
        .spawn()
        .expect("Could not launch gcc")
        .wait()
        .unwrap();
    assert!(status.success(), "gcc returned error");
    let _ = std::process::Command::new("/tmp/chm-bench")
        .spawn()
        .expect("Could not launch benchmark")
        .wait();
}

fn main() {
    let mut args = Args::parse();
    
    if args.prefix.len() > 0 && !args.prefix.ends_with("_") {
        args.prefix.push('_');
    }
    
    let view = frontend::SourceView::from_file(&args.grammar);
    let mut lexer = frontend::Lexer::new(&view);
    
    let tokens = match lexer.lex() {
        Ok(tokens) => tokens,
        Err(error) => {
            if let Err(_) = print_lexing_error(&view, &error) {
                println!("{:?}", error);
            }
            std::process::exit(1);
        },
    };
    
    let mut parser = frontend::Parser::new(&view, &tokens);
    
    let grammar = match parser.parse() {
        Ok(grammar) => grammar,
        Err(error) => {
            if let Err(_) = print_parsing_error(&view, &error) {
                println!("{:?}", error);
            }
            std::process::exit(1);
        },
    };
    
    verify_grammar(&view, &grammar, &args);
    
    let mut did_action = false;
    
    if args.outfile.is_some() {
        backend::C::compile_grammar(&args, &grammar, &view);
        did_action = true;
    }
    
    if args.bench {
        if args.outfile.is_none() {
            args.outfile = Some("/tmp/chm-generator.c".to_string());
            backend::C::compile_grammar(&args, &grammar, &view);
        }
        run_benchmark(&args.outfile.unwrap());
        did_action = true;
    }
    
    if !did_action {
        let _ = warning("No action specified. Doing nothing.");
    }
}
