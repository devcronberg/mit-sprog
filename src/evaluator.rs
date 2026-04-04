use crate::ast::{Operator, Saetning, Type, Udtryk};
use std::collections::HashMap;

/// En runtime-værdi i sproget
#[derive(Debug, Clone)]
pub enum Værdi {
    Nummer(f64),
    Streng(String),
    SandFalsk(bool),
}

impl Værdi {
    fn vis(&self) -> String {
        match self {
            Værdi::Nummer(n) => {
                if n.fract() == 0.0 && n.abs() < 1e15 {
                    format!("{}", *n as i64)
                } else {
                    format!("{}", n)
                }
            }
            Værdi::Streng(s) => s.clone(),
            Værdi::SandFalsk(b) => if *b { "sand".to_string() } else { "falsk".to_string() },
        }
    }

    fn type_navn(&self) -> &'static str {
        match self {
            Værdi::Nummer(_)    => "nummer",
            Værdi::Streng(_)    => "streng",
            Værdi::SandFalsk(_) => "sandFalsk",
        }
    }
}

/// En funktion registreret i sproget
#[derive(Clone)]
struct FunktionInfo {
    parametre: Vec<(String, Type)>,
    returtype: Option<Type>,
    krop: Vec<Saetning>,
}

/// Evaluator — udfører et program og forklarer hvert trin
pub struct Evaluator {
    hukommelse: HashMap<String, Værdi>,
    functioner: HashMap<String, FunktionInfo>,
    return_signal: Option<Værdi>,
}

impl Evaluator {
    pub fn ny() -> Self {
        Evaluator {
            hukommelse: HashMap::new(),
            functioner: HashMap::new(),
            return_signal: None,
        }
    }

    fn evaluer_udtryk(&mut self, udtryk: &Udtryk) -> Result<Værdi, String> {
        match udtryk {
            Udtryk::Bogstav(s)      => Ok(Værdi::Streng(s.clone())),
            Udtryk::Tal(n)          => Ok(Værdi::Nummer(*n)),
            Udtryk::BoolLiteral(b)  => Ok(Værdi::SandFalsk(*b)),
            Udtryk::Variable(navn)  => {
                self.hukommelse.get(navn).cloned().ok_or_else(|| {
                    format!("Ukendt variabel '{}' — er den erklæret med 'erklær'?", navn)
                })
            }
            Udtryk::BinærOp { venstre, operator, højre } => {
                let v = self.evaluer_udtryk(venstre)?;
                let h = self.evaluer_udtryk(højre)?;
                match (&v, operator, &h) {
                    // Tal: regning
                    (Værdi::Nummer(a), Operator::Plus,  Værdi::Nummer(b)) => Ok(Værdi::Nummer(a + b)),
                    (Værdi::Nummer(a), Operator::Minus, Værdi::Nummer(b)) => Ok(Værdi::Nummer(a - b)),
                    // Tal: sammenligning
                    (Værdi::Nummer(a), Operator::ErLig,          Værdi::Nummer(b)) => Ok(Værdi::SandFalsk(a == b)),
                    (Værdi::Nummer(a), Operator::ErIkke,         Værdi::Nummer(b)) => Ok(Værdi::SandFalsk(a != b)),
                    (Værdi::Nummer(a), Operator::Mindre,         Værdi::Nummer(b)) => Ok(Værdi::SandFalsk(a < b)),
                    (Værdi::Nummer(a), Operator::Større,         Værdi::Nummer(b)) => Ok(Værdi::SandFalsk(a > b)),
                    (Værdi::Nummer(a), Operator::MindreEllerLig, Værdi::Nummer(b)) => Ok(Værdi::SandFalsk(a <= b)),
                    (Værdi::Nummer(a), Operator::StørreEllerLig, Værdi::Nummer(b)) => Ok(Værdi::SandFalsk(a >= b)),
                    // Streng: sammenkædning
                    (Værdi::Streng(a), Operator::Plus, Værdi::Streng(b)) => Ok(Værdi::Streng(a.clone() + b)),
                    // Streng: sammenligning
                    (Værdi::Streng(a), Operator::ErLig,  Værdi::Streng(b)) => Ok(Værdi::SandFalsk(a == b)),
                    (Værdi::Streng(a), Operator::ErIkke, Værdi::Streng(b)) => Ok(Værdi::SandFalsk(a != b)),
                    // Fejl
                    (Værdi::Streng(_), Operator::Minus, Værdi::Streng(_)) =>
                        Err("Typefejl: man kan ikke trække to strenge fra hinanden".to_string()),
                    _ => Err(format!(
                        "Typefejl: operatoren kan ikke bruges på '{}' og '{}'",
                        v.type_navn(), h.type_navn()
                    )),
                }
            }
            Udtryk::FunktionsKald { navn, argumenter } => {
                // Evaluér argumenter, opslå funktion og kald den
                let mut arg_værdier = Vec::new();
                for arg in argumenter {
                    arg_værdier.push(self.evaluer_udtryk(arg)?);
                }
                let info = self.functioner.get(navn).cloned().ok_or_else(|| {
                    format!("Ukendt funktion '{}' — er den erklæret med 'funktion'?", navn)
                })?;
                let kald_navn = navn.clone();
                self.kald_funktion(info, arg_værdier, &kald_navn)
            }
        }
    }

    /// Kalder en funktion med en ny scope og håndterer returværdi
    fn kald_funktion(&mut self, info: FunktionInfo, argumenter: Vec<Værdi>, navn: &str) -> Result<Værdi, String> {
        if argumenter.len() != info.parametre.len() {
            return Err(format!(
                "Funktion '{}' forventer {} {}, men fik {}",
                navn, info.parametre.len(),
                if info.parametre.len() == 1 { "argument" } else { "argumenter" },
                argumenter.len()
            ));
        }
        // Typetjek og byg trace-streng
        let mut param_trace = Vec::new();
        for ((p_navn, p_type), arg) in info.parametre.iter().zip(&argumenter) {
            let ok = matches!(
                (p_type, arg),
                (Type::Nummer,    Værdi::Nummer(_))    |
                (Type::Streng,    Værdi::Streng(_))    |
                (Type::SandFalsk, Værdi::SandFalsk(_))
            );
            if !ok {
                return Err(format!(
                    "Typefejl i kald af '{}': parameter '{}' er {}, men fik {}",
                    navn, p_navn, type_navn_fra_type(p_type), arg.type_navn()
                ));
            }
            param_trace.push(format!("{} = {}", p_navn, arg.vis()));
        }
        println!("[trace] Kalder funktion '{}' ({})", navn, param_trace.join(", "));

        // Gem ydre scope og nulstil returnsignal
        let gammelt_hukommelse = std::mem::replace(&mut self.hukommelse, HashMap::new());
        let gammelt_return = self.return_signal.take();

        // Bind parametre til lokal scope
        for ((p_navn, _), arg) in info.parametre.iter().zip(argumenter) {
            self.hukommelse.insert(p_navn.clone(), arg);
        }

        // Udfør funktionskroppen
        self.kør(&info.krop)?;

        // Hent returværdi og gendan ydre scope
        let retur = self.return_signal.take();
        self.hukommelse = gammelt_hukommelse;
        self.return_signal = gammelt_return;

        let v = match retur {
            Some(v) => {
                println!("[trace] Funktion '{}' returnerede: {}", navn, v.vis());
                v
            }
            None => Værdi::SandFalsk(false), // void-funktion giver dummy-værdi
        };
        Ok(v)
    }

    pub fn kør(&mut self, saetninger: &[Saetning]) -> Result<(), String> {
        // Første pas: registrer alle funktionsdefinitioner i denne blok
        for s in saetninger {
            if let Saetning::FunktionDef { navn, parametre, returtype, krop } = s {
                self.functioner.insert(navn.clone(), FunktionInfo {
                    parametre: parametre.clone(),
                    returtype: returtype.clone(),
                    krop: krop.clone(),
                });
            }
        }
        // Andet pas: udfør de øvrige sætninger
        for saetning in saetninger {
            if self.return_signal.is_some() { break; }
            if !matches!(saetning, Saetning::FunktionDef { .. }) {
                self.udfør(saetning)?;
            }
        }
        Ok(())
    }

    fn udfør(&mut self, saetning: &Saetning) -> Result<(), String> {
        match saetning {
            Saetning::Skriv(udtryk) => {
                let værdi = self.evaluer_udtryk(udtryk)?;
                println!("[trace] Skriver: {}", værdi.vis());
                println!("{}", værdi.vis());
                Ok(())
            }
            Saetning::Erklær { navn, type_, startværdi } => {
                let værdi = match startværdi {
                    Some(udtryk) => {
                        let v = self.evaluer_udtryk(udtryk)?;
                        // Typecheck
                        match (type_, &v) {
                            (Type::Nummer,    Værdi::Nummer(_))    => {}
                            (Type::Streng,    Værdi::Streng(_))    => {}
                            (Type::SandFalsk, Værdi::SandFalsk(_)) => {}
                            _ => return Err(format!(
                                "Typefejl: variablen '{}' er erklæret som '{}', men startværdien er af typen '{}'",
                                navn, type_navn_fra_type(type_), v.type_navn()
                            )),
                        }
                        v
                    }
                    None => match type_ {
                        Type::Nummer    => Værdi::Nummer(0.0),
                        Type::Streng    => Værdi::Streng(String::new()),
                        Type::SandFalsk => Værdi::SandFalsk(false),
                    },
                };
                println!(
                    "[trace] Erklærer '{}' som {} = {}",
                    navn, type_navn_fra_type(type_), værdi.vis()
                );
                self.hukommelse.insert(navn.clone(), værdi);
                Ok(())
            }
            Saetning::Sæt { navn, udtryk } => {
                // Tjek at variablen eksisterer og bevar typen
                let eksisterende_type = self.hukommelse.get(navn).map(|v| v.type_navn()).ok_or_else(|| {
                    format!("Ukendt variabel '{}' — erklær den først med 'erklær'", navn)
                })?;
                let ny_værdi = self.evaluer_udtryk(udtryk)?;
                if ny_værdi.type_navn() != eksisterende_type {
                    return Err(format!(
                        "Typefejl: '{}' er af typen '{}', men du prøver at sætte en '{}'-værdi",
                        navn, eksisterende_type, ny_værdi.type_navn()
                    ));
                }
                println!("[trace] Sætter '{}' = {}", navn, ny_værdi.vis());
                self.hukommelse.insert(navn.clone(), ny_værdi);
                Ok(())
            }
            Saetning::VisHukommelse => {
                println!("[trace] Viser hukommelse");
                if self.hukommelse.is_empty() {
                    println!("(ingen variabler endnu)");
                } else {
                    let mut navne: Vec<&String> = self.hukommelse.keys().collect();
                    navne.sort();
                    println!("{:<20} {:<12} {:<18} {}", "Navn", "Type", "Adresse", "Værdi");
                    println!("{}", "-".repeat(66));
                    for navn in navne {
                        let v = &self.hukommelse[navn];
                        let adresse = v as *const Værdi as usize;
                        println!("{:<20} {:<12} {:#018x} {}", navn, v.type_navn(), adresse, v.vis());
                    }
                }
                Ok(())
            }
            Saetning::Hvis { betingelse, så_gren, ellers_gren } => {
                let værdi = self.evaluer_udtryk(betingelse)?;
                match værdi {
                    Værdi::SandFalsk(sand) => {
                        println!("[trace] hvis-betingelse er {}", if sand { "sand" } else { "falsk" });
                        if sand {
                            self.kør(så_gren)?;
                        } else if let Some(ellers) = ellers_gren {
                            self.kør(ellers)?;
                        }
                        Ok(())
                    }
                    v => Err(format!(
                        "Typefejl: 'hvis' kræver en sand/falsk-betingelse, men fandt '{}'",
                        v.type_navn()
                    )),
                }
            }
            Saetning::Gentag { antal, krop } => {
                let v = self.evaluer_udtryk(antal)?;
                let n = match v {
                    Værdi::Nummer(n) => {
                        if n < 0.0 || n.fract() != 0.0 {
                            return Err(format!("'gentag' kræver et positivt heltal, men fandt {}", n));
                        }
                        n as u64
                    }
                    v => return Err(format!("'gentag' kræver et tal, men fandt '{}'", v.type_navn())),
                };
                println!("[trace] Gentager {} gange", n);
                for i in 1..=n {
                    println!("[trace] Gentagelse {}/{}", i, n);
                    self.kør(krop)?;
                    if self.return_signal.is_some() { break; }
                }
                Ok(())
            }
            Saetning::Mens { betingelse, krop } => {
                let mut iteration = 0u64;
                loop {
                    if self.return_signal.is_some() { break; }
                    let v = self.evaluer_udtryk(betingelse)?;
                    match v {
                        Værdi::SandFalsk(true) => {
                            iteration += 1;
                            println!("[trace] mens-betingelse er sand (iteration {})", iteration);
                            self.kør(krop)?;
                        }
                        Værdi::SandFalsk(false) => {
                            println!("[trace] mens-betingelse er falsk — stopper");
                            break;
                        }
                        v => return Err(format!(
                            "Typefejl: 'mens' kræver en sand/falsk-betingelse, men fandt '{}'",
                            v.type_navn()
                        )),
                    }
                }
                Ok(())
            }
            Saetning::FunktionDef { .. } => {
                // Allerede registreret i kør() — intet at gøre her
                Ok(())
            }
            Saetning::Returner(udtryk) => {
                let v = match udtryk {
                    Some(u) => self.evaluer_udtryk(u)?,
                    None    => Værdi::SandFalsk(false),
                };
                println!("[trace] Returnerer: {}", v.vis());
                self.return_signal = Some(v);
                Ok(())
            }
            Saetning::Udtryksaetning(udtryk) => {
                // Udfør udtryk (typisk et funktionskald) og kassér værdien
                self.evaluer_udtryk(udtryk)?;
                Ok(())
            }
        }
    }
}

fn type_navn_fra_type(t: &Type) -> &'static str {
    match t {
        Type::Nummer    => "nummer",
        Type::Streng    => "streng",
        Type::SandFalsk => "sandFalsk",
    }
}

