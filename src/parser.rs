use crate::ast::{Operator, Saetning, Type, Udtryk};
use crate::lexer::Token;

/// Parser — omdanner en liste af tokens til et AST (abstrakt syntakstræ)
pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

impl Parser {
    pub fn ny(tokens: Vec<Token>) -> Self {
        Parser { tokens, position: 0 }
    }

    fn kig(&self) -> &Token {
        self.tokens.get(self.position).unwrap_or(&Token::FilSlut)
    }

    fn spis(&mut self) -> Token {
        let t = self.tokens.get(self.position).cloned().unwrap_or(Token::FilSlut);
        self.position += 1;
        t
    }

    fn forvent(&mut self, forventet: &Token) -> Result<Token, String> {
        let t = self.spis();
        if &t == forventet {
            Ok(t)
        } else {
            Err(format!("Forventede {:?}, men fandt {:?}", forventet, t))
        }
    }

    fn forvent_ident(&mut self) -> Result<String, String> {
        match self.spis() {
            Token::Ident(navn) => Ok(navn),
            t => Err(format!("Forventede et variabelnavn, men fandt {:?}", t)),
        }
    }

    fn parse_type(&mut self) -> Result<Type, String> {
        match self.spis() {
            Token::Nummer   => Ok(Type::Nummer),
            Token::Streng   => Ok(Type::Streng),
            Token::SandFalsk => Ok(Type::SandFalsk),
            t => Err(format!("Forventede en type (nummer/streng/sandFalsk), men fandt {:?}", t)),
        }
    }

    /// Parser én sætning
    fn parse_saetning(&mut self) -> Result<Saetning, String> {
        match self.kig() {
            Token::Skriv => {
                self.spis();
                let udtryk = self.parse_udtryk()?;
                Ok(Saetning::Skriv(udtryk))
            }
            Token::Erklær => {
                self.spis(); // spis 'erklær'
                let navn = self.forvent_ident()?;
                self.forvent(&Token::Som)?;
                let type_ = self.parse_type()?;
                // Valgfri startværdi: = <udtryk>
                let startværdi = if self.kig() == &Token::Ligmed {
                    self.spis(); // spis '='
                    Some(self.parse_udtryk()?)
                } else {
                    None
                };
                Ok(Saetning::Erklær { navn, type_, startværdi })
            }
            Token::Sæt => {
                self.spis(); // spis 'sæt'
                let navn = self.forvent_ident()?;
                self.forvent(&Token::Ligmed)?;
                let udtryk = self.parse_udtryk()?;
                Ok(Saetning::Sæt { navn, udtryk })
            }
            Token::Vis => {
                self.spis(); // spis 'vis'
                self.forvent(&Token::Hukommelse)?;
                Ok(Saetning::VisHukommelse)
            }
            Token::Hvis => {
                self.spis(); // spis 'hvis'
                let betingelse = self.parse_udtryk()?;
                self.forvent(&Token::Så)?;
                if self.kig() == &Token::Newline { self.spis(); }
                let så_gren = self.parse_blok()?;
                let ellers_gren = if self.kig() == &Token::Ellers {
                    self.spis();
                    if self.kig() == &Token::Newline { self.spis(); }
                    Some(self.parse_blok()?)
                } else {
                    None
                };
                self.forvent(&Token::Slut)?;
                Ok(Saetning::Hvis { betingelse, så_gren, ellers_gren })
            }
            Token::Gentag => {
                self.spis(); // spis 'gentag'
                let antal = self.parse_udtryk()?;
                self.forvent(&Token::Gange)?;
                if self.kig() == &Token::Newline { self.spis(); }
                let krop = self.parse_blok()?;
                self.forvent(&Token::Slut)?;
                Ok(Saetning::Gentag { antal, krop })
            }
            Token::Mens => {
                self.spis(); // spis 'mens'
                let betingelse = self.parse_udtryk()?;
                self.forvent(&Token::Så)?;
                if self.kig() == &Token::Newline { self.spis(); }
                let krop = self.parse_blok()?;
                self.forvent(&Token::Slut)?;
                Ok(Saetning::Mens { betingelse, krop })
            }
            Token::FilSlut => Err("Uventet slutning af fil".to_string()),
            t => Err(format!("Uventet token: {:?}", t)),
        }
    }

    /// Parser en blok af sætninger indtil 'ellers' eller 'slut'
    fn parse_blok(&mut self) -> Result<Vec<Saetning>, String> {
        let mut saetninger = Vec::new();
        loop {
            match self.kig() {
                Token::Slut | Token::Ellers | Token::FilSlut => break,
                Token::Newline => { self.spis(); }
                _ => {
                    saetninger.push(self.parse_saetning()?);
                    match self.kig() {
                        Token::Newline => { self.spis(); }
                        Token::Slut | Token::Ellers | Token::FilSlut => {}
                        t => return Err(format!("Forventede linjeskift, men fandt {:?}", t)),
                    }
                }
            }
        }
        Ok(saetninger)
    }

    /// Parser ét udtryk — sammenligning har lavere prioritet end + og -
    fn parse_udtryk(&mut self) -> Result<Udtryk, String> {
        let venstre = self.parse_addition()?;
        let op = match self.kig() {
            Token::ErLig          => Some(Operator::ErLig),
            Token::ErIkke         => Some(Operator::ErIkke),
            Token::Mindre         => Some(Operator::Mindre),
            Token::Større         => Some(Operator::Større),
            Token::MindreEllerLig => Some(Operator::MindreEllerLig),
            Token::StørreEllerLig => Some(Operator::StørreEllerLig),
            _ => None,
        };
        if let Some(operator) = op {
            self.spis();
            let højre = self.parse_addition()?;
            Ok(Udtryk::BinærOp { venstre: Box::new(venstre), operator, højre: Box::new(højre) })
        } else {
            Ok(venstre)
        }
    }

    /// Parser addition og subtraktion (venstre-associativ)
    fn parse_addition(&mut self) -> Result<Udtryk, String> {
        let mut venstre = self.parse_primær()?;
        loop {
            match self.kig() {
                Token::Plus => {
                    self.spis();
                    let højre = self.parse_primær()?;
                    venstre = Udtryk::BinærOp {
                        venstre: Box::new(venstre),
                        operator: Operator::Plus,
                        højre: Box::new(højre),
                    };
                }
                Token::Minus => {
                    self.spis();
                    let højre = self.parse_primær()?;
                    venstre = Udtryk::BinærOp {
                        venstre: Box::new(venstre),
                        operator: Operator::Minus,
                        højre: Box::new(højre),
                    };
                }
                _ => break,
            }
        }
        Ok(venstre)
    }

    /// Parser ét primært udtryk (en enkelt værdi)
    fn parse_primær(&mut self) -> Result<Udtryk, String> {
        match self.spis() {
            Token::BogstavLiteral(s) => Ok(Udtryk::Bogstav(s)),
            Token::TalLiteral(n)     => Ok(Udtryk::Tal(n)),
            Token::Sand              => Ok(Udtryk::BoolLiteral(true)),
            Token::Falsk             => Ok(Udtryk::BoolLiteral(false)),
            Token::Ident(navn)       => Ok(Udtryk::Variable(navn)),
            t => Err(format!("Forventede en værdi, men fandt {:?}", t)),
        }
    }

    /// Parser hele programmet til en liste af sætninger
    pub fn parse(&mut self) -> Result<Vec<Saetning>, String> {
        let mut saetninger = Vec::new();
        while self.kig() != &Token::FilSlut {
            saetninger.push(self.parse_saetning()?);
            match self.kig() {
                Token::Newline => { self.spis(); }
                Token::FilSlut => {}
                t => {
                    return Err(format!(
                        "Forventede linjeskift efter instruktion, men fandt {:?}",
                        t
                    ))
                }
            }
        }
        Ok(saetninger)
    }
}
