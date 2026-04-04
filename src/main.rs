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

    // Fortolk argumenter
    let flags: Vec<&str> = args[1..].iter().map(|s| s.as_str()).collect();

    if flags.contains(&"--version") || flags.contains(&"-v") {
        println!("mit-sprog v{}", env!("CARGO_PKG_VERSION"));
        return;
    }

    let filnavn = flags.iter().find(|f| !f.starts_with("--")).copied().unwrap_or_else(|| {
        eprintln!("mit-sprog v{} — et dansk programmeringssprog", env!("CARGO_PKG_VERSION"));
        eprintln!();
        eprintln!("Brug: mit-sprog <fil.ms> [--kompiler] [--behold-c]");
        eprintln!("  Uden --kompiler:  kører programmet direkte (med [trace]-forklaringer)");
        eprintln!("  Med --kompiler:   oversætter til C og bygger mit-sprog.exe via gcc");
        eprintln!("  Med --behold-c:   beholder den genererede .c-fil (kræver --kompiler)");
        eprintln!("  --version / -v:   vis versionsnummer");
        process::exit(1);
    }).to_string();
    let kompiler_tilstand = flags.contains(&"--kompiler");
    let behold_c = flags.contains(&"--behold-c");

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
        match generator.kompiler(&saetninger, &filnavn, behold_c) {
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

