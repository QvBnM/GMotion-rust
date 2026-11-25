// Generatorul shell-ului: o singura fereastra (`Aplicatie`) care contine toate
// ecranele, si cate un tip Rust per ecran (`FereastraLista`, ...) cu aceleasi
// metode ca inainte, ca restul codului sa nu stie ca ecranele nu mai sunt
// ferestre separate.
//
// Se strica astfel: o proprietate noua pe radacina unui ecran nu ajunge in
// Rust (declaratie pe care parserul nu o recunoaste), sau un tip Slint nou nu
// are corespondent Rust aici (build-ul cade cu numele tipului).
//
// Fisierul e inclus si de `build.rs`, si de testele din `lib.rs`.

/// Un ecran: numele componentei Slint, prefixul din shell, numarul lui.
pub struct EcranShell {
    pub componenta: &'static str,
    pub prefix: &'static str,
    pub fisier: &'static str,
    pub numar: i32,
    /// are `forward-focus` (un `FocusScope` pentru taste)
    pub cu_taste: bool,
}

pub const ECRANE: [EcranShell; 4] = [
    EcranShell {
        componenta: "FereastraConectare",
        prefix: "conectare",
        fisier: "conectare.slint",
        numar: 0,
        cu_taste: true,
    },
    EcranShell {
        componenta: "FereastraLista",
        prefix: "lista",
        fisier: "lista.slint",
        numar: 1,
        cu_taste: true,
    },
    EcranShell {
        componenta: "EditorDiferente",
        prefix: "editor",
        fisier: "editor.slint",
        numar: 2,
        cu_taste: true,
    },
    EcranShell {
        componenta: "FereastraDeploy",
        prefix: "deploy",
        fisier: "deploy.slint",
        numar: 3,
        cu_taste: false,
    },
];

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Declaratie {
    /// `in`, `out` sau `in-out`, tipul Slint, numele
    Proprietate(String, String, String),
    /// numele, tipurile argumentelor, tipul intors (sau gol)
    Apel(String, Vec<String>, String),
    /// `public function`: numele, argumentele (nume, tip), tipul intors
    Functie(String, Vec<(String, String)>, String),
}

/// Declaratiile publice de pe radacina componentei `componenta` din `sursa`:
/// liniile cu exact 4 spatii intre `export component X inherits ... {` si
/// acolada care o inchide (coloana 0).
pub fn declaratii(sursa: &str, componenta: &str) -> Vec<Declaratie> {
    let antet = format!("export component {componenta} inherits ");
    let mut in_radacina = false;
    let mut v = Vec::new();
    for linie in sursa.lines() {
        if !in_radacina {
            in_radacina = linie.starts_with(&antet);
            continue;
        }
        if linie.starts_with('}') {
            break;
        }
        let Some(rest) = linie.strip_prefix("    ") else {
            continue;
        };
        if rest.starts_with(' ') {
            continue;
        }
        if let Some(d) = proprietate(rest) {
            v.push(d);
        } else if let Some(d) = functie(rest) {
            v.push(d);
        } else if let Some(d) = apel(rest) {
            v.push(d);
        }
    }
    v
}

fn proprietate(rest: &str) -> Option<Declaratie> {
    let (acces, dupa) = ["in-out", "in", "out"].iter().find_map(|a| {
        rest.strip_prefix(&format!("{a} property <"))
            .map(|d| (*a, d))
    })?;
    let (tip, dupa) = dupa.split_once("> ")?;
    let nume: String = dupa
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
        .collect();
    (!nume.is_empty()).then(|| Declaratie::Proprietate(acces.into(), tip.trim().into(), nume))
}

fn functie(rest: &str) -> Option<Declaratie> {
    let rest = fara_comentarii(rest);
    let dupa = rest
        .strip_prefix("public function ")
        .or_else(|| rest.strip_prefix("public pure function "))
        .or_else(|| rest.strip_prefix("pure public function "))?;
    let (nume, dupa) = dupa.split_once('(')?;
    let (argumente, dupa) = dupa.split_once(')')?;
    let argumente = argumente
        .split(',')
        .filter(|a| !a.trim().is_empty())
        .filter_map(|a| {
            let (n, t) = a.split_once(':')?;
            Some((n.trim().to_string(), t.trim().to_string()))
        })
        .collect();
    let intors = dupa
        .trim()
        .strip_prefix("->")
        .map(|r| r.trim().trim_end_matches('{').trim().to_string())
        .unwrap_or_default();
    Some(Declaratie::Functie(
        nume.trim().to_string(),
        argumente,
        intors,
    ))
}

/// Linia fara comentarii (`/* ... */` si `// ...`).
fn fara_comentarii(l: &str) -> String {
    let mut s = String::new();
    let mut rest = l;
    while let Some(i) = rest.find("/*") {
        s.push_str(&rest[..i]);
        rest = rest[i..].split_once("*/").map_or("", |(_, d)| d);
    }
    s.push_str(rest);
    match s.find("//") {
        Some(i) => s[..i].to_string(),
        None => s,
    }
}

fn apel(rest: &str) -> Option<Declaratie> {
    let rest = fara_comentarii(rest);
    let rest = rest.strip_prefix("pure ").unwrap_or(&rest);
    let dupa = rest.strip_prefix("callback ")?;
    let nume: String = dupa
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
        .collect();
    if nume.is_empty() {
        return None;
    }
    let dupa = &dupa[nume.len()..];
    let (argumente, intors) = match dupa.strip_prefix('(') {
        Some(d) => {
            let (a, r) = d.split_once(')')?;
            (a.to_string(), r.to_string())
        }
        None => (String::new(), dupa.to_string()),
    };
    let argumente: Vec<String> = argumente
        .split(',')
        .map(|a| a.trim())
        .filter(|a| !a.is_empty())
        // `nume: tip` sau doar `tip`
        .map(|a| a.rsplit(':').next().unwrap_or(a).trim().to_string())
        .collect();
    let intors = intors
        .trim()
        .strip_prefix("->")
        .map(|r| r.trim().trim_end_matches(';').trim().to_string())
        .unwrap_or_default();
    Some(Declaratie::Apel(nume, argumente, intors))
}

/// Tipul Rust pentru un tip Slint de pe radacina unui ecran.
pub fn tip_rust(tip: &str) -> String {
    let tip = tip.trim();
    if let Some(interior) = tip.strip_prefix('[').and_then(|t| t.strip_suffix(']')) {
        return format!("slint::ModelRc<{}>", tip_rust(interior));
    }
    match tip {
        "string" => "slint::SharedString".into(),
        "bool" => "bool".into(),
        "int" => "i32".into(),
        "float" | "length" | "percent" | "physical-length" | "angle" | "relative-font-size" => {
            "f32".into()
        }
        "duration" => "i64".into(),
        "color" => "slint::Color".into(),
        "brush" => "slint::Brush".into(),
        "image" => "slint::Image".into(),
        alt => alt.into(),
    }
}

pub fn sarpe(n: &str) -> String {
    n.replace('-', "_")
}

/// Sursa `.slint` a shell-ului. `ui` e calea absoluta a directorului `ui/`.
/// `structuri`: (numele tipului, fisierul `.slint` care il exporta).
pub fn shell_slint(
    ui: &str,
    exporturi: &[String],
    structuri: &[(String, String)],
    decl: &[Vec<Declaratie>],
) -> String {
    let mut s = String::new();
    s.push_str("// GENERAT de build.rs (build_shell.rs). Nu se editeaza de mana.\n");
    // tipurile struct folosite pe radacina ecranelor, din fisierul lor
    let mut folosite: Vec<&(String, String)> = structuri
        .iter()
        .filter(|(n, _)| {
            !exporturi.contains(n)
                && decl.iter().flatten().any(|d| match d {
                    Declaratie::Proprietate(_, t, _) => {
                        t.trim_matches(|c| c == '[' || c == ']') == n
                    }
                    Declaratie::Apel(_, a, r) => a.iter().any(|t| t == n) || r == n,
                    Declaratie::Functie(_, a, r) => a.iter().any(|(_, t)| t == n) || r == n,
                })
        })
        .collect();
    folosite.sort();
    for (n, f) in folosite {
        s.push_str(&format!("import {{ {n} }} from \"{ui}/{f}\";\n"));
        s.push_str(&format!("export {{ {n} }}\n"));
    }
    let ecrane: Vec<&str> = ECRANE.iter().map(|e| e.componenta).collect();
    let mut importate: Vec<&str> = ecrane.clone();
    importate.extend(exporturi.iter().map(|e| e.as_str()));
    if !importate.contains(&"Tema") {
        importate.push("Tema");
    }
    s.push_str(&format!(
        "import {{ Sfat, SfatAfisat }} from \"{ui}/comune.slint\";\n"
    ));
    s.push_str(&format!(
        "import {{ {} }} from \"{ui}/fereastra.slint\";\n",
        importate.join(", ")
    ));
    if !exporturi.is_empty() {
        s.push_str(&format!("export {{ {} }}\n", exporturi.join(", ")));
    }
    s.push_str("export component Aplicatie inherits Window {\n");
    s.push_str("    title: root.ecran == 1 ? lista.titlu : root.ecran == 2 ? editor.titlu : root.ecran == 3 ? deploy.titlu : conectare.titlu;\n");
    s.push_str("    preferred-width: 1440px;\n    preferred-height: 900px;\n");
    s.push_str("    min-width: 900px;\n    min-height: 560px;\n");
    s.push_str("    background: Tema.fundal;\n");
    s.push_str("    default-font-family: Tema.font-ui;\n    default-font-size: 13px;\n");
    s.push_str("    // ecranul vizibil: 0 conectare, 1 lista, 2 editor, 3 deploy\n");
    s.push_str("    in-out property <int> ecran;\n");
    s.push_str("    // cat dureaza trecerea dintre ecrane; 0 la pornire si in capturi\n");
    s.push_str("    in property <duration> durata-tranzitie: 0ms;\n");
    for e in ECRANE.iter() {
        s.push_str(&format!(
            "    {p} := {c} {{\n        x: 0; y: 0; width: root.width; height: root.height;\n        opacity: root.ecran == {n} ? 1 : 0;\n        visible: self.opacity > 0;\n        animate opacity {{ duration: root.durata-tranzitie; easing: ease-out; }}\n    }}\n",
            p = e.prefix,
            c = e.componenta,
            n = e.numar
        ));
    }
    // sfatul la hover, unul pentru toata fereastra, deasupra ecranelor
    s.push_str("    SfatAfisat { x: 0; y: 0; width: root.width; height: root.height; }\n");
    // tastele merg pe ecranul vizibil; un sfat ramas de pe ecranul vechi pleaca
    s.push_str("    changed ecran => {\n        Sfat.goleste();\n");
    // un ecran care are `public function intra()` decide singur unde merg
    // tastele (editorul nu fura focusul campului de editare)
    for (e, d) in ECRANE.iter().zip(decl) {
        let are_intra = d
            .iter()
            .any(|x| matches!(x, Declaratie::Functie(n, a, _) if n == "intra" && a.is_empty()));
        if are_intra {
            s.push_str(&format!(
                "        if (root.ecran == {}) {{ {}.intra(); }}\n",
                e.numar, e.prefix
            ));
        } else if e.cu_taste {
            s.push_str(&format!(
                "        if (root.ecran == {}) {{ {}.focus(); }}\n",
                e.numar, e.prefix
            ));
        }
    }
    s.push_str("    }\n");
    for (e, d) in ECRANE.iter().zip(decl) {
        for x in d {
            match x {
                Declaratie::Proprietate(acces, tip, nume) => s.push_str(&format!(
                    "    {acces} property <{tip}> {p}-{nume} <=> {p}.{nume};\n",
                    p = e.prefix
                )),
                Declaratie::Apel(nume, _, _) => s.push_str(&format!(
                    "    callback {p}-{nume} <=> {p}.{nume};\n",
                    p = e.prefix
                )),
                Declaratie::Functie(nume, argumente, intors) => {
                    let semn: Vec<String> =
                        argumente.iter().map(|(n, t)| format!("{n}: {t}")).collect();
                    let nume_arg: Vec<&str> = argumente.iter().map(|(n, _)| n.as_str()).collect();
                    let (r, ret) = if intors.is_empty() {
                        (String::new(), "")
                    } else {
                        (format!(" -> {intors}"), "return ")
                    };
                    s.push_str(&format!(
                        "    public function {p}-{nume}({}){r} {{ {ret}{p}.{nume}({}); }}\n",
                        semn.join(", "),
                        nume_arg.join(", "),
                        p = e.prefix
                    ));
                }
            }
        }
    }
    s.push_str("}\n");
    s
}

/// Tipurile Rust, cate unul per ecran, peste `Aplicatie`.
pub fn shell_rust(decl: &[Vec<Declaratie>]) -> String {
    let mut s = String::new();
    s.push_str("// GENERAT de build.rs (build_shell.rs). Nu se editeaza de mana.\n");
    for (e, d) in ECRANE.iter().zip(decl) {
        let t = e.componenta;
        let p = sarpe(e.prefix);
        s.push_str(&format!(
            r#"
/// Ecranul `{t}` din fereastra unica. Se comporta ca o fereastra: `show`
/// il face ecranul vizibil, `window` e fereastra comuna.
pub struct {t}(Aplicatie);
impl Clone for {t} {{
    fn clone(&self) -> Self {{
        Self(slint::ComponentHandle::clone_strong(&self.0))
    }}
}}
/// Referinta slaba la [`{t}`].
#[derive(Clone)]
pub struct {t}Slaba(slint::Weak<Aplicatie>);
impl {t}Slaba {{
    pub fn upgrade(&self) -> Option<{t}> {{
        self.0.upgrade().map({t})
    }}
}}
impl AreAplicatie for {t} {{
    fn aplicatie(&self) -> &Aplicatie {{
        &self.0
    }}
}}
impl {t} {{
    /// Numarul ecranului in `Aplicatie::ecran`.
    pub const ECRAN: i32 = {n};
    /// O fereastra noua cu doar acest ecran vizibil (teste, capturi).
    pub fn new() -> Result<Self, slint::PlatformError> {{
        let a = Aplicatie::new()?;
        a.set_ecran({n});
        Ok(Self(a))
    }}
    /// Ecranul acesta din fereastra `a`.
    pub fn din(a: &Aplicatie) -> Self {{
        Self(slint::ComponentHandle::clone_strong(a))
    }}
    pub fn as_weak(&self) -> {t}Slaba {{
        {t}Slaba(slint::ComponentHandle::as_weak(&self.0))
    }}
    pub fn clone_strong(&self) -> Self {{
        self.clone()
    }}
    pub fn window(&self) -> &slint::Window {{
        slint::ComponentHandle::window(&self.0)
    }}
    /// Face ecranul vizibil si arata fereastra, daca nu era.
    pub fn show(&self) -> Result<(), slint::PlatformError> {{
        self.0.set_ecran({n});
        slint::ComponentHandle::show(&self.0)
    }}
    /// Fereastra e comuna: un ecran nu o poate ascunde singur.
    pub fn hide(&self) -> Result<(), slint::PlatformError> {{
        if self.0.get_ecran() == {n} {{
            slint::ComponentHandle::hide(&self.0)
        }} else {{
            Ok(())
        }}
    }}
    pub fn global<'a, T: slint::Global<'a, Aplicatie>>(&'a self) -> T {{
        slint::ComponentHandle::global::<T>(&self.0)
    }}
"#,
            n = e.numar
        ));
        for x in d {
            match x {
                Declaratie::Proprietate(acces, tip, nume) => {
                    let n = sarpe(nume);
                    let r = tip_rust(tip);
                    s.push_str(&format!(
                        "    pub fn get_{n}(&self) -> {r} {{ self.0.get_{p}_{n}() }}\n"
                    ));
                    if acces != "out" {
                        s.push_str(&format!(
                            "    pub fn set_{n}(&self, v: {r}) {{ self.0.set_{p}_{n}(v) }}\n"
                        ));
                    }
                }
                Declaratie::Functie(nume, argumente, intors) => {
                    let n = sarpe(nume);
                    let r = if intors.is_empty() {
                        String::new()
                    } else {
                        format!(" -> {}", tip_rust(intors))
                    };
                    let parametri: Vec<String> = argumente
                        .iter()
                        .enumerate()
                        .map(|(i, (_, t))| format!("a{i}: {}", tip_rust(t)))
                        .collect();
                    let nume_arg: Vec<String> =
                        (0..argumente.len()).map(|i| format!("a{i}")).collect();
                    s.push_str(&format!(
                        "    pub fn invoke_{n}(&self, {}){r} {{ self.0.invoke_{p}_{n}({}) }}\n",
                        parametri.join(", "),
                        nume_arg.join(", ")
                    ));
                }
                Declaratie::Apel(nume, argumente, intors) => {
                    let n = sarpe(nume);
                    let tipuri: Vec<String> = argumente.iter().map(|a| tip_rust(a)).collect();
                    let r = if intors.is_empty() {
                        String::new()
                    } else {
                        format!(" -> {}", tip_rust(intors))
                    };
                    let parametri: Vec<String> = tipuri
                        .iter()
                        .enumerate()
                        .map(|(i, t)| format!("a{i}: {t}"))
                        .collect();
                    let nume_arg: Vec<String> =
                        (0..tipuri.len()).map(|i| format!("a{i}")).collect();
                    s.push_str(&format!(
                        "    pub fn on_{n}(&self, f: impl FnMut({}){r} + 'static) {{ self.0.on_{p}_{n}(f) }}\n",
                        tipuri.join(", ")
                    ));
                    s.push_str(&format!(
                        "    pub fn invoke_{n}(&self, {}){r} {{ self.0.invoke_{p}_{n}({}) }}\n",
                        parametri.join(", "),
                        nume_arg.join(", ")
                    ));
                }
            }
        }
        s.push_str("}\n");
    }
    s
}

/// Ce exporta `fereastra.slint`, fara ecrane (shell-ul le exporta ca tipuri
/// Rust proprii).
pub fn exporturi(fereastra: &str) -> Vec<String> {
    let mut v = Vec::new();
    for linie in fereastra.lines() {
        let Some(rest) = linie.trim().strip_prefix("export {") else {
            continue;
        };
        let Some((lista, _)) = rest.split_once('}') else {
            continue;
        };
        for n in lista.split(',') {
            let n = n.trim();
            if !n.is_empty() && !ECRANE.iter().any(|e| e.componenta == n) {
                v.push(n.to_string());
            }
        }
    }
    v
}

/// `export struct Nume` din sursa unui fisier `.slint`.
pub fn structuri_exportate(sursa: &str) -> Vec<String> {
    sursa
        .lines()
        .filter_map(|l| l.strip_prefix("export struct "))
        .map(|r| {
            r.chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                .collect()
        })
        .collect()
}
