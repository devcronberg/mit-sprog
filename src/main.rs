mod ast;
mod evaluator;
mod kodegenerator;
mod lexer;
mod parser;

use std::env;
use std::fs;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    // Fortolk argumenter: mit-sprog <fil.ms> [--kompiler]
    let (filnavn, kompiler_tilstand) = match args.as_slice() {
        [_, fil] => (fil.clone(), false),
        [_, fil, flag] if flag == "--kompiler" => (fil.clone(), true),
        _ => {
            eprintln!("Brug: mit-sprog <fil.ms> [--kompiler]");
            eprintln!("  Uden --kompiler: kører programmet direkte");
            eprintln!("  Med --kompiler:  oversætter til C og bygger en .exe via gcc");
            process::exit(1);
        }
    };

    let kildekode = match fs::read_to_string(&filnavn) {
        Ok(indhold) => indhold,
        Err(e) => {
            eprintln!("Fejl: Kunne ikke læse filen '{}': {}", filnavn, e);
            process::exit(1);
        }
    };

    // Fase 1: Lexer
    let mut lekser = lexer::Lexer::ny(&kildekode);
    let tokens = match lekser.tokenisér() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Lekserfejl: {}", e);
            process::exit(1);
        }
    };

    // Fase 2: Parser
    let mut parser = parser::Parser::ny(tokens);
    let saetninger = match parser.parse() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Syntaksfejl: {}", e);
            process::exit(1);
        }
    };

    if kompiler_tilstand {
        // Fase 3b: Transpiler til C → gcc → .exe
        let generator = kodegenerator::Kodegenerator::ny();
        match generator.kompiler(&saetninger, &filnavn) {
            Ok(exe) => println!("[kompiler] Færdig: '{}'", exe),
            Err(e) => {
                eprintln!("Kompileringsfejl: {}", e);
                process::exit(1);
            }
        }
    } else {
        // Fase 3a: Fortolk direkte (standard)
        let mut evaluator = evaluator::Evaluator::ny();
        if let Err(e) = evaluator.kør(&saetninger) {
            eprintln!("Kørselsfejl: {}", e);
            process::exit(1);
        }
    }
}

