//! Contractul prin care restul aplicatiei ia obiecte: `SursaObiecte`.
//!
//! **Se strica astfel:** nimic singur. Dar daca granita e trasa greșit, tot ce
//! e deasupra devine imposibil de testat fara SQL Server — si asta e cea mai
//! scumpa greseala pe care o putem face in proiectul asta.
//!
//! Doua lucruri deliberate:
//!
//! **Sincron.** `tiberius` e asincron, dar asincronia se opreste in
//! `sqlserver.rs`, care tine un runtime propriu. Altfel `async` ar urca prin
//! platforma si module pana in interfata, pentru o aplicatie care face citiri
//! in lot, nu mii de conexiuni concurente. Costul nu merita.
//!
//! **Fara normalizare.** Sursa intoarce text brut. Normalizarea si hash-ul sunt
//! `gmotion-core`, si le calculeaza cine are nevoie de ele. Versiunea Python
//! facea totul la citire si tinea token-ii in memorie pentru fiecare obiect din
//! fiecare baza.

use crate::eroare::Rezultat;
use crate::model::Obiect;

/// De unde vin obiectele SQL. Implementat de `SursaSqlServer` (baza reala) si
/// de `SursaFixture` (instantaneu dintr-un fisier).
///
/// `&mut self` fiindca o implementare pe retea isi tine conexiunea si o
/// refoloseste intre apeluri.
pub trait SursaObiecte {
    /// Pentru jurnal, diagnostic si rapoarte. Niciodata parole.
    fn descriere(&self) -> String;

    /// Bazele disponibile, sortate. Exclude bazele de sistem.
    fn baze(&mut self) -> Rezultat<Vec<String>>;

    /// Obiectele unei baze: proceduri, view-uri, functii, triggere.
    fn obiecte(&mut self, baza: &str) -> Rezultat<Vec<Obiect>>;

    /// Citeste mai multe baze, raportand progresul. Implementarea implicita
    /// ajunge pentru ambele surse; exista ca sa nu repete fiecare apelant bucla
    /// si raportarea.
    ///
    /// Prima eroare oprește citirea: o comparare pe baze incomplete ar da
    /// rezultate false, ceea ce e mai rau decat sa nu dea nimic.
    ///
    /// `progres` e `&mut dyn FnMut`, nu `impl FnMut`: un parametru generic ar
    /// face traitul incompatibil cu `dyn`, iar `Box<dyn SursaObiecte>` e exact
    /// cum alege linia de comanda intre server si instantaneu la rulare.
    fn obiecte_multe(
        &mut self,
        baze: &[String],
        progres: &mut dyn FnMut(usize, usize, &str),
    ) -> Rezultat<Vec<(String, Vec<Obiect>)>> {
        let mut out = Vec::with_capacity(baze.len());
        for (i, baza) in baze.iter().enumerate() {
            progres(i, baze.len(), baza);
            let ob = self.obiecte(baza)?;
            tracing::debug!(target: "gmotion::db", %baza, nr = ob.len(), "obiecte citite");
            out.push((baza.clone(), ob));
        }
        progres(baze.len(), baze.len(), "");
        Ok(out)
    }
}

#[cfg(test)]
mod teste {
    use super::*;
    use crate::eroare::Eroare;

    struct SursaTest {
        esueaza_la: Option<String>,
    }

    impl SursaObiecte for SursaTest {
        fn descriere(&self) -> String {
            "test".into()
        }
        fn baze(&mut self) -> Rezultat<Vec<String>> {
            Ok(vec!["A".into(), "B".into(), "C".into()])
        }
        fn obiecte(&mut self, baza: &str) -> Rezultat<Vec<Obiect>> {
            if self.esueaza_la.as_deref() == Some(baza) {
                return Err(Eroare::Interogare {
                    baza: baza.into(),
                    detaliu: "test".into(),
                });
            }
            Ok(vec![Obiect {
                schema: "dbo".into(),
                nume: format!("p{baza}"),
                tip: "P".into(),
                definitie: "SELECT 1".into(),
                setari: None,
            }])
        }
    }

    #[test]
    fn obiecte_multe_raporteaza_progresul_si_pastreaza_ordinea() {
        let mut s = SursaTest { esueaza_la: None };
        let baze = vec!["A".to_string(), "B".to_string()];
        let mut pasi = Vec::new();
        let rez = s
            .obiecte_multe(&baze, &mut |i, n, b| pasi.push((i, n, b.to_string())))
            .unwrap();
        assert_eq!(rez.len(), 2);
        assert_eq!(rez[0].0, "A");
        assert_eq!(rez[1].0, "B");
        // un pas per baza, plus unul final de incheiere
        assert_eq!(pasi.len(), 3);
        assert_eq!(pasi[2], (2, 2, String::new()));
    }

    /// Traitul trebuie sa rămână folosibil ca `Box<dyn SursaObiecte>`: asa
    /// alege linia de comanda intre server si instantaneu la rulare. Un
    /// parametru generic intr-o metoda ar rupe asta, si s-a intamplat o data.
    #[test]
    fn traitul_rămâne_compatibil_cu_dyn() {
        let mut cutie: Box<dyn SursaObiecte> = Box::new(SursaTest { esueaza_la: None });
        assert_eq!(cutie.baze().unwrap().len(), 3);
        let baze = vec!["A".to_string()];
        assert_eq!(
            cutie.obiecte_multe(&baze, &mut |_, _, _| {}).unwrap().len(),
            1
        );
    }

    #[test]
    fn prima_eroare_oprește_citirea() {
        let mut s = SursaTest {
            esueaza_la: Some("B".into()),
        };
        let baze = vec!["A".to_string(), "B".to_string(), "C".to_string()];
        let mut vazute = Vec::new();
        let e = s
            .obiecte_multe(&baze, &mut |_, _, b| vazute.push(b.to_string()))
            .unwrap_err();
        assert_eq!(e.cod(), "GM-DB-004");
        // C nu a mai fost cerut
        assert_eq!(vazute, vec!["A", "B"]);
    }
}
