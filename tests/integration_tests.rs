use std::fs;
use std::process::Command;

/// Kører en `.ms`-fil og sammenligner output (uden [trace]-linjer) med `.expected`-fil.
fn run_test(fixture_navn: &str) {
    let binary = env!("CARGO_BIN_EXE_mit-sprog");
    let ms_sti = format!("tests/fixtures/{}.ms", fixture_navn);
    let expected_sti = format!("tests/fixtures/{}.expected", fixture_navn);

    let forventet = fs::read_to_string(&expected_sti)
        .unwrap_or_else(|_| panic!("Mangler forventet output: {}", expected_sti));

    let output = Command::new(binary)
        .arg(&ms_sti)
        .output()
        .unwrap_or_else(|_| panic!("Kunne ikke køre mit-sprog med '{}'", ms_sti));

    assert!(
        output.status.success(),
        "Programmet '{}' fejlede uventet:\n{}",
        fixture_navn,
        String::from_utf8_lossy(&output.stderr)
    );

    // Filtrer [trace]-linjer fra — tests er robuste over for trace-ændringer
    let stdout = String::from_utf8_lossy(&output.stdout);
    let faktisk: String = stdout
        .lines()
        .filter(|l| !l.starts_with("[trace]"))
        .collect::<Vec<_>>()
        .join("\n");

    assert_eq!(
        faktisk.trim_end(),
        forventet.trim_end(),
        "\nTest '{}' fejlede!\n\nForventet:\n{}\n\nFaktisk:\n{}",
        fixture_navn,
        forventet.trim_end(),
        faktisk.trim_end()
    );
}

/// Kører en `.ms`-fil og verificerer at den fejler (exit code != 0).
fn run_fejl_test(fixture_navn: &str) {
    let binary = env!("CARGO_BIN_EXE_mit-sprog");
    let ms_sti = format!("tests/fixtures/{}.ms", fixture_navn);

    let output = Command::new(binary)
        .arg(&ms_sti)
        .output()
        .unwrap_or_else(|_| panic!("Kunne ikke køre mit-sprog med '{}'", ms_sti));

    assert!(
        !output.status.success(),
        "Test '{}' forventede en fejl, men programmet lykkedes",
        fixture_navn
    );
}

#[test] fn test_hej()              { run_test("hej"); }
#[test] fn test_variabler()        { run_test("variabler"); }
#[test] fn test_aritmetik()        { run_test("aritmetik"); }
#[test] fn test_sammenligning()    { run_test("sammenligning"); }
#[test] fn test_hvis()             { run_test("hvis"); }
#[test] fn test_gentag()           { run_test("gentag"); }
#[test] fn test_mens()             { run_test("mens"); }
#[test] fn test_funktioner()       { run_test("funktioner"); }
#[test] fn test_funktioner2()      { run_test("funktioner2"); }
#[test] fn test_parenteser()       { run_test("parenteser"); }
#[test] fn test_fejl_ukendt_var()  { run_fejl_test("fejl_ukendt_var"); }
#[test] fn test_fejl_syntaks()     { run_fejl_test("fejl_syntaks"); }
