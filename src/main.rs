use std::io::{self, Read};
mod ast;
mod parser;
mod render;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let prog = args[0].clone();

    let mut opts = getopts::Options::new();
    opts.optopt("i", "input", "When set, input will be read from the given file, otherwise input will be read from stdin.", "FILE");
    opts.optopt("o", "output", "When set, output will be written to the given file, otherwise output will be written to stdout.", "FILE");
    opts.optflag("h", "help", "Print this help menu.");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(err) => print_usage_fatal(&prog, opts),
    };

    if matches.opt_present("h") {
        print_usage(&prog, opts);
        return;
    }

    let input_file = matches.opt_str("i");
    let output_file = matches.opt_str("o");

    // Ensure that no positional arguments are set.
    if !matches.free.is_empty() {
        print_usage_fatal(&prog, opts);
    }

    let input = match input_file {
        Some(s) => {
            std::fs::read_to_string(s).unwrap()
        },
        None => {
            let mut buf = String::new();
            io::stdin().read_to_string(&mut buf).unwrap();
            buf
        }
    };

    println!("f: {}", input);

}

fn print_usage(prog: &str, opts: getopts::Options) {
    let brief = format!("Usage: {} [options]", prog);
    print!("{}", opts.usage(&brief));
}

fn print_usage_fatal(prog: &str, opts: getopts::Options) -> ! {
    let brief = format!("Usage: {} [options]", prog);
    eprint!("{}", opts.usage(&brief));
    std::process::exit(1);
}