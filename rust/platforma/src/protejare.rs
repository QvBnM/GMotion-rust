//! Protejarea la repaus a fisierelor de sesiune.
//!
//! **Se strica astfel:** o sesiune salvata pe un calculator nu se mai poate
//! citi pe altul — ceea ce e **comportamentul corect**, nu un bug, dar arata ca
//! unul daca mesajul nu o spune. De-aia `GM-PLAT-004` zice explicit „alt
//! utilizator sau alt calculator".
//!
//! Pe Windows: DPAPI (`CryptProtectData`), legat de utilizator si masina.
//! Nimeni altcineva nu poate citi fisierul, nici copiindu-l.
//!
//! Pe alt sistem: **nu se cripteaza**, si fisierul o spune in clar. E acceptabil
//! doar fiindca o sesiune **nu conține niciodata secrete** — vezi
//! `sesiune.rs`, unde tipul nu are loc pentru o parola. Daca vreodata ajunge un
//! secret intr-o sesiune, calea de non-Windows trebuie sa refuze salvarea, nu
//! sa scrie in clar.
//!
//! FFI scris direct, fara crate de legaturi Windows: doua functii din
//! `crypt32.dll`. Verificat la compilare de poarta, prin
//! `cargo check --target x86_64-pc-windows-msvc`.

/// Marcajul pus la inceputul fisierelor neprotejate, ca sa fie evident la
/// inspectie ce e si ce nu e criptat.
pub const MARCAJ_CLAR: &[u8] = b"GMOTION-CLAR\x00";

#[cfg(windows)]
mod platforma_nativa {
    use std::ptr;

    #[repr(C)]
    struct DataBlob {
        cb_data: u32,
        pb_data: *mut u8,
    }

    #[link(name = "crypt32")]
    extern "system" {
        fn CryptProtectData(
            p_data_in: *const DataBlob,
            sz_data_descr: *const u16,
            p_optional_entropy: *const DataBlob,
            pv_reserved: *mut core::ffi::c_void,
            p_prompt_struct: *mut core::ffi::c_void,
            dw_flags: u32,
            p_data_out: *mut DataBlob,
        ) -> i32;

        fn CryptUnprotectData(
            p_data_in: *const DataBlob,
            pp_sz_data_descr: *mut *mut u16,
            p_optional_entropy: *const DataBlob,
            pv_reserved: *mut core::ffi::c_void,
            p_prompt_struct: *mut core::ffi::c_void,
            dw_flags: u32,
            p_data_out: *mut DataBlob,
        ) -> i32;
    }

    #[link(name = "kernel32")]
    extern "system" {
        fn LocalFree(h_mem: *mut core::ffi::c_void) -> *mut core::ffi::c_void;
    }

    /// `CRYPTPROTECT_UI_FORBIDDEN`: niciodata o fereastra de dialog. Codul
    /// poate rula si fara sesiune interactiva.
    const UI_FORBIDDEN: u32 = 0x1;

    /// Partea comuna celor doua apeluri: pregatirea blobului de intrare,
    /// verificarea rezultatului si eliberarea memoriei alocate de Windows.
    ///
    /// `f` e un closure Rust, nu un pointer `extern "system"`: primele doua
    /// functii difera doar prin argumentul `pp_sz_data_descr`, pe care
    /// `CryptProtectData` nu-l are. Prima versiune incerca sa le unifice printr-un
    /// pointer de functie si nu compila pe Windows — prinsa de
    /// `cargo check --target x86_64-pc-windows-msvc` din poarta.
    fn apel(
        intrare: &[u8],
        f: impl FnOnce(*const DataBlob, *mut DataBlob) -> i32,
    ) -> Option<Vec<u8>> {
        // DataBlob cere un pointer mutabil, dar niciuna din cele doua functii
        // nu scrie in intrare.
        let mut copie = intrare.to_vec();
        let intrare_blob = DataBlob {
            cb_data: u32::try_from(copie.len()).ok()?,
            pb_data: copie.as_mut_ptr(),
        };
        let mut ieșire = DataBlob {
            cb_data: 0,
            pb_data: ptr::null_mut(),
        };
        let ok = f(&intrare_blob, &mut ieșire);
        if ok == 0 || ieșire.pb_data.is_null() {
            return None;
        }
        let rezultat =
            unsafe { std::slice::from_raw_parts(ieșire.pb_data, ieșire.cb_data as usize) }.to_vec();
        unsafe { LocalFree(ieșire.pb_data.cast()) };
        Some(rezultat)
    }

    pub fn protejeaza(clar: &[u8]) -> Option<Vec<u8>> {
        apel(clar, |intrare, ieșire| unsafe {
            CryptProtectData(
                intrare,
                ptr::null(),
                ptr::null(),
                ptr::null_mut(),
                ptr::null_mut(),
                UI_FORBIDDEN,
                ieșire,
            )
        })
    }

    pub fn deprotejeaza(protejat: &[u8]) -> Option<Vec<u8>> {
        apel(protejat, |intrare, ieșire| unsafe {
            CryptUnprotectData(
                intrare,
                ptr::null_mut(),
                ptr::null(),
                ptr::null_mut(),
                ptr::null_mut(),
                UI_FORBIDDEN,
                ieșire,
            )
        })
    }

    pub const CRIPTAT: bool = true;
}

#[cfg(not(windows))]
mod platforma_nativa {
    use super::MARCAJ_CLAR;

    pub fn protejeaza(clar: &[u8]) -> Option<Vec<u8>> {
        let mut v = MARCAJ_CLAR.to_vec();
        v.extend_from_slice(clar);
        Some(v)
    }

    pub fn deprotejeaza(protejat: &[u8]) -> Option<Vec<u8>> {
        protejat.strip_prefix(MARCAJ_CLAR).map(<[u8]>::to_vec)
    }

    pub const CRIPTAT: bool = false;
}

/// Protejeaza datele pentru stocare. `None` daca sistemul refuza.
pub fn protejeaza(clar: &[u8]) -> Option<Vec<u8>> {
    platforma_nativa::protejeaza(clar)
}

/// Inverseaza [`protejeaza`]. `None` daca datele au fost protejate de alt
/// utilizator sau pe alt calculator.
pub fn deprotejeaza(protejat: &[u8]) -> Option<Vec<u8>> {
    platforma_nativa::deprotejeaza(protejat)
}

/// Daca protejarea e criptare reala pe acest sistem. `false` doar in
/// dezvoltare, pe non-Windows.
pub fn e_criptat() -> bool {
    platforma_nativa::CRIPTAT
}

#[cfg(test)]
mod teste {
    use super::*;

    #[test]
    fn dus_intors() {
        for date in [
            &b""[..],
            b"x",
            b"{\"sesiune\": true}",
            &[0u8, 1, 2, 255, 254][..],
        ] {
            let p = protejeaza(date).expect("protejarea nu ar trebui sa cada");
            assert_eq!(deprotejeaza(&p).as_deref(), Some(date));
        }
    }

    #[test]
    fn date_straine_nu_se_deprotejeaza() {
        assert!(deprotejeaza(b"gunoi care nu vine de la noi").is_none());
    }

    #[cfg(not(windows))]
    #[test]
    fn pe_linux_marcajul_e_vizibil_si_nu_pretinde_criptare() {
        let p = protejeaza(b"ceva").unwrap();
        assert!(p.starts_with(MARCAJ_CLAR));
        assert!(!e_criptat(), "pe non-Windows nu avem criptare, si o spunem");
    }
}
