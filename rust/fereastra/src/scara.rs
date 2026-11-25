//! Scara unica a ecranului de conectare (etapa 17): ecranul e gandit la
//! 1440x900 si creste sau scade cu totul, text, campuri si spatii, dupa cat
//! din 1440x900 incape in fereastra.
//!
//! Folosita doar de capturi (`examples/captura_conectare.rs`): pe fereastra
//! nu se mai pune nicio scara peste cea a sistemului (2026-11-25, vezi
//! `leaga_scara`).
//!
//! **Se strica astfel:** ecranul pare ca nu incape pe un laptop si e pierdut
//! intr-un monitor mare, sau scara se bate cu marirea aleasa in Windows (se
//! micsoreaza peste 150%, ori sare la mutarea pe alt monitor).

/// Latimea si inaltimea pentru care e gandit ecranul, in pixeli logici.
pub const REFERINTA: (f32, f32) = (1440.0, 900.0);
/// Cat se poate micsora: sub asta textul de 13 px nu se mai citeste.
pub const MINIM: f32 = 0.85;
/// Cat se poate mari: peste asta ecranul pare facut pentru alt aparat.
pub const MAXIM: f32 = 1.3;
/// De la ce marire aleasa in sistem nu se mai micsoreaza (Mihai,
/// 2026-11-19): cine a ales 125% sau peste vrea textul mare.
pub const SISTEM_MARIT: f32 = 1.25;

/// Scara pentru o fereastra de `latime` x `inaltime` pixeli logici ai
/// sistemului, cand sistemul mareste cu `factor_sistem`.
pub fn scara(latime: f32, inaltime: f32, factor_sistem: f32) -> f32 {
    if !(latime > 0.0 && inaltime > 0.0) {
        return 1.0;
    }
    let minim = if factor_sistem >= SISTEM_MARIT {
        1.0
    } else {
        MINIM
    };
    (latime / REFERINTA.0)
        .min(inaltime / REFERINTA.1)
        .clamp(minim, MAXIM)
}

#[cfg(test)]
mod teste {
    use super::*;

    #[test]
    fn scara_e_cat_incape_din_1440x900_intre_limite() {
        assert_eq!(scara(1440.0, 900.0, 1.0), 1.0);
        assert!((scara(1920.0, 1080.0, 1.0) - 1.2).abs() < 1e-6);
        assert_eq!(scara(2560.0, 1440.0, 1.0), MAXIM);
        assert_eq!(scara(1280.0, 720.0, 1.0), MINIM);
        // ingusta si inalta: decide latimea
        assert!((scara(1296.0, 1200.0, 1.0) - 0.9).abs() < 1e-6);
        assert_eq!(scara(0.0, 900.0, 1.0), 1.0);
    }

    #[test]
    fn cu_sistemul_marit_nu_se_micsoreaza() {
        // laptop 1920x1080 la 150%: 1280x720 logici
        assert_eq!(scara(1280.0, 720.0, 1.5), 1.0);
        assert_eq!(scara(1280.0, 720.0, 1.25), 1.0);
        assert_eq!(scara(1280.0, 720.0, 1.2), MINIM);
        // marirea in sus ramane
        assert_eq!(scara(2560.0, 1440.0, 2.0), MAXIM);
    }
}
