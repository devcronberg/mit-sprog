use crate::ast::{Operator, Saetning, Type, Udtryk};
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

/// Kodegenerator — oversætter AST til C-kode (transpilering)
pub struct Kodegenerator;

impl Kodegenerator {
    pub fn ny() -> Self {
        Kodegenerator
    }

    /// Genererer et komplet C-program som en streng
    pub fn generer(&self, saetninger: &[Saetning]) -> String {
        let mut linjer: Vec<String> = Vec::new();

        // Opbyg funktionstype-map til brug i udtryk og kald
        let mut fun_typer: HashMap<String, Option<Type>> = HashMap::new();
        for s in saetninger {
            if let Saetning::FunktionDef { navn, returtype, .. } = s {
                fun_typer.insert(navn.clone(), returtype.clone());
            }
        }

        linjer.push("#include <stdio.h>".to_string());
        linjer.push("#include <string.h>".to_string());
        linjer.push("#include <stdlib.h>".to_string());
        linjer.push("#include <windows.h>".to_string());
        // Hjælpefunktion til strengsammenkædning
        linjer.push("static char* _ms_strjoin(const char* a, const char* b) {".to_string());
        linjer.push("    size_t la = strlen(a), lb = strlen(b);".to_string());
        linjer.push("    char* r = (char*)malloc(la + lb + 1);".to_string());
        linjer.push("    memcpy(r, a, la); memcpy(r + la, b, lb + 1); return r;".to_string());
        linjer.push("}".to_string());

        // Generer C-funktioner før main
        for s in saetninger {
            if let Saetning::FunktionDef { navn, parametre, returtype, krop } = s {
                linjer.push(Self::generer_funktion(navn, parametre, returtype, krop, &fun_typer));
            }
        }

        linjer.push("int main(void) {".to_string());
        // Sæt konsollen til UTF-8 så danske bogstaver vises korrekt
        linjer.push("    SetConsoleOutputCP(65001);".to_string());

        let mut type_map: HashMap<String, Type> = HashMap::new();
        for s in saetninger {
            if !matches!(s, Saetning::FunktionDef { .. }) {
                linjer.push(Self::saetning_til_c(s, &mut type_map, &fun_typer));
            }
        }

        linjer.push("    return 0;".to_string());
        linjer.push("}".to_string());

        linjer.join("\n")
    }

    fn generer_funktion(
        navn: &str,
        parametre: &[(String, Type)],
        returtype: &Option<Type>,
        krop: &[Saetning],
        fun_typer: &HashMap<String, Option<Type>>,
    ) -> String {
        let retur_c = match returtype {
            None                    => "void",
            Some(Type::Nummer)      => "double",
            Some(Type::Streng)      => "char*",
            Some(Type::SandFalsk)   => "int",
        };
        let param_str = parametre.iter()
            .map(|(n, t)| match t {
                Type::Nummer    => format!("double {}", n),
                Type::Streng    => format!("char* {}", n),
                Type::SandFalsk => format!("int {}", n),
            })
            .collect::<Vec<_>>()
            .join(", ");

        let mut type_map: HashMap<String, Type> = parametre.iter()
            .map(|(n, t)| (n.clone(), t.clone()))
            .collect();

        let mut linjer = Vec::new();
        linjer.push(format!("{} {}({}) {{", retur_c, navn, param_str));
        for s in krop {
            linjer.push(Self::saetning_til_c(s, &mut type_map, fun_typer));
        }
        linjer.push("}".to_string());
        linjer.join("\n")
    }

    fn saetning_til_c(saetning: &Saetning, type_map: &mut HashMap<String, Type>, fun_typer: &HashMap<String, Option<Type>>) -> String {
        match saetning {
            Saetning::Skriv(udtryk) => {
                let (fmt, expr) = Self::udtryk_til_c(udtryk, type_map, fun_typer);
                format!("    printf(\"{}\\n\", {});", fmt, expr)
            }
            Saetning::Erklær { navn, type_, startværdi } => {
                type_map.insert(navn.clone(), type_.clone());
                let (c_type, default) = match type_ {
                    Type::Nummer    => ("double", "0.0".to_string()),
                    Type::Streng    => ("char*",  "\"\"".to_string()),
                    Type::SandFalsk => ("int",    "0".to_string()),
                };
                let init = match startværdi {
                    Some(udtryk) => {
                        let (_, expr) = Self::udtryk_til_c(udtryk, type_map, fun_typer);
                        expr
                    }
                    None => default,
                };
                format!("    {} {} = {};", c_type, navn, init)
            }
            Saetning::Sæt { navn, udtryk } => {
                let (_, expr) = Self::udtryk_til_c(udtryk, type_map, fun_typer);
                format!("    {} = {};", navn, expr)
            }
            Saetning::VisHukommelse => {
                // vis hukommelse er kun meningsfuldt i fortolker-tilstand
                "    /* vis hukommelse: ikke tilgængeligt i kompileret tilstand */".to_string()
            }
            Saetning::Hvis { betingelse, så_gren, ellers_gren } => {
                let (_, betingelse_c) = Self::udtryk_til_c(betingelse, type_map, fun_typer);
                let mut linjer = Vec::new();
                linjer.push(format!("    if ({}) {{", betingelse_c));
                for s in så_gren {
                    // Indryk så_gren med 4 ekstra mellemrum
                    let linje = Self::saetning_til_c(s, type_map, fun_typer);
                    linjer.push(format!("    {}", linje.trim_start()));
                }
                if let Some(ellers) = ellers_gren {
                    linjer.push("    } else {".to_string());
                    for s in ellers {
                        let linje = Self::saetning_til_c(s, type_map, fun_typer);
                        linjer.push(format!("    {}", linje.trim_start()));
                    }
                }
                linjer.push("    }".to_string());
                linjer.join("\n")
            }
            Saetning::Gentag { antal, krop } => {
                let (_, antal_c) = Self::udtryk_til_c(antal, type_map, fun_typer);
                let mut linjer = Vec::new();
                linjer.push(format!("    for (long long _i = 0; _i < (long long)({}); _i++) {{", antal_c));
                for s in krop {
                    let linje = Self::saetning_til_c(s, type_map, fun_typer);
                    linjer.push(format!("    {}", linje.trim_start()));
                }
                linjer.push("    }".to_string());
                linjer.join("\n")
            }
            Saetning::Mens { betingelse, krop } => {
                let (_, betingelse_c) = Self::udtryk_til_c(betingelse, type_map, fun_typer);
                let mut linjer = Vec::new();
                linjer.push(format!("    while ({}) {{", betingelse_c));
                for s in krop {
                    let linje = Self::saetning_til_c(s, type_map, fun_typer);
                    linjer.push(format!("    {}", linje.trim_start()));
                }
                linjer.push("    }".to_string());
                linjer.join("\n")
            }
            Saetning::FunktionDef { .. } => {
                // Genereres før main — intet at gøre her
                String::new()
            }
            Saetning::Returner(udtryk) => {
                match udtryk {
                    Some(u) => {
                        let (_, expr) = Self::udtryk_til_c(u, type_map, fun_typer);
                        format!("    return {};", expr)
                    }
                    None => "    return;".to_string(),
                }
            }
            Saetning::Udtryksaetning(udtryk) => {
                let (_, expr) = Self::udtryk_til_c(udtryk, type_map, fun_typer);
                format!("    {};", expr)
            }
        }
    }

    fn udtryk_til_c(udtryk: &Udtryk, type_map: &HashMap<String, Type>, fun_typer: &HashMap<String, Option<Type>>) -> (String, String) {
        match udtryk {
            Udtryk::Bogstav(s) => {
                let escaped = s.replace('\\', "\\\\").replace('"', "\\\"");
                ("%s".to_string(), format!("\"{}\"", escaped))
            }
            Udtryk::Tal(n) => {
                ("%g".to_string(), format!("{}", n))
            }
            Udtryk::BoolLiteral(b) => {
                let s = if *b { "\"sand\"" } else { "\"falsk\"" };
                ("%s".to_string(), s.to_string())
            }
            Udtryk::Variable(navn) => {
                match type_map.get(navn) {
                    Some(Type::Nummer)    => ("%g".to_string(), navn.clone()),
                    Some(Type::Streng)    => ("%s".to_string(), navn.clone()),
                    Some(Type::SandFalsk) => (
                        "%s".to_string(),
                        format!("({} ? \"sand\" : \"falsk\")", navn),
                    ),
                    None => ("%s".to_string(), navn.clone()),
                }
            }
            Udtryk::BinærOp { venstre, operator, højre } => {
                let (_, ve) = Self::udtryk_til_c(venstre, type_map, fun_typer);
                let (_, he) = Self::udtryk_til_c(højre, type_map, fun_typer);
                let venstre_er_streng = matches!(Self::udled_type(venstre, type_map, fun_typer), Some(Type::Streng));
                match operator {
                    // Strengsammenkædning
                    Operator::Plus if venstre_er_streng =>
                        ("%s".to_string(), format!("_ms_strjoin({}, {})", ve, he)),
                    // Strengsammenligning
                    Operator::ErLig  if venstre_er_streng =>
                        ("%s".to_string(), format!("(strcmp({}, {}) == 0 ? \"sand\" : \"falsk\")", ve, he)),
                    Operator::ErIkke if venstre_er_streng =>
                        ("%s".to_string(), format!("(strcmp({}, {}) != 0 ? \"sand\" : \"falsk\")", ve, he)),
                    // Talregning
                    Operator::Plus  => ("%g".to_string(), format!("({} + {})", ve, he)),
                    Operator::Minus => ("%g".to_string(), format!("({} - {})", ve, he)),
                    // Talsammenligning — returnerer "sand"/"falsk"
                    Operator::ErLig          => ("%s".to_string(), format!("(({} == {}) ? \"sand\" : \"falsk\")", ve, he)),
                    Operator::ErIkke         => ("%s".to_string(), format!("(({} != {}) ? \"sand\" : \"falsk\")", ve, he)),
                    Operator::Mindre         => ("%s".to_string(), format!("(({} <  {}) ? \"sand\" : \"falsk\")", ve, he)),
                    Operator::Større         => ("%s".to_string(), format!("(({} >  {}) ? \"sand\" : \"falsk\")", ve, he)),
                    Operator::MindreEllerLig => ("%s".to_string(), format!("(({} <= {}) ? \"sand\" : \"falsk\")", ve, he)),
                    Operator::StørreEllerLig => ("%s".to_string(), format!("(({} >= {}) ? \"sand\" : \"falsk\")", ve, he)),
                }
            }
            Udtryk::FunktionsKald { navn, argumenter } => {
                let arg_strs: Vec<String> = argumenter.iter()
                    .map(|a| Self::udtryk_til_c(a, type_map, fun_typer).1)
                    .collect();
                let kald = format!("{}({})", navn, arg_strs.join(", "));
                let fmt = match fun_typer.get(navn) {
                    Some(Some(Type::Nummer))    => "%g",
                    Some(Some(Type::Streng))    => "%s",
                    Some(Some(Type::SandFalsk)) => "%d",
                    _ => "%s",
                };
                (fmt.to_string(), kald)
            }
        }
    }

    fn udled_type(udtryk: &Udtryk, type_map: &HashMap<String, Type>, fun_typer: &HashMap<String, Option<Type>>) -> Option<Type> {
        match udtryk {
            Udtryk::Bogstav(_)     => Some(Type::Streng),
            Udtryk::Tal(_)         => Some(Type::Nummer),
            Udtryk::BoolLiteral(_) => Some(Type::SandFalsk),
            Udtryk::Variable(n)    => type_map.get(n).cloned(),
            Udtryk::BinærOp { venstre, .. } => Self::udled_type(venstre, type_map, fun_typer),
            Udtryk::FunktionsKald { navn, .. } => {
                fun_typer.get(navn).and_then(|t| t.clone())
            }
        }
    }

    /// Transpilerer til C og kalder gcc for at producere en .exe
    pub fn kompiler(
        &self,
        saetninger: &[Saetning],
        kilde_sti: &str,
        behold_c: bool,
    ) -> Result<String, String> {
        // Bestem filnavne: kilde bruges til .c, output hedder altid mit-sprog.exe
        let stamme = Path::new(kilde_sti)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("program");

        let c_fil = format!("{}.c", stamme);
        let exe_fil = "mit-sprog.exe".to_string();

        // Skriv C-koden til disk
        let c_kode = self.generer(saetninger);
        std::fs::write(&c_fil, &c_kode).map_err(|e| {
            format!("Kunne ikke skrive '{}': {}", c_fil, e)
        })?;

        println!("[transpiler] Genererede '{}'", c_fil);

        // Kald gcc
        let resultat = Command::new("gcc")
            .args([
                &c_fil,
                "-o",
                &exe_fil,
                "-static",   // ingen DLL-afhængigheder
                "-O2",
            ])
            .output()
            .map_err(|_| {
                "Kunne ikke finde 'gcc'. Installér MinGW: scoop install mingw".to_string()
            })?;

        if !resultat.status.success() {
            let fejl = String::from_utf8_lossy(&resultat.stderr);
            return Err(format!("gcc fejlede:\n{}", fejl));
        }

        // Ryd den midlertidige C-fil op (medmindre --behold-c er angivet)
        if !behold_c {
            let _ = std::fs::remove_file(&c_fil);
        } else {
            println!("[transpiler] Beholder '{}'", c_fil);
        }

        Ok(exe_fil)
    }
}


