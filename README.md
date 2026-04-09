# mit-sprog

Et dansk pædagogisk programmeringssprog — designet til at vise hvad programmering *er* for en størrelse.

Sproget er bevidst simpelt. Det handler ikke om at bygge apps. Det handler om at forstå hvad en computer egentlig gør, trin for trin, på dansk.

---

## Hent og kør

Download den seneste [`mit-sprog.exe`](https://github.com/devcronberg/mit-sprog/releases/latest/download/mit-sprog.exe) — eller gå til [Releases](https://github.com/devcronberg/mit-sprog/releases/latest).

**Ingen installation nødvendig.** Kopier `.exe`-filen et sted på din maskine og kør den direkte fra Windows Terminal:

### Fortolker-tilstand (standard)

Kører programmet direkte og viser hvad der sker trin for trin:

```
mit-sprog.exe mittprogram.ms
```

```
[trace] Skriver: "hej verden"
hej verden
```

### Transpiler-tilstand (`--kompiler`)

Oversætter `.ms`-filen til C og kalder `gcc` for at bygge en selvstændig `.exe`:

```
mit-sprog.exe mittprogram.ms --kompiler
```

```
[transpiler] Genererede 'mittprogram.c'
[kompiler] Færdig: 'mittprogram.exe'
```

Den færdige `mittprogram.exe` kører uden `mit-sprog`, uden MinGW og uden nogen installation — eleven kan køre den direkte.

**Krav til transpiler-tilstand:** `gcc` skal være installeret og tilgængeligt i PATH. Installér med [Scoop](https://scoop.sh):
```
scoop install mingw
```

Hvis du er admin kan Scoop installeres som

```
irm get.scoop.sh -outfile install.ps1
.\install.ps1 -RunAsAdmin
```

### Forskellen på de to tilstande

|                   | Fortolker                          | Transpiler                          |
| ----------------- | ---------------------------------- | ----------------------------------- |
| Hvad sker der?    | Læser og kører `.ms`-filen direkte | Oversætter til C → bygger `.exe`    |
| Output            | Tekst i terminalen                 | En ny `.exe`-fil                    |
| Kræver gcc?       | Nej                                | Ja                                  |
| Eleven ser trace? | Ja                                 | Nej — den færdige `.exe` kører bare |
| Brug til          | Undervisning, udforskning          | "Tag dit program med hjem"          |

---

## Sproget — sådan virker det

Programmer skrives i filer med `.ms` som endelse.

### skriv

Skriver en værdi til skærmen. Virker med tekst, tal, variabler og sand/falsk-udtryk.

```
skriv "hej verden"
skriv 42
skriv sand
skriv alder
```

### Kommentarer

Linjer der starter med `#` ignoreres helt af computeren. Brug dem til at forklare hvad koden gør.

```
# Dette er et kommentar
skriv "hej verden"   # kommentarer må også stå efter kode
```

### Variabler

En variabel er en navngivet kasse i computerens hukommelse. Du erklærer den med `erklær`, ændrer værdien med `sæt` og kan se alle variabler med `vis hukommelse`.

**Erklær en variabel:**
```
erklær alder som nummer = 16
erklær navn som streng = "Alice"
erklær erElev som sandFalsk = sand
```

Hvis du ikke angiver en startværdi, sættes den automatisk til `0`, `""` eller `falsk`.

**Opdatér en variabel:**
```
sæt alder = 17
sæt navn = "Bob"
```

**Se hvad computeren husker:**
```
vis hukommelse
```
```
Navn                 Type         Værdi
--------------------------------------------------
alder                nummer       16
erElev               sandFalsk    sand
navn                 streng       Alice
```

**De tre typer:**

| Type        | Hvad er det?          | Eksempel           |
| ----------- | --------------------- | ------------------ |
| `nummer`    | Et tal (heltal/komma) | `42`, `3.14`, `-7` |
| `streng`    | En tekst              | `"Alice"`, `"hej"` |
| `sandFalsk` | Sand eller falsk      | `sand`, `falsk`    |

### Hvad sker der når programmet kører?

Computeren forklarer hvad den gør, mens den gør det:

```
[trace] Erklærer 'alder' som nummer = 16
[trace] Skriver: Alice
Alice
[trace] Viser hukommelse
Navn                 Type         Værdi
--------------------------------------------------
alder                nummer       16
```

Linjer der starter med `[trace]` er computerens egne forklaringer — ikke en del af dit program.

---

## Kommende sprogfunktioner

Sproget er under aktiv udvikling. Planlagte funktioner i prioriteret rækkefølge:

| Funktion            | Eksempel                                      | Status          |
| ------------------- | --------------------------------------------- | --------------- |
| Variabler med typer | `erklær alder som nummer = 16`                | ✅ Implementeret |
| Tekstvariabler      | `erklær navn som streng = "Mia"`              | ✅ Implementeret |
| Boolske værdier     | `erklær voksen som sandFalsk = sand`          | ✅ Implementeret |
| Hukommelsesdump     | `vis hukommelse`                              | ✅ Implementeret |
| Betingelser         | `hvis alder >= 18 så ... ellers ... slut`     | ✅ Implementeret |
| Løkker              | `gentag 5 gange ... slut`                      | ✅ Implementeret |
| While-løkke         | `mens x < 10 så ... slut`                     | ✅ Implementeret |
| Regneoperationer    | `sæt sum = a + b`                             | ✅ Implementeret |
| Funktioner          | `funktion f(n som nummer) giver nummer ... slut`| ✅ Implementeret |
| Input fra brugeren  | `spørg "Hvad hedder du?"`                     | Planlagt        |

---

## Sådan virker mit-sprog — konceptuelt

`mit-sprog` kan arbejde på to måder: som **fortolker** (standard) eller som **transpiler** (`--kompiler`).

I **fortolker-tilstand** læser, forstår og udfører `mit-sprog` koden i ét hug — der produceres ingen ny fil, og du ser `[trace]`-output trin for trin.

I **transpiler-tilstand** oversætter `mit-sprog` `.ms`-koden til C-kode og kalder `gcc` for at bygge en selvstændig `.exe` — to separate skridt, men automatiseret. Den færdige `.exe` kører uden `mit-sprog` overhovedet.

Begge tilstande bruger den samme pipeline (lexer → parser → AST). I fortolker-tilstand evalueres AST direkte; i transpiler-tilstand genereres C-kode fra AST i stedet.

Når du kører `mit-sprog.exe hej.ms` (uden flag) sker alt på én gang: filen åbnes, læses og køres direkte. Rust er kun brugt til at bygge selve fortolkeren — din `.ms`-kode oversættes ikke til maskinkode.

Et program er bare tekst. For at `mit-sprog` kan *forstå* det, behandles teksten i tre trin:

```
Kildekode (.ms-fil)
        │
        ▼
  ┌───────────┐
  │   Lexer   │  — "Hvad er ordene?"
  └───────────┘
        │  liste af tokens: [Erklær, Ident("alder"), Som, Nummer, Ligmed, TalLiteral(16), Slut]
        ▼
  ┌───────────┐
  │   Parser  │  — "Hvad betyder sætningerne?"
  └───────────┘
        │  abstrakt syntakstræ: [Erklær { navn: "alder", type_: Nummer, startværdi: Some(Tal(16)) }]
        ▼
  ┌───────────────┐         ┌────────────────────┐
  │ Evaluator     │  — eller — │ Kodegenerator (C)  │
  │ (fortolker)   │         │ (transpiler)       │
  └───────────────┘         └────────────────────┘
        │                           │
        ▼                           ▼
  Output + [trace]           mit-sprog.exe (via gcc)
```

### Trin 1 — Lexer (tokenisering)

Lexeren læser kildekoden tegn for tegn og grupperer dem til meningsfulde enheder kaldet *tokens*. Den ved ikke hvad tokens *betyder* — den genkender bare mønstre.

```
skriv "hej verden"
  │         │
  ▼         ▼
Skriv    BogstavLiteral("hej verden")
```

### Trin 2 — Parser (syntaksanalyse)

Parseren tager listen af tokens og bygger et *abstrakt syntakstræ* (AST). Træet repræsenterer programmets *struktur* — ikke bare ord, men hvad der hører sammen.

En `skriv`-sætning forventer præcis ét udtryk efter sig. Hvis det mangler, stopper parseren med en fejl.

### Trin 3 — Evaluator eller Kodegenerator

I **fortolker-tilstand** gennemgår evaluatoren syntakstræet og *udfører* det. For hvert trin printer den hvad den er ved at gøre (`[trace]`), og derefter selve resultatet.

I **transpiler-tilstand** gennemgår kodegeneratoren det samme syntakstræ og *oversætter* det til C-kode, som `gcc` derefter kompilerer til en selvstændig `.exe`.

---

## Vil du selv bygge fortolkeren?

Du skal bruge:
- [Rust](https://rustup.rs) — installationsvejledning på rustup.rs
- MinGW (GNU-linker til Windows) — installér med [Scoop](https://scoop.sh):
  ```
  scoop install mingw
  ```

Klon projektet og byg:

```
git clone <repo-url>
cd mit-sprog

# Sæt GNU-linkeren i PATH (kræves én gang per terminal-session)
$env:PATH = "$env:USERPROFILE\scoop\apps\mingw\current\bin;$env:PATH"

# Byg
cargo build --target x86_64-pc-windows-gnu --release

# Kør
.\target\x86_64-pc-windows-gnu\release\mit-sprog.exe .\eksempler\hej.ms
```

Den færdige `.exe` ligger i `target\x86_64-pc-windows-gnu\release\mit-sprog.exe` og kan kopieres og køres uden installation.

### Projektstruktur

```
src/
├── main.rs            — CLI-indgang: læser .ms-filen, kører pipelinen
├── ast.rs             — datastrukturerne: Type, Udtryk og Saetning enums
├── lexer.rs           — tegn-for-tegn tokenisering af kildekoden
├── parser.rs          — tokens → abstrakt syntakstræ
├── evaluator.rs       — udfører AST, printer trace-output, håndterer hukommelse
└── kodegenerator.rs   — oversætter AST til C-kode, kalder gcc
eksempler/
├── hej.ms             — "hej verden"-program
└── variabler.ms       — demonstration af erklær/sæt/vis hukommelse
```

### Kildekoden er bevidst enkel

Ingen externe biblioteker. Ingen makroer. Rust-koden er skrevet til at være læsbar, ikke optimal — så man kan følge logikken fra `main.rs` til output uden at fare vild.

---

## Licens

MIT
