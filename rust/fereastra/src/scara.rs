//! Scara unica a ecranului de conectare (etapa 17): ecranul e gandit la
//! 1440x900 si creste sau scade cu totul, text, campuri si spatii, dupa cat
//! din 1440x900 incape in fereastra.
//!
//! **Se strica astfel:** ecranul pare ca nu incape pe un laptop si e pierdut
//! intr-un monitor mare, sau scara se bate cu marirea aleasa in Windows (se
//! micsoreaza peste 150%, ori sare la mutarea pe alt monitor).

use std::cell::Cell;

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

/// Tine minte factorul sistemului si pe cel pus de noi, ca o schimbare venita
/// din sistem (alt monitor, alta marire) sa fie recunoscuta si nu luata drept
/// a noastra.
#[derive(Debug, Default)]
pub struct ScaraFereastra {
    sistem: Cell<f32>,
    pusa: Cell<f32>,
}

impl ScaraFereastra {
    pub fn noua() -> Self {
        Self::default()
    }

    /// Factorul total de pus pe fereastra (sistem x scara), sau `None` daca
    /// cel de acum e deja bun. `actual` e factorul ferestrei acum, `fizic`
    /// marimea ei in pixeli de ecran.
    pub fn calculeaza(&self, actual: f32, fizic: (u32, u32)) -> Option<f32> {
        if (actual - self.pusa.get()).abs() > 1e-4 {
            // nu e factorul pus de noi: l-a schimbat sistemul
            self.sistem.set(actual);
        }
        let sistem = self.sistem.get();
        if sistem <= 0.0 {
            return None;
        }
        let k = scara(fizic.0 as f32 / sistem, fizic.1 as f32 / sistem, sistem);
        let nou = sistem * k;
        if (nou - actual).abs() < 1e-3 {
            return None;
        }
        self.pusa.set(nou);
        Some(nou)
    }

    /// Pune scara pe fereastra, daca s-a schimbat ceva. Se cheama la fiecare
    /// redimensionare; marimea fizica nu se schimba, doar cea logica.
    pub fn aplica(&self, w: &slint::Window) {
        use slint::platform::WindowEvent;
        let fizic = w.size();
        let Some(nou) = self.calculeaza(w.scale_factor(), (fizic.width, fizic.height)) else {
            return;
        };
        w.dispatch_event(WindowEvent::ScaleFactorChanged { scale_factor: nou });
        w.dispatch_event(WindowEvent::Resized {
            size: fizic.to_logical(nou),
        });
    }
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

    #[test]
    fn deosebeste_factorul_sistemului_de_al_nostru() {
        let s = ScaraFereastra::noua();
        // 1920x1080 fizici, sistem 1.0: pune 1.2
        assert_eq!(s.calculeaza(1.0, (1920, 1080)), Some(1.2));
        // dupa ce l-am pus, acelasi factor nu se mai cere
        assert_eq!(s.calculeaza(1.2, (1920, 1080)), None);
        // fereastra micsorata: tot din sistemul 1.0 se calculeaza
        assert_eq!(s.calculeaza(1.2, (1440, 900)), Some(1.0));
        // sistemul trece pe 1.5 (alt monitor): 1920x1080 fizici = 1280x720
        // logici, deci scara 1, factor total 1.5, adica exact al sistemului
        assert_eq!(s.calculeaza(1.5, (1920, 1080)), None);
        assert_eq!(s.calculeaza(1.5, (3840, 2160)), Some(1.5 * MAXIM));
    }
}
