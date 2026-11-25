//! Unde si cat de mare porneste fereastra: pe monitorul pe care e cursorul,
//! centrata, sau unde a fost lasata data trecuta, daca monitorul acela mai
//! exista. Partea de Win32 (monitoare, DPI, cursor) e toata aici.
//!
//! **Se strica astfel:** fereastra porneste mai mare decat ecranul, taiata la
//! dreapta sau jos (laptop la 125-150%, Mihai 2026-11-25), pe alt monitor
//! decat cel pe care lucrezi, ori intr-un loc care nu mai exista (monitorul
//! de data trecuta a fost scos).

/// Un dreptunghi in pixeli fizici, in coordonatele desktopului virtual.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Dreptunghi {
    pub x: i32,
    pub y: i32,
    pub latime: u32,
    pub inaltime: u32,
}

impl Dreptunghi {
    fn contine(&self, (px, py): (i32, i32)) -> bool {
        px >= self.x
            && py >= self.y
            && (px as i64) < self.x as i64 + self.latime as i64
            && (py as i64) < self.y as i64 + self.inaltime as i64
    }

    fn centru(&self) -> (i32, i32) {
        (
            self.x + (self.latime / 2) as i32,
            self.y + (self.inaltime / 2) as i32,
        )
    }
}

/// Un monitor: zona de lucru (fara bara de activitati) si marirea lui.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Monitor {
    pub lucru: Dreptunghi,
    pub scara: f32,
}

/// Marimea pentru care e gandita interfata, in pixeli logici.
pub const MARIME_GANDITA: (f32, f32) = (1440.0, 900.0);
/// Cat din zona de lucru poate lua fereastra la prima pornire.
pub const CEL_MULT: f32 = 0.92;

/// Geometria de pornire. `salvata` e unde a fost lasata data trecuta;
/// `cursor` e pozitia cursorului acum. `None` daca nu se stie niciun monitor
/// (atunci decide sistemul).
pub fn alege_geometria(
    salvata: Option<Dreptunghi>,
    monitoare: &[Monitor],
    cursor: Option<(i32, i32)>,
) -> Option<Dreptunghi> {
    // data trecuta: doar daca mijlocul ei e inca pe un monitor
    if let Some(s) = salvata.filter(|s| s.latime > 0 && s.inaltime > 0) {
        if let Some(m) = monitoare.iter().find(|m| m.lucru.contine(s.centru())) {
            return Some(incadreaza(s, &m.lucru));
        }
    }
    let m = cursor
        .and_then(|c| monitoare.iter().find(|m| m.lucru.contine(c)))
        .or_else(|| monitoare.first())?;
    let l = m.lucru;
    let latime = ((MARIME_GANDITA.0 * m.scara) as u32).min((l.latime as f32 * CEL_MULT) as u32);
    let inaltime = ((MARIME_GANDITA.1 * m.scara) as u32).min((l.inaltime as f32 * CEL_MULT) as u32);
    Some(Dreptunghi {
        x: l.x + ((l.latime - latime) / 2) as i32,
        y: l.y + ((l.inaltime - inaltime) / 2) as i32,
        latime,
        inaltime,
    })
}

/// `d` mutat si, la nevoie, micsorat ca sa incapa in `zona`.
fn incadreaza(d: Dreptunghi, zona: &Dreptunghi) -> Dreptunghi {
    let latime = d.latime.min(zona.latime);
    let inaltime = d.inaltime.min(zona.inaltime);
    let max_x = zona.x + (zona.latime - latime) as i32;
    let max_y = zona.y + (zona.inaltime - inaltime) as i32;
    Dreptunghi {
        x: d.x.clamp(zona.x, max_x),
        y: d.y.clamp(zona.y, max_y),
        latime,
        inaltime,
    }
}

/// Monitoarele si cursorul, citite din sistem. Pe altceva decat Windows nu
/// se stie nimic: decide sistemul de ferestre.
pub fn citeste_sistemul() -> (Vec<Monitor>, Option<(i32, i32)>) {
    sistem::citeste()
}

#[cfg(not(windows))]
mod sistem {
    pub fn citeste() -> (Vec<super::Monitor>, Option<(i32, i32)>) {
        (Vec::new(), None)
    }
}

#[cfg(windows)]
mod sistem {
    use super::{Dreptunghi, Monitor};
    use std::ffi::c_void;

    #[repr(C)]
    #[derive(Default, Clone, Copy)]
    struct Rect {
        left: i32,
        top: i32,
        right: i32,
        bottom: i32,
    }
    #[repr(C)]
    #[derive(Default)]
    struct Punct {
        x: i32,
        y: i32,
    }
    #[repr(C)]
    struct InfoMonitor {
        cb_size: u32,
        rc_monitor: Rect,
        rc_work: Rect,
        dw_flags: u32,
    }

    type Hmonitor = *mut c_void;
    type Enumerare = unsafe extern "system" fn(Hmonitor, *mut c_void, *mut Rect, isize) -> i32;

    #[link(name = "user32")]
    extern "system" {
        fn GetCursorPos(p: *mut Punct) -> i32;
        fn EnumDisplayMonitors(
            hdc: *mut c_void,
            clip: *const Rect,
            f: Enumerare,
            date: isize,
        ) -> i32;
        fn GetMonitorInfoW(m: Hmonitor, info: *mut InfoMonitor) -> i32;
    }
    #[link(name = "shcore")]
    extern "system" {
        fn GetDpiForMonitor(m: Hmonitor, fel: i32, x: *mut u32, y: *mut u32) -> i32;
    }

    unsafe extern "system" fn unul(m: Hmonitor, _: *mut c_void, _: *mut Rect, date: isize) -> i32 {
        // SAFETY: `date` e adresa vectorului din `citeste`, viu pe toata
        // durata lui `EnumDisplayMonitors` (apel sincron).
        let v = unsafe { &mut *(date as *mut Vec<Monitor>) };
        let mut info = InfoMonitor {
            cb_size: std::mem::size_of::<InfoMonitor>() as u32,
            rc_monitor: Rect::default(),
            rc_work: Rect::default(),
            dw_flags: 0,
        };
        // SAFETY: `info` are `cb_size` pus, cum cere API-ul
        if unsafe { GetMonitorInfoW(m, &mut info) } == 0 {
            return 1;
        }
        let (mut dx, mut dy) = (96u32, 96u32);
        // 0 = MDT_EFFECTIVE_DPI; la esec ramane 96 (100%)
        // SAFETY: pointerii sunt la variabile locale
        let _ = unsafe { GetDpiForMonitor(m, 0, &mut dx, &mut dy) };
        let r = info.rc_work;
        if r.right > r.left && r.bottom > r.top {
            v.push(Monitor {
                lucru: Dreptunghi {
                    x: r.left,
                    y: r.top,
                    latime: (r.right - r.left) as u32,
                    inaltime: (r.bottom - r.top) as u32,
                },
                scara: dx.max(96) as f32 / 96.0,
            });
        }
        1
    }

    pub fn citeste() -> (Vec<Monitor>, Option<(i32, i32)>) {
        let mut v: Vec<Monitor> = Vec::new();
        // SAFETY: callback-ul primeste adresa lui `v` si o foloseste doar cat
        // tine apelul
        unsafe {
            EnumDisplayMonitors(
                std::ptr::null_mut(),
                std::ptr::null(),
                unul,
                &mut v as *mut Vec<Monitor> as isize,
            )
        };
        let mut p = Punct::default();
        // SAFETY: `p` e local
        let cursor = (unsafe { GetCursorPos(&mut p) } != 0).then_some((p.x, p.y));
        (v, cursor)
    }
}

#[cfg(test)]
mod teste {
    use super::*;

    fn monitor(x: i32, latime: u32, inaltime: u32, scara: f32) -> Monitor {
        Monitor {
            lucru: Dreptunghi {
                x,
                y: 0,
                latime,
                inaltime,
            },
            scara,
        }
    }

    #[test]
    fn porneste_centrata_pe_monitorul_cursorului() {
        let m = [monitor(0, 1920, 1040, 1.0), monitor(1920, 2560, 1400, 1.0)];
        let g = alege_geometria(None, &m, Some((2500, 300))).expect("geometrie");
        assert_eq!((g.latime, g.inaltime), (1440, 900));
        assert_eq!(g.x, 1920 + (2560 - 1440) / 2);
        assert_eq!(g.y, (1400 - 900) / 2);
    }

    #[test]
    fn pe_un_laptop_marit_nu_iese_din_ecran() {
        // 1920x1080 la 150%, bara jos: 1440x900 logici ar fi 2160x1350 fizici
        let m = [monitor(0, 1920, 1032, 1.5)];
        let g = alege_geometria(None, &m, Some((10, 10))).expect("geometrie");
        assert!(g.latime <= 1920 && g.inaltime <= 1032, "{g:?}");
        assert!(g.x >= 0 && g.y >= 0);
        assert!(g.x as u32 + g.latime <= 1920 && g.y as u32 + g.inaltime <= 1032);
    }

    #[test]
    fn reia_locul_de_data_trecuta_daca_monitorul_mai_exista() {
        let m = [monitor(0, 1920, 1040, 1.0), monitor(1920, 1920, 1040, 1.0)];
        let s = Dreptunghi {
            x: 2000,
            y: 50,
            latime: 1200,
            inaltime: 800,
        };
        assert_eq!(alege_geometria(Some(s), &m, Some((10, 10))), Some(s));
        // monitorul al doilea a fost scos: pe al cursorului, centrata
        let g = alege_geometria(Some(s), &m[..1], Some((10, 10))).expect("geometrie");
        assert!(g.x < 1920, "{g:?}");
    }

    #[test]
    fn locul_de_data_trecuta_se_incadreaza_in_monitor() {
        let m = [monitor(0, 1366, 728, 1.0)];
        let s = Dreptunghi {
            x: 600,
            y: 200,
            latime: 1440,
            inaltime: 900,
        };
        let g = alege_geometria(Some(s), &m, None).expect("geometrie");
        assert_eq!((g.x, g.y, g.latime, g.inaltime), (0, 0, 1366, 728));
    }

    #[test]
    fn fara_monitoare_decide_sistemul() {
        assert_eq!(alege_geometria(None, &[], Some((1, 1))), None);
        // cursorul in afara oricarui monitor: primul
        let m = [monitor(0, 1920, 1040, 1.0)];
        assert!(alege_geometria(None, &m, Some((-5000, 0))).is_some());
    }
}
