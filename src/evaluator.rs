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

/// Evaluator — udfører et program og forklarer hvert trin
pub struct Evaluator {
    hukommelse: HashMap<String, Værdi>,
}

impl Evaluator {
    pub fn ny() -> Self {
        Evaluator { hukommelse: HashMap::new() }
    }

    fn evaluer_udtryk(&self, udtryk: &Udtryk) -> Result<Værdi, String> {
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
        }
    }

    pub fn kør(&mut self, saetninger: &[Saetning]) -> Result<(), String> {
        for saetning in saetninger {
            self.udfør(saetning)?;
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
                }
                Ok(())
            }
            Saetning::Mens { betingelse, krop } => {
                let mut iteration = 0u64;
                loop {
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
