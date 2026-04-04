/// De tre typer i sproget
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Nummer,
    Streng,
    SandFalsk,
}

/// Operatorer
#[derive(Debug, Clone, PartialEq)]
pub enum Operator {
    Plus,
    Minus,
    ErLig,          // ==
    ErIkke,         // <>
    Mindre,         // <
    Større,         // >
    MindreEllerLig, // <=
    StørreEllerLig, // >=
}

/// Udtryk (expressions) — hvad der kan beregnes til en værdi
#[derive(Debug, Clone)]
pub enum Udtryk {
    /// En tekstliteral: "hej"
    Bogstav(String),
    /// Et talliteral: 42 eller 3.14
    Tal(f64),
    /// En boolsk literal: sand eller falsk
    BoolLiteral(bool),
    /// En variabelreference: alder
    Variable(String),
    /// En regneudtryk: a + b, a - b
    BinærOp {
        venstre: Box<Udtryk>,
        operator: Operator,
        højre: Box<Udtryk>,
    },
}

/// Sætninger (statements) — hvad der kan udføres
#[derive(Debug, Clone)]
pub enum Saetning {
    /// skriv <udtryk>
    Skriv(Udtryk),
    /// erklær <navn> som <type> [= <udtryk>]
    Erklær {
        navn: String,
        type_: Type,
        startværdi: Option<Udtryk>,
    },
    /// sæt <navn> = <udtryk>
    Sæt {
        navn: String,
        udtryk: Udtryk,
    },
    /// vis hukommelse
    VisHukommelse,
    /// hvis <betingelse> så <blok> [ellers <blok>] slut
    Hvis {
        betingelse: Udtryk,
        så_gren: Vec<Saetning>,
        ellers_gren: Option<Vec<Saetning>>,
    },
    /// gentag <antal> gange ... slut
    Gentag {
        antal: Udtryk,
        krop: Vec<Saetning>,
    },
    /// mens <betingelse> så ... slut
    Mens {
        betingelse: Udtryk,
        krop: Vec<Saetning>,
    },
}
