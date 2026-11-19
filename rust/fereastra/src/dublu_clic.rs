//! Dublu clicul, numarat in Rust din doua clicuri simple.
//! Categoria de bug: dublu clic care nu ajunge (pe Windows, `double-clicked`
//! din Slint n-a venit deloc: lista, coloana editorului si taburile primeau
//! doar clicuri simple, vezi jurnalul din 2026-11-19) sau care se dubleaza.
use std::time::{Duration, Instant};

/// Al doilea clic pe aceeasi tinta, la cel mult [`DubluClic::INTERVAL`] dupa
/// primul, e dublu clic. Dupa un dublu clic numaratoarea o ia de la capat:
/// trei clicuri repezi nu deschid de doua ori.
#[derive(Debug, Default)]
pub struct DubluClic {
    ultim: Option<(i32, Instant)>,
}

impl DubluClic {
    /// Intervalul implicit din Windows pentru dublu clic.
    pub const INTERVAL: Duration = Duration::from_millis(500);

    /// Un clic pe `tinta` (randul, tabul) la momentul `acum`; `true` daca
    /// impreuna cu cel dinainte face un dublu clic.
    pub fn apasat(&mut self, tinta: i32, acum: Instant) -> bool {
        let dublu = matches!(self.ultim, Some((t, la))
            if t == tinta && acum.saturating_duration_since(la) <= Self::INTERVAL);
        self.ultim = if dublu { None } else { Some((tinta, acum)) };
        dublu
    }
}

#[cfg(test)]
mod teste {
    use super::*;

    #[test]
    fn doua_clicuri_repezi_pe_aceeasi_tinta_fac_dublu_clic() {
        let t0 = Instant::now();
        let ms = |n| t0 + Duration::from_millis(n);
        let mut d = DubluClic::default();
        assert!(!d.apasat(5, t0));
        assert!(d.apasat(5, ms(300)));
        // al treilea clic incepe o numaratoare noua
        assert!(!d.apasat(5, ms(400)));
        // prea tarziu
        assert!(!d.apasat(5, ms(1000)));
        // alta tinta
        assert!(!d.apasat(6, ms(1100)));
        assert!(d.apasat(6, ms(1600)));
        // exact la interval inca se numara
        let mut d = DubluClic::default();
        d.apasat(1, t0);
        assert!(d.apasat(1, t0 + DubluClic::INTERVAL));
    }
}
