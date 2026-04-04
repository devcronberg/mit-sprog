/// Tokens — de mindste meningsfulde enheder i sproget
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Nøgleord
    Skriv,
    Erklær,
    Som,
    Sæt,
    Vis,
    Hukommelse,
    Hvis,
    Så,
    Ellers,
    Slut,
    Gentag,
    Gange,
    Mens,
    // Typenavn-tokens
    Nummer,
    Streng,
    SandFalsk,
    // Literal-tokens
    TalLiteral(f64),
    BogstavLiteral(String),
    Sand,
    Falsk,
    // Identifikator (variabelnavn)
    Ident(String),
    // Tegn
    Ligmed,         // =
    Plus,           // +
    Minus,          // -
    ErLig,          // ==
    ErIkke,         // <>
    Mindre,         // <
    Større,         // >
    MindreEllerLig, // <=
    StørreEllerLig, // >=
    // Linjeskift — afslutter en instruktion
    Newline,
    /// Slutning af fil
    FilSlut,
}

/// Lexer — omdanner kildekode til en liste af tokens
pub struct Lexer {
    kilde: Vec<char>,
    position: usize,
    pub linje: usize,
}

impl Lexer {
    pub fn ny(kilde: &str) -> Self {
        Lexer {
            kilde: kilde.chars().collect(),
            position: 0,
            linje: 1,
        }
    }

    fn kig(&self) -> Option<char> {
        self.kilde.get(self.position).copied()
    }

    fn spis(&mut self) -> Option<char> {
        let c = self.kilde.get(self.position).copied();
        if let Some(ch) = c {
            self.position += 1;
            if ch == '\n' {
                self.linje += 1;
            }
        }
        c
    }

    /// Springer kun mellemrum og tabulatorer over — IKKE newlines
    fn spring_over_vandret_mellemrum(&mut self) {
        while let Some(c) = self.kig() {
            if c == ' ' || c == '\t' || c == '\r' {
                self.spis();
            } else {
                break;
            }
        }
    }

    /// Springer tomme linjer og kommentarlinjer over mellem instruktioner
    fn spring_over_tomme_linjer(&mut self) {
        loop {
            self.spring_over_vandret_mellemrum();
            match self.kig() {
                // Kommentar: spring resten af linjen over
                Some('#') => {
                    while let Some(c) = self.spis() {
                        if c == '\n' { break; }
                    }
                }
                // Tom linje: spring newline over og fortsæt
                Some('\n') => { self.spis(); }
                _ => break,
            }
        }
    }

    fn læs_tekst(&mut self) -> Result<String, String> {
        // Spiser det åbne anførelsestegn
        self.spis();
        let mut tekst = String::new();
        loop {
            match self.spis() {
                Some('"') => return Ok(tekst),
                Some(c) => tekst.push(c),
                None => {
                    return Err(format!(
                        "Linje {}: Tekst mangler afsluttende \"-tegn",
                        self.linje
                    ))
                }
            }
        }
    }

    fn læs_ord(&mut self) -> String {
        let mut ord = String::new();
        while let Some(c) = self.kig() {
            // Tillad bogstaver (inkl. æ/ø/å), tal og underscore
            if c.is_alphanumeric() || c == '_' {
                ord.push(c);
                self.spis();
            } else {
                break;
            }
        }
        ord
    }

    fn læs_tal(&mut self) -> f64 {
        let mut s = String::new();
        while let Some(c) = self.kig() {
            if c.is_ascii_digit() || c == '.' {
                s.push(c);
                self.spis();
            } else {
                break;
            }
        }
        s.parse().unwrap_or(0.0)
    }

    /// Returnerer alle tokens fra kildekoden
    pub fn tokenisér(&mut self) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::new();

        self.spring_over_tomme_linjer();

        loop {
            self.spring_over_vandret_mellemrum();
            match self.kig() {
                None => {
                    tokens.push(Token::FilSlut);
                    break;
                }
                Some('#') => {
                    while let Some(c) = self.spis() {
                        if c == '\n' { break; }
                    }
                    tokens.push(Token::Newline);
                    self.spring_over_tomme_linjer();
                }
                Some('\n') => {
                    self.spis();
                    tokens.push(Token::Newline);
                    self.spring_over_tomme_linjer();
                }
                Some('=') => {
                    self.spis();
                    if self.kig() == Some('=') {
                        self.spis();
                        tokens.push(Token::ErLig);
                    } else {
                        tokens.push(Token::Ligmed);
                    }
                }
                Some('<') => {
                    self.spis();
                    match self.kig() {
                        Some('=') => { self.spis(); tokens.push(Token::MindreEllerLig); }
                        Some('>') => { self.spis(); tokens.push(Token::ErIkke); }
                        _ => tokens.push(Token::Mindre),
                    }
                }
                Some('>') => {
                    self.spis();
                    if self.kig() == Some('=') {
                        self.spis();
                        tokens.push(Token::StørreEllerLig);
                    } else {
                        tokens.push(Token::Større);
                    }
                }
                Some('+') => {
                    self.spis();
                    tokens.push(Token::Plus);
                }
                Some('-') => {
                    self.spis();
                    tokens.push(Token::Minus);
                }
                Some('"') => {
                    let tekst = self.læs_tekst()?;
                    tokens.push(Token::BogstavLiteral(tekst));
                }
                Some(c) if c.is_ascii_digit() => {
                    let tal = self.læs_tal();
                    tokens.push(Token::TalLiteral(tal));
                }
                Some(c) if c.is_alphabetic() || c == '_' => {
                    let ord = self.læs_ord();
                    let token = match ord.as_str() {
                        "skriv"      => Token::Skriv,
                        "erkl\u{00e6}r"  => Token::Erklær,
                        "som"        => Token::Som,
                        "s\u{00e6}t"     => Token::Sæt,
                        "vis"        => Token::Vis,
                        "hukommelse" => Token::Hukommelse,
                        "hvis"       => Token::Hvis,
                        "s\u{00e5}"      => Token::Så,
                        "ellers"     => Token::Ellers,
                        "slut"       => Token::Slut,
                        "gentag"     => Token::Gentag,
                        "gange"      => Token::Gange,
                        "mens"       => Token::Mens,
                        "nummer"     => Token::Nummer,
                        "streng"     => Token::Streng,
                        "sandFalsk"  => Token::SandFalsk,
                        "sand"       => Token::Sand,
                        "falsk"      => Token::Falsk,
                        _            => Token::Ident(ord),
                    };
                    tokens.push(token);
                }
                Some(c) => {
                    return Err(format!(
                        "Linje {}: Uventet tegn '{}'",
                        self.linje, c
                    ))
                }
            }
        }
        Ok(tokens)
    }
}
