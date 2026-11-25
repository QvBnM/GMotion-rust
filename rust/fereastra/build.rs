// Compileaza interfata. Ecranele (`ui/*.slint`) nu mai sunt ferestre: le
// pune pe toate intr-o singura fereastra `Aplicatie`, generata aici, ca
// trecerea dintre ecrane sa nu inchida si sa redeschida nimic.
//
// Se strica astfel: build-ul cade cu o eroare din `aplicatie.slint` sau din
// `ecrane.rs` din OUT_DIR — vezi `build_shell.rs`.
include!("build_shell.rs");

fn main() {
    let radacina = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR");
    let ui = std::path::Path::new(&radacina).join("ui");
    let iesire = std::path::PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR"));
    println!("cargo:rerun-if-changed=build_shell.rs");
    println!("cargo:rerun-if-changed=ui");
    let citeste =
        |f: &str| std::fs::read_to_string(ui.join(f)).unwrap_or_else(|e| panic!("ui/{f}: {e}"));
    let decl: Vec<Vec<Declaratie>> = ECRANE
        .iter()
        .map(|e| {
            let d = declaratii(&citeste(e.fisier), e.componenta);
            assert!(!d.is_empty(), "{}: nicio declaratie gasita", e.componenta);
            d
        })
        .collect();
    let ui_abs = ui.display().to_string().replace('\\', "/");
    let mut structuri = Vec::new();
    for f in [
        "comune.slint",
        "conectare.slint",
        "lista.slint",
        "editor.slint",
        "deploy.slint",
    ] {
        for n in structuri_exportate(&citeste(f)) {
            structuri.push((n, f.to_string()));
        }
    }
    let slint = shell_slint(
        &ui_abs,
        &exporturi(&citeste("fereastra.slint")),
        &structuri,
        &decl,
    );
    let cale = iesire.join("aplicatie.slint");
    std::fs::write(&cale, slint).expect("aplicatie.slint nu se scrie");
    std::fs::write(iesire.join("ecrane.rs"), shell_rust(&decl)).expect("ecrane.rs nu se scrie");
    slint_build::compile(&cale).expect("aplicatie.slint nu se compileaza");
}
