"""ui.py - Interfata SPA GMotion - SQL Diff & Deploy."""

def genereaza_pagina():
    return PAGINA

PAGINA = r"""<!DOCTYPE html><html lang="ro"><head><meta charset="utf-8">
<title>GMotion — SQL Diff &amp; Deploy</title>
<!-- FARA CDN-uri: iconite inline (SVG), fonturi de sistem, motor diff propriu.
     Aplicatia merge complet offline / pe orice server RDP fara internet. -->

<style id="dtheme">:root{
  --navy:#ffffff;--navy2:#f6f8fa;--card:#ffffff;--cardhi:#eaeef2;
  --teal:#2f81f7;--teal2:rgba(47,129,247,.14);--teal3:rgba(47,129,247,.06);
  --coral:#cf222e;--coral2:rgba(207,34,46,.12);
  --blue:#218bff;--blue2:rgba(33,139,255,.12);
  --neon:#2da44e;--neon2:rgba(45,164,78,.14);
  --amber:#9a6700;--amber2:rgba(154,103,0,.13);
  --border:rgba(47,129,247,.25);--border2:rgba(31,35,40,.09);
  --txt:#1f2328;--muted:#59636e;
  --inbg:#ffffff;--kw:#8250df;--kw2:rgba(130,80,223,.16);--kw3:rgba(130,80,223,.06);--sqlvar:#bf3989;
  --focus:#d6409f;--focus2:rgba(214,64,159,.16);--focus3:rgba(214,64,159,.07);
  --mono:ui-monospace,'Cascadia Code','Consolas','Courier New',monospace;
}</style>

<style id="mainStyles">
*{box-sizing:border-box;margin:0;padding:0}
body{background:var(--navy);color:var(--txt);font-family:-apple-system,'Segoe UI',system-ui,Roboto,sans-serif;height:100vh;overflow:hidden;font-size:13px;transition:background .3s,color .3s;padding-top:0px}
/* iconite SVG inline - inlocuiesc fontul Tabler, dimensionate ca el */
i.ti{display:inline-flex;align-items:center;justify-content:center;width:1em;height:1em;vertical-align:-0.125em;line-height:1}
svg.gi{width:1em;height:1em;display:block}
::-webkit-scrollbar{width:6px}
::-webkit-scrollbar-thumb{background:var(--teal2);border-radius:10px}

/* GRID ANIMAT */
.grid-bg{position:fixed;inset:0;background-image:linear-gradient(var(--border2) 1px,transparent 1px),linear-gradient(90deg,var(--border2) 1px,transparent 1px);background-size:60px 60px;pointer-events:none;animation:gridmove 20s linear infinite;opacity:.45}
/* pe lista si detaliu vrem fundal simplu, fara grila animata */
.grid-bg{display:none}
@keyframes gridmove{from{transform:translateY(0)}to{transform:translateY(60px)}}

/* ECRANE */
.screen{display:none;height:100vh;position:relative;overflow:hidden;flex-direction:column}
.screen.active{display:flex!important;animation:gmScreenIn .4s ease}
@keyframes gmScreenIn{from{opacity:0;transform:translateY(8px)}to{opacity:1;transform:translateY(0)}}

/* BRAND [GMOTION] */
.brand-gm{display:flex;align-items:center;gap:8px;font-family:var(--mono);font-weight:900;font-size:14px;padding:5px 12px;background:var(--card);border:1px solid var(--border);border-radius:6px;letter-spacing:0}
.gm-bracket{color:var(--teal);font-size:17px;font-weight:300;animation:blink-t 2s infinite}
.gm-dots{display:grid;grid-template-columns:repeat(2,5px);gap:3px}
.gm-dot{width:4px;height:4px;background:var(--border);border-radius:50%}
.gm-dot.a{background:var(--neon);box-shadow:0 0 6px var(--neon);animation:blink-t 1.4s infinite}
@keyframes blink-t{0%,100%{opacity:.4}50%{opacity:1}}

/* TEMA DROPDOWN */
.tema-wrap{position:relative}
.tema-btn{display:flex;align-items:center;gap:7px;background:var(--card);border:1px solid var(--border);border-radius:8px;padding:6px 12px;color:var(--teal);font-size:12px;font-weight:700;cursor:pointer;font-family:-apple-system,'Segoe UI',system-ui,Roboto,sans-serif;transition:all .2s;white-space:nowrap}
.tema-btn:hover{background:var(--teal2)}
.tema-btn i{font-size:15px}
.tema-btn .arr{font-size:11px;margin-left:2px;transition:transform .2s}
.tema-btn.open .arr{transform:rotate(180deg)}
.tema-dropdown{position:absolute;top:calc(100% + 6px);right:0;background:var(--card);border:1px solid var(--border);border-radius:10px;padding:6px;width:175px;z-index:200;display:none;box-shadow:0 16px 40px rgba(0,0,0,.45)}
.tema-dropdown.open{display:block}
.tema-item{display:flex;align-items:center;gap:8px;padding:7px 10px;border-radius:7px;cursor:pointer;font-size:11.5px;font-weight:600;color:var(--muted);transition:all .18s;border:1px solid transparent}
.tema-item:hover{background:var(--border2);color:var(--txt);border-color:var(--border)}
.tema-item.active{background:var(--teal)!important;color:#000!important;border-color:var(--teal)!important;font-weight:800}
.tema-item.active .tdot{background:#000!important}
.srv-wrap{position:relative}
.srv-dropdown{position:absolute;top:calc(100% + 6px);left:0;right:0;background:var(--card);border:1px solid var(--border);border-radius:10px;padding:6px;z-index:200;display:none;box-shadow:0 16px 40px rgba(0,0,0,.45);max-height:260px;overflow-y:auto;background-image:radial-gradient(circle,var(--border2) 1px,transparent 1px);background-size:14px 14px}
.srv-dropdown.open{display:block}
.srv-item{display:flex;align-items:center;gap:10px;padding:8px 10px;border-radius:7px;cursor:pointer;transition:all .18s;border:1px solid transparent}
.srv-item:hover{background:var(--border2);border-color:var(--border)}
.srv-item i{color:var(--teal);font-size:17px;flex-shrink:0}
.srv-info{display:flex;flex-direction:column;gap:2px;min-width:0;flex:1}
.srv-item .srv-nume{font-size:12px;font-weight:700;color:var(--txt);font-family:var(--mono);white-space:nowrap;overflow:hidden;text-overflow:ellipsis}
.srv-item .srv-sub{font-size:10px;color:var(--muted);white-space:nowrap;overflow:hidden;text-overflow:ellipsis}
.srv-badge{font-size:8.5px;font-weight:800;letter-spacing:.5px;color:var(--amber);background:var(--amber2);border:1px solid var(--amber2);border-radius:4px;padding:2px 6px;flex-shrink:0}
.srv-details{display:flex;flex-wrap:wrap;gap:6px;margin-bottom:14px}
.srv-detail-chip{display:inline-flex;align-items:center;gap:5px;font-size:10.5px;font-weight:600;color:var(--muted);background:var(--border2);border:1px solid var(--border2);border-radius:20px;padding:4px 9px}
.srv-detail-chip i{font-size:12px;color:var(--teal)}
.srv-detail-chip b{color:var(--txt);font-weight:700}
.srv-empty{padding:14px 10px;text-align:center;font-size:11.5px;color:var(--muted)}
.srv-scan{padding:14px 10px;display:flex;align-items:center;gap:8px;justify-content:center;font-size:11.5px;color:var(--muted)}
.btn-cauta{display:flex;align-items:center;justify-content:center;width:38px;flex-shrink:0;background:var(--inbg);border:1px solid var(--border);border-radius:7px;color:var(--teal);cursor:pointer;transition:all .18s}
.btn-cauta:hover{background:var(--teal2);border-color:var(--teal)}
.btn-cauta.spinning i{animation:spin .8s linear infinite}
@keyframes spin{to{transform:rotate(360deg)}}
.sesbanner {
  background: linear-gradient(135deg, var(--teal2), var(--teal3));
  border: 1px solid var(--teal);
  border-radius: 10px;
  padding: 10px 12px;
  margin-bottom: 14px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  /* Permitem wrap-ul, dar ne asigurăm că elementele se așază aerisit pe ecrane mici */
  flex-wrap: wrap; 
}

.sesbanner-top {
  display: flex;
  align-items: center;
  gap: 9px;
  /* Permite zonei de text să ocupe tot spațiul rămas */
  flex: 1 1 200px; 
  min-width: 0;
}

.sesbanner-txt {
  display: flex;
  flex-direction: column;
  gap: 3px; /* spațiul mic dintre liniile de detalii */
  flex: 1;
  min-width: 0;
}


.sesbanner-txt b {
  font-size: 13px;
  color: var(--teal);
  font-weight: bold;
  text-shadow: 0 0 5px var(--teal2); /* Un efect subtil de strălucire */
  margin-bottom: 4px;
  display: block; /* Ne asigurăm că stă pe rândul lui, deasupra detaliilor */
}

/* MODIFICAREA CHEIE: Permitem textului să treacă pe mai multe rânduri dacă este prea lung */
.sesbanner-txt span {
  font-size: 11.5px;
  color: var(--txt);
  font-weight: 500;
  white-space: normal; /* asigură vizibilitatea completă chiar dacă o linie este foarte lungă */
  word-break: break-all;
}
.sesbanner-txt span strong{color:var(--teal);font-weight:700}
.sesbanner-activ{border-color:var(--kw);background:linear-gradient(135deg,var(--kw2),var(--kw3))}
.sesbanner-activ .sesbanner-txt b{color:var(--kw);text-shadow:0 0 5px var(--kw2)}
.sesbanner-activ .sesbanner-txt span strong{color:var(--kw)}

.sesbanner-btns {
  display: flex;
  gap: 6px;
  flex-shrink: 0;
  /* Pe ecrane foarte mici, butoanele se vor alinia frumos la dreapta sau pe rând nou */
  margin-left: auto; 
}
.ses-btn{display:flex;align-items:center;gap:5px;background:var(--border2);border:1px solid var(--border);border-radius:7px;padding:6px 11px;font-size:11px;font-weight:700;color:var(--muted);cursor:pointer;font-family:inherit;transition:all .15s}
.ses-btn:hover{background:var(--border);color:var(--txt)}
.ses-btn-on{background:var(--teal);border-color:var(--teal);color:#000}
.ses-btn-on:hover{background:var(--teal);opacity:.88;color:#000}
.lch-tabs{display:flex;gap:4px;margin-top:12px;border-bottom:1px solid var(--border2)}
.lchtab{background:transparent;border:none;color:var(--muted);font-size:11.5px;font-weight:700;padding:8px 4px;cursor:pointer;display:flex;align-items:center;gap:6px;font-family:inherit;border-bottom:2px solid transparent;margin-bottom:-1px;transition:all .15s}
.lchtab:hover{color:var(--txt)}
.lchtab.on{color:var(--teal);border-bottom-color:var(--teal)}
.lchtab-badge{background:var(--teal2);color:var(--teal);border-radius:20px;padding:1px 7px;font-size:10px;font-weight:800}
.lchtab.has-ses::after{content:'';width:6px;height:6px;border-radius:50%;background:var(--kw);box-shadow:0 0 6px var(--kw);animation:blink-t 1.4s infinite}
.ses-empty{color:var(--muted);font-size:12px;padding:24px 4px;text-align:center}
.dot-dep{display:inline-flex;color:var(--teal);font-size:13px;margin-left:4px;vertical-align:-2px}
.dot-stale{display:inline-flex;color:var(--amber);font-size:13px;margin-left:4px;vertical-align:-2px}
.dep-cell{display:flex;flex-direction:column;gap:2px;min-width:0}
.dep-chip{display:inline-flex;align-items:center;gap:4px;font-size:10px;font-weight:700;white-space:nowrap;overflow:hidden;text-overflow:ellipsis}
.dep-chip.dep-da{color:var(--teal)}
.dep-chip.dep-nu{color:var(--muted);font-weight:600}
.dep-chip.dep-stale{color:var(--amber)}
.dep-chip.dep-ready{color:var(--blue)}
.dep-chip.dep-omis{color:var(--muted);font-weight:700}
.ses-hint{display:flex;align-items:center;gap:7px;font-size:11px;padding:8px 11px;border-radius:8px;margin-bottom:12px;border:1px solid var(--border2);color:var(--muted);background:var(--border2)}
.ses-hint.warn{color:var(--amber);background:var(--amber2);border-color:var(--amber2)}
.ses-hint i{font-size:14px;flex-shrink:0}
.dep-chip i{font-size:12px;flex-shrink:0}
.mod-chip{display:inline-flex;align-items:center;gap:4px;font-size:10px;font-weight:600;color:var(--muted);white-space:nowrap;overflow:hidden;text-overflow:ellipsis}
.mod-chip i{font-size:12px;flex-shrink:0;color:var(--muted)}
.mod-chip.mod-are i{color:var(--teal)}
.ses-save-info{font-size:10.5px;color:var(--muted);display:flex;align-items:center;gap:5px;margin-top:2px}
.ses-save-info i{font-size:12px;color:var(--teal)}
.ses-save-info.pulsand i{animation:pulseSave .9s ease}
@keyframes pulseSave{0%{transform:scale(1.4);color:var(--neon)}100%{transform:scale(1)}}
.ses-save-info.st-salvand i{color:var(--muted);animation:spin .8s linear infinite}
.ses-save-info.st-ok{color:var(--muted)}
.ses-save-info.st-ok i{color:var(--neon)}
.ses-save-info.st-eroare{color:var(--coral);font-weight:700}
.ses-save-info.st-eroare i{color:var(--coral)}
.tdot{width:8px;height:8px;border-radius:50%;flex-shrink:0}

/* ALIVE BADGE */
.alive{display:inline-flex;align-items:center;gap:5px;background:var(--blue2);border:1px solid var(--blue2);color:var(--blue);font-size:10px;padding:2px 8px;border-radius:20px;font-weight:700}
.alive-d{width:5px;height:5px;background:var(--blue);border-radius:50%;animation:blink-t 2s infinite}

/* ═══ S1 CONECTARE ═══ */
#ec{align-items:center;justify-content:center;overflow:hidden;padding:0;background:var(--navy)}
.gm-window{position:relative;z-index:2;width:100%;height:100%;max-width:none;max-height:none;background:var(--navy2);border:none;border-radius:0;box-shadow:none;display:flex;flex-direction:column;overflow:hidden}
.gm-body{flex:1;display:flex;min-height:0}
.gm-main{flex:1;display:flex;min-width:0;min-height:0}
.gm-formcol {
  width: 420px; /* Lățimea ta curentă */
  height: 100% !important;
  max-height: 100% !important;
  display: flex !important;
  flex-direction: column !important;
  box-sizing: border-box !important;
  overflow: hidden !important;

  /* MODIFICAREA DE PADDING: */
  padding: 5px !important; /* Pune 12px spațiu gol pe toate cele 4 laturi (sus, dreapta, jos, stânga) */
}
.gm-termcol{flex:1;display:flex;flex-direction:column;min-width:0;min-height:0}

#ec .brand-gm{background:var(--card);border:1px solid var(--border);border-radius:7px}

.lcard{width:100%;background:transparent;border:none;border-radius:0;box-shadow:none;display:flex;flex-direction:column;flex:1;min-height:0}
.lch{padding:0 0 16px}
.brow{display:flex;align-items:center;gap:12px;margin-bottom:14px}
.bnm{font-size:16px;font-weight:800;letter-spacing:-.5px;color:var(--txt)}
.bsub{font-size:10px;color:var(--muted);margin-top:1px}
.aseg{display:flex;background:var(--card);border:1px solid var(--border);border-radius:9px;padding:3px;margin-bottom:14px}
.abtn{flex:1;border:none;background:transparent;color:var(--muted);font-size:11.5px;font-weight:600;padding:8px;border-radius:7px;cursor:pointer;display:flex;align-items:center;justify-content:center;gap:5px;transition:all .2s;font-family:inherit}
.abtn.on{background:var(--kw2);color:var(--kw);border:1px solid var(--kw)}
.abtn i{font-size:14px}
.lcb{padding:0;overflow-y:auto;flex:1}
.fld{margin-bottom:12px}
.fld label{display:block;font-size:9.5px;font-weight:700;color:var(--muted);letter-spacing:.8px;text-transform:uppercase;margin-bottom:5px}
.fld label i{margin-right:4px;font-size:12px}
.fld input{width:100%;background:var(--inbg);border:1px solid var(--border);border-radius:8px;padding:10px 12px;color:var(--txt);font-family:var(--mono);font-size:12px;outline:none;transition:all .2s}
.fld input::placeholder{color:var(--muted)}
.fld input:focus{border-color:var(--kw);background:var(--kw3);box-shadow:0 0 0 3px var(--kw2)}
.fld input[readonly]{opacity:.55;cursor:not-allowed;background:var(--border2)}
.fld textarea{width:100%;background:var(--inbg);border:1px solid var(--border);border-radius:8px;padding:9px 11px;color:var(--txt);font-family:var(--mono);font-size:11.5px;line-height:1.5;outline:none;transition:all .2s;min-height:120px;max-height:220px}
.fld textarea::placeholder{color:var(--muted)}
.fld textarea:focus{border-color:var(--kw);background:var(--kw3);box-shadow:0 0 0 3px var(--kw2)}
.fld-lbl-row{display:flex;align-items:center;justify-content:space-between;margin-bottom:5px}
.fld-lbl-row label{margin-bottom:0}
.fld-mini-btn{display:inline-flex;align-items:center;gap:3px;background:var(--border2);border:1px solid var(--border);border-radius:5px;padding:2px 7px;font-size:9.5px;font-weight:700;color:var(--muted);cursor:pointer;text-transform:none;letter-spacing:0;transition:all .15s}
.fld-mini-btn:hover{background:var(--teal2);color:var(--teal);border-color:var(--teal)}
.fld-mini-btn i{font-size:11px}
.frow{display:grid;grid-template-columns:1fr 1fr;gap:10px}
.stbar{display:flex;align-items:center;gap:8px;padding:9px 12px;border-radius:9px;font-size:11.5px;margin-bottom:14px;border:1px solid;display:none}
.stbar.ok{border-color:var(--neon);background:var(--neon2);color:var(--neon);display:flex}
.stbar.err{border-color:var(--coral);background:var(--coral2);color:var(--coral);display:flex}
.stbar.load{border-color:var(--amber);background:var(--amber2);color:var(--amber);display:flex}
.stbar i{font-size:15px;flex-shrink:0}

.dbt{display:grid;grid-template-columns:44px 1fr 52px;padding:0 12px;font-size:9px;font-weight:700;color:var(--muted);letter-spacing:.8px;text-transform:uppercase;margin-bottom:6px}
.dbl{border:1px solid var(--border);border-radius:10px;overflow:hidden;max-height:clamp(120px,26vh,320px);overflow-y:auto;margin-bottom:16px;background:var(--border2);opacity:.4;pointer-events:none;transition:opacity .3s,filter .3s}
.dbl.activ{opacity:1;pointer-events:auto}
.dbl.rulat{opacity:1;pointer-events:none;filter:grayscale(1);background:var(--border2)}
.dbrow{display:grid;grid-template-columns:44px 1fr 52px;align-items:center;padding:10px 12px;border-bottom:1px solid var(--border2);transition:background .15s;cursor:pointer}
.dbrow:last-child{border-bottom:none}
.dbrow:hover{background:var(--kw3)}
.dbrow.ref-sel{background:var(--amber2);box-shadow:inset 3px 0 0 var(--amber)}
.dbrow.ref-sel .dbnm{color:var(--amber);font-weight:700}
.dbrow.ref-sel:hover{background:var(--amber2)}
.chkb{font-size:18px;color:var(--muted);transition:all .2s}
.chkb.on{color:var(--neon);text-shadow:0 0 8px var(--neon2)}
.chkb.lock{color:var(--border);cursor:not-allowed}
.dbnm{font-family:var(--mono);font-size:12px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;color:var(--txt)}
.dbnm.ref{color:var(--amber);font-weight:700}
.radb{font-size:18px;color:var(--muted);display:flex;justify-content:flex-end;transition:all .2s}
.radb.on{color:var(--amber)!important;filter:drop-shadow(0 0 1px rgba(0,0,0,.4))}
.btn-test{width:100%;display:flex;align-items:center;justify-content:center;gap:7px;background:var(--card);border:1px solid var(--border);border-radius:9px;padding:11px;color:var(--kw);font-size:13px;font-weight:700;cursor:pointer;margin-bottom:12px;font-family:inherit;transition:all .2s}
.btn-test:hover{background:var(--kw2);border-color:var(--kw)}
.btn-test:disabled{opacity:.4;cursor:not-allowed}
.btn-run{width:100%;padding:13px;background:linear-gradient(135deg,var(--kw),var(--blue));border:none;border-radius:10px;color:#fff;font-size:14px;font-weight:800;cursor:pointer;display:flex;align-items:center;justify-content:center;gap:8px;transition:all .2s;font-family:inherit;box-shadow:0 8px 24px var(--blue2)}
.btn-run i{font-size:17px}
.btn-run:hover{transform:translateY(-1px);box-shadow:0 10px 28px var(--blue2)}
.btn-run:disabled{opacity:.5;cursor:not-allowed;transform:none;box-shadow:none}
.pgbar{height:6px;background:rgba(255,255,255,.09);border-radius:3px;overflow:hidden;margin-top:10px;display:none}
.pgbar.show{display:block}
.pgfill{height:100%;background:linear-gradient(90deg,var(--teal),var(--blue));width:0;transition:width .3s;border-radius:3px}
.pginfo{display:none;justify-content:space-between;font-size:11px;color:var(--muted);margin-top:6px}
.pginfo.show{display:flex}
.lock-msg{font-size:11px;color:var(--muted);text-align:center;margin-top:10px}
.lock-msg a{color:var(--coral);text-decoration:none;font-weight:700}
.lock-msg a:hover{text-decoration:underline}

.term-wrap{display:flex;flex-direction:column;flex:1 1 0;min-height:0;background:var(--card)}
.term-hd{display:flex;align-items:center;gap:8px;padding:10px 18px;background:rgba(0,0,0,.12);border-bottom:1px solid var(--border2);font-size:9.5px;font-weight:600;color:var(--muted);letter-spacing:1px;text-transform:uppercase;flex-shrink:0;font-family:var(--mono)}
.term-hd span{flex:1;color:var(--muted)}
.term-clr{background:none;border:none;color:var(--muted);cursor:pointer;padding:3px 8px;border-radius:5px;font-size:9.5px;font-family:var(--mono);transition:all .2s}
.term-clr:hover{color:var(--txt);background:var(--kw2)}
.term-body{margin:0;padding:16px 20px 18px;font-family:var(--mono);font-size:12.5px;line-height:1.85;color:var(--txt);background:transparent;flex:1 1 0;min-height:0;overflow-y:auto;white-space:pre-wrap;word-break:break-all}
.term-line{display:inline}
.term-prompt{color:inherit;opacity:.7;font-weight:800;margin-right:2px}
.t-info{color:var(--muted)}
.t-ok{color:var(--neon);text-shadow:0 0 6px var(--neon2)}
.t-err{color:var(--coral);text-shadow:0 0 6px var(--coral2)}
.t-warn{color:var(--amber);text-shadow:0 0 6px var(--amber2)}
.term-cursor{color:var(--kw);font-weight:700;animation:blink-t 1s infinite}
.term-ts{color:var(--txt);font-weight:700;opacity:.95}
@keyframes termGlow{0%,100%{box-shadow:inset 0 0 0 1px var(--kw2)}50%{box-shadow:inset 0 0 0 1px var(--teal2),0 0 26px var(--teal3)}}
.term-wrap.term-pulse{animation:termGlow 1.1s ease-in-out infinite}
.gm-transition{position:absolute;inset:0;display:flex;flex-direction:column;align-items:center;justify-content:center;gap:14px;background:rgba(4,6,12,.9);opacity:0;pointer-events:none;transition:opacity .35s ease;z-index:50}
.gm-transition.show{opacity:1;pointer-events:auto}
.gm-trans-spin{width:32px;height:32px;border-radius:50%;border:3px solid var(--border);border-top-color:var(--kw);animation:gmspin .8s linear infinite}
.gm-trans-msg{font-size:12px;color:var(--muted);letter-spacing:.3px;font-family:var(--mono)}
@keyframes gmspin{to{transform:rotate(360deg)}}
.spin{display:inline-block;width:11px;height:11px;border-radius:50%;border:2px solid var(--border);border-top-color:currentColor;animation:gmspin .7s linear infinite;vertical-align:-1.5px}
.term-empty{color:var(--muted);font-style:italic;font-size:11px;padding:18px 0;letter-spacing:.3px}
/* ═══ LEGENDA TERMINAL - schematic + timeline ═══ */
.term-legend{
  position:relative;padding:20px 20px 16px;border-radius:10px;
  background:var(--card);border:1px solid var(--border2);
  background-image:radial-gradient(circle,var(--border2) 1px,transparent 1px);
  background-size:16px 16px;
  font-style:normal;white-space:normal;
}
.tl-corner{position:absolute;width:14px;height:14px;border:2px solid var(--teal);pointer-events:none}
.tl-c1{top:-1px;left:-1px;border-right:none;border-bottom:none;border-radius:8px 0 0 0}
.tl-c2{top:-1px;right:-1px;border-left:none;border-bottom:none;border-radius:0 8px 0 0}
.tl-c3{bottom:-1px;left:-1px;border-right:none;border-top:none;border-radius:0 0 0 8px}
.tl-c4{bottom:-1px;right:-1px;border-left:none;border-top:none;border-radius:0 0 8px 0}
.tl-sec{display:flex;gap:12px;padding:0 0 16px}
.tl-sec:last-child{padding-bottom:0}
.tl-rail{display:flex;flex-direction:column;align-items:center;flex-shrink:0;width:8px;padding-top:3px}
.tl-dot{width:8px;height:8px;border-radius:50%;background:var(--teal);box-shadow:0 0 6px var(--teal);flex-shrink:0}
.tl-line{flex:1;width:1px;background:var(--border2);margin-top:4px}
.tl-body{flex:1;min-width:0;opacity:0;transform:translateX(-6px);animation:tlIn .4s ease forwards}
.tl-sec:nth-child(2) .tl-body{animation-delay:.03s}
.tl-sec:nth-child(3) .tl-body{animation-delay:.1s}
.tl-sec:nth-child(4) .tl-body{animation-delay:.17s}
.tl-sec:nth-child(5) .tl-body{animation-delay:.24s}
.tl-sec:nth-child(6) .tl-body{animation-delay:.31s}
@keyframes tlIn{to{opacity:1;transform:translateX(0)}}
.tl-tag{font-family:var(--mono);font-size:10px;letter-spacing:1px;color:var(--kw);opacity:.8;margin-bottom:2px}
.tl-title{font-size:12px;font-weight:800;letter-spacing:.2px;color:var(--txt);margin-bottom:4px}
.tl-desc{font-size:12px;line-height:1.65;color:var(--muted)}
.tl-desc b{color:var(--txt);font-weight:700}
.tl-tokens{display:flex;flex-wrap:wrap;gap:6px 8px;margin-top:4px}
.tl-tok{display:inline-flex;align-items:center;gap:6px;font-size:11px;font-weight:600;color:var(--txt);background:var(--border2);border:1px solid var(--border2);border-radius:20px;padding:4px 10px 4px 8px}
.tl-tok .d{width:8px;height:8px;border-radius:50%;flex-shrink:0}
.tl-hint{font-size:11.5px;color:var(--muted);font-style:italic}
.tl-hint b{color:var(--txt);font-style:normal;font-weight:700}
.tl-cursor{display:inline-block;width:6px;height:12px;background:var(--teal);vertical-align:-2px;margin-left:2px;animation:blink-t 1s infinite}
#ec ::-webkit-scrollbar-thumb{background:var(--border);border-radius:10px}
#ec ::-webkit-scrollbar-thumb:hover,.term-body::-webkit-scrollbar-thumb{background:var(--kw)}
/* ═══ S2 LISTA ═══ */
#el{background:var(--navy);min-height:0}
.ltop{padding:10px 20px;background:var(--navy2);border-bottom:1px solid var(--border2);display:flex;flex:0 0 auto;position:relative;z-index:80}
.ltop-row1{display:flex;align-items:center;gap:12px;width:100%}
.ltop-title{flex:1;min-width:0}
.ltop-stats{display:flex;flex-direction:column;gap:5px;align-items:flex-start;flex-shrink:0}
.lstats-prog{display:flex;gap:8px;flex-wrap:wrap}
.ltop-actions{display:flex;flex-direction:column;align-items:flex-end;gap:6px;flex-shrink:0;margin-left:auto}
.ltitle{font-size:14px;font-weight:800}
.lsub{font-size:11px;color:var(--muted)}
.lstats{display:flex;gap:8px;align-items:center;flex-wrap:wrap}
.prog-chip{display:flex;align-items:center;gap:6px;background:rgba(255,255,255,.05);border:1px solid var(--border);border-radius:20px;padding:5px 10px;font-size:11px;font-weight:700;color:var(--muted);white-space:nowrap}
.prog-chip i{font-size:13px}
.prog-mini{width:40px;height:5px;border-radius:3px;background:rgba(255,255,255,.1);overflow:hidden;flex-shrink:0}
.prog-mini-fill{height:100%;background:linear-gradient(90deg,var(--teal),var(--blue));border-radius:3px;transition:width .3s}
.prog-mini-fill.dep{background:linear-gradient(90deg,var(--kw),var(--blue))}
.prog-chip-val{color:var(--txt)}
.btn-deconectare{background:var(--kw3);border:1px solid var(--kw);color:var(--kw)}
.btn-deconectare:hover{background:var(--kw2);color:var(--txt)}
.btn-deconectare i{color:var(--kw)}
.scp{display:flex;align-items:center;gap:5px;padding:5px 11px;border-radius:7px;font-size:11px;font-weight:700;border:1px solid}
.scp i{font-size:14px}
.scp-r{background:var(--coral2);color:var(--coral);border-color:rgba(255,0,85,.2)}
.scp-a{background:var(--amber2);color:var(--amber);border-color:rgba(255,183,0,.2)}
.scp-g{background:var(--neon2);color:var(--neon);border-color:rgba(57,255,20,.2)}
.lflt{padding:10px 20px;background:var(--navy2);border-bottom:1px solid var(--border);display:flex;gap:10px;align-items:center;flex-shrink:0;position:relative;z-index:5;flex-wrap:wrap}
.finw{background:var(--inbg);border:1px solid var(--border);border-radius:7px;padding:7px 11px;display:flex;align-items:center;gap:7px;width:190px}
.finw i{color:var(--muted);font-size:15px}
.finw input{background:none;border:none;color:var(--txt);font-size:12px;outline:none;width:100%;font-family:inherit}
.fsel{background:var(--inbg);border:1px solid var(--border);border-radius:7px;padding:7px 11px;color:var(--txt);outline:none;font-size:12px;font-family:inherit}
.fsel option{background:var(--navy2);color:var(--txt)}
.frange{display:flex;align-items:center;gap:5px;background:var(--inbg);border:1px solid var(--border);border-radius:7px;padding:4px 9px;font-size:11px;color:var(--muted)}
.frange input{width:46px;background:transparent;border:none;color:var(--teal);font-family:var(--mono);font-size:11.5px;outline:none;text-align:center;font-weight:700}
.frange input::-webkit-outer-spin-button,.frange input::-webkit-inner-spin-button{-webkit-appearance:none;margin:0}
.bcsv{display:flex;align-items:center;gap:5px;background:rgba(255,255,255,.05);border:1px solid var(--border);border-radius:7px;padding:8px 12px;color:var(--teal);font-size:12px;font-weight:700;cursor:pointer;font-family:inherit;transition:all .2s}
.bcsv:hover{background:var(--teal2)}
.bdep{margin-left:auto;display:flex;align-items:center;gap:6px;background:linear-gradient(135deg,var(--teal),var(--blue));border:none;border-radius:7px;padding:8px 14px;color:#040811;font-size:12px;font-weight:800;cursor:pointer;font-family:inherit}
.bdep:disabled{opacity:.4;cursor:not-allowed}
#el{--cw-idx:38px;--cw-obj:300px;--cw-baza:150px;--cw-tip:52px;--cw-scor:130px;--cw-dif:170px;--cw-stare:60px;--cw-mod:150px;--cw-dep:172px;--cw-act:78px}
.lhd,.lrow{display:grid;grid-template-columns:var(--cw-idx) var(--cw-obj) var(--cw-baza) var(--cw-tip) var(--cw-scor) var(--cw-dif) var(--cw-stare) var(--cw-mod) var(--cw-dep) var(--cw-act);align-items:center;padding:0 20px}
.lhd{background:rgba(0,0,0,.25);border-bottom:1px solid var(--border2);flex-shrink:0;position:relative;z-index:5}
.lh{padding:10px 8px;font-size:10px;font-weight:700;color:var(--muted);text-transform:uppercase;letter-spacing:.8px;display:flex;align-items:center;gap:4px;cursor:pointer;user-select:none;position:relative;min-width:0;overflow:hidden;white-space:nowrap}
.lh:hover{color:var(--teal)}
.lh .sarr{font-size:11px;color:var(--teal)}
/* maner de redimensionare intre coloane */
.col-grip{position:absolute;top:0;right:-4px;width:8px;height:100%;cursor:col-resize;z-index:6}
.col-grip::after{content:'';position:absolute;top:20%;bottom:20%;right:3px;width:2px;background:var(--border2);transition:background .15s}
.col-grip:hover::after,.col-grip.drag::after{background:var(--teal)}
.lbdy{flex:1 1 0;min-height:0;overflow-y:auto;overflow-x:hidden;padding-bottom:40px;position:relative;z-index:2;contain:strict;overflow-anchor:none;overscroll-behavior:contain}
#rows{contain:content}
.lrow{height:38px;border-bottom:1px solid var(--border2);cursor:pointer;transition:background .15s,transform .15s,box-shadow .15s;position:relative}
.lrow.activ{background:var(--teal3)!important;box-shadow:inset 4px 0 0 var(--teal),0 0 14px var(--teal2);border-bottom:1px solid var(--teal);position:relative;animation:activPulse 1.6s ease-in-out 2}
.lrow.activ:hover{background:var(--teal3)!important}
@keyframes activPulse{0%,100%{box-shadow:inset 4px 0 0 var(--teal),0 0 6px var(--teal2)}50%{box-shadow:inset 4px 0 0 var(--teal),0 0 26px var(--teal)}}
.lrow:hover{background:var(--cardhi);transform:translateX(4px);box-shadow:0 0 16px var(--teal2)}
.activ-badge{position:absolute;left:-9px;top:50%;transform:translateY(-50%);width:20px;height:20px;
  background:var(--teal);color:#000;border-radius:50%;display:flex;align-items:center;justify-content:center;
  box-shadow:0 0 8px var(--teal);z-index:2}
.activ-badge i{font-size:12px}
.ld{padding:0 8px;font-size:12px;display:flex;align-items:center;gap:6px;min-width:0}
.lidx{font-family:var(--mono);color:var(--muted);font-weight:600;font-size:11px}
.onm{font-family:var(--mono);font-size:11.5px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.baza-nm{font-size:11px;color:var(--muted);overflow:hidden;text-overflow:ellipsis;white-space:nowrap;font-family:var(--mono)}
.tip-chip{font-size:9px;font-weight:900;letter-spacing:.5px;padding:2px 6px;border-radius:4px;border:1px solid var(--border2);color:var(--muted);background:var(--border2)}
.dot-rez{width:13px;height:13px;border-radius:50%;flex-shrink:0;margin:0 auto}
.dot-rez.on{background:var(--neon);box-shadow:0 0 9px var(--neon)}
.dot-rez.off{background:transparent;border:2px solid var(--muted);opacity:.55}
.dot-rez.omis{background:var(--muted);box-shadow:0 0 6px rgba(89,99,110,.4)}
.scbar{display:flex;align-items:center;gap:6px;width:100%}
.scbg{flex:1;height:4px;background:rgba(255,255,255,.06);border-radius:2px;overflow:hidden}
.scf{height:100%;border-radius:2px}
.scv{font-size:11px;font-family:var(--mono);min-width:38px;text-align:right}
.bta{background:var(--teal3);border:1px solid var(--border);color:var(--teal);padding:5px 10px;border-radius:6px;font-size:11px;font-weight:700;cursor:pointer;font-family:inherit;transition:all .15s}
.bta:hover{background:var(--teal2)}
.bara-jos{padding:10px 20px;background:var(--navy2);border-top:1px solid var(--border2);flex-shrink:0;display:flex;align-items:center;gap:12px;position:relative;z-index:5}
.dep-overlay{position:fixed;inset:0;background:rgba(0,0,0,.7);z-index:300;display:none;align-items:center;justify-content:center}
.dep-overlay.on{display:flex}
.dep-modal{background:var(--card);border:1px solid var(--border);border-radius:14px;width:min(480px,92vw);box-shadow:0 24px 70px rgba(0,0,0,.4);overflow:hidden}
.dep-head{padding:14px 20px;background:var(--navy2);border-bottom:1px solid var(--border2);font-size:14px;font-weight:800;color:var(--teal);display:flex;align-items:center;gap:8px}
.dep-body{padding:16px 20px}
.dep-q{font-size:11px;font-weight:700;color:var(--muted);text-transform:uppercase;letter-spacing:.6px;margin-bottom:10px}
.dep-opt{display:flex;gap:10px;align-items:flex-start;padding:10px 12px;border:1px solid var(--border2);border-radius:9px;margin-bottom:8px;cursor:pointer;transition:all .15s}
.dep-opt:hover{background:var(--teal3);border-color:var(--border)}
.dep-opt input[type=radio]{margin-top:2px;accent-color:var(--teal)}
.dep-opt b{font-size:12.5px;color:var(--txt)}
.dep-opt small{color:var(--muted);font-size:11px}
.dep-opt input[type=number]{width:56px;background:var(--inbg);border:1px solid var(--border2);border-radius:6px;padding:2px 6px;color:var(--txt);font-family:var(--mono);font-size:12px;margin-left:6px}
.dep-foot{padding:14px 20px;border-top:1px solid var(--border2);display:flex;justify-content:flex-end;gap:10px}
.dep-foot button{border:1px solid;border-radius:8px;padding:8px 16px;font-size:12px;font-weight:700;cursor:pointer;font-family:inherit;display:flex;align-items:center;gap:6px}
.dep-cancel{background:transparent;border-color:var(--border2);color:var(--muted)}
.dep-cancel:hover{border-color:var(--coral);color:var(--coral)}
.dep-go{background:linear-gradient(135deg,var(--teal),var(--blue));border:none;color:#fff}
.dep-go:hover{opacity:.92}
.cnt{font-size:12px;color:var(--muted)}

/* ═══ S3 DETALIU ═══ */
#ed{background:var(--navy)}
.dtp{padding:10px 20px;background:var(--navy2);border-bottom:1px solid var(--border2);display:flex;align-items:center;gap:10px;flex-shrink:0;position:relative;z-index:5}
.bbk{background:transparent;border:1px solid var(--border2);color:var(--txt);padding:7px 14px;border-radius:7px;font-size:11.5px;font-weight:700;cursor:pointer;display:flex;align-items:center;gap:5px;font-family:inherit;transition:all .2s}
.bbk:hover{border-color:var(--teal);color:var(--teal)}
.det-obj{font-family:var(--mono);font-size:13px;font-weight:700}
.det-meta{font-size:10.5px;color:var(--muted);margin-top:1px}
.bconcl{margin-left:auto;display:flex;align-items:center;gap:7px;background:rgba(255,255,255,.03);border:1.5px solid var(--border2);color:var(--muted);padding:9px 15px;border-radius:8px;font-size:12px;font-weight:700;cursor:pointer;font-family:inherit;transition:all .25s}
.bconcl.done{background:var(--neon)!important;border-color:var(--neon)!important;color:#000!important;box-shadow:0 0 16px var(--neon2)!important}
.bconcl i{font-size:16px}
.bconcl.bomite{margin-left:8px}
.bconcl.bomite.activ-omis{background:var(--muted)!important;border-color:var(--muted)!important;color:#fff!important;box-shadow:0 0 14px rgba(89,99,110,.4)!important}
.lvbar{padding:5px 20px;background:var(--teal3);border-bottom:1px solid var(--border);display:flex;align-items:center;gap:8px;font-size:10.5px;color:var(--teal);flex-shrink:0;position:relative;z-index:5}
.lvdot{width:6px;height:6px;background:var(--teal);border-radius:50%;animation:blink-t 1.4s infinite;box-shadow:0 0 6px var(--teal)}
.dtb{padding:8px 20px;background:rgba(0,0,0,.2);border-bottom:1px solid var(--border2);display:flex;align-items:center;gap:7px;flex-shrink:0;position:relative;z-index:5}
.tbb{border:1px solid var(--border2);background:rgba(255,255,255,.04);color:var(--txt);padding:5px 11px;border-radius:6px;font-size:11px;font-weight:600;cursor:pointer;display:flex;align-items:center;gap:5px;font-family:inherit;transition:all .15s}
.tbb i{font-size:13px}
.tbb:hover{background:rgba(255,255,255,.08)}
.tbb.grn{background:var(--neon2);border-color:var(--neon);color:var(--neon)}
.tbb.grn:hover{background:rgba(57,255,20,.25)}
.tbb.blu{background:var(--teal3);border-color:var(--border);color:var(--teal)}
.tsep{width:1px;height:16px;background:var(--border2)}
.nind{color:var(--amber);font-size:11.5px;font-family:var(--mono);min-width:36px;text-align:center}
.tinf{font-size:11px;color:var(--muted)}
.tcnt{color:var(--teal);font-weight:700}
.detWrap{flex:1 1 0;min-height:0;position:relative;display:flex;flex-direction:column}
.dcols{flex:1 1 0;min-height:0;grid-template-columns:1fr 46px 1fr;display:grid;position:relative;z-index:2}
.dgut{position:relative;min-height:0;overflow:hidden;background:var(--card)}
.gmb-overlay{position:absolute;left:16px;right:16px;z-index:30;display:none;
  box-shadow:0 10px 30px rgba(0,0,0,.4);border-radius:10px;pointer-events:auto}
.gmb-overlay.on{display:block}
.gmb-overlay .gmb{border-radius:10px;flex-wrap:wrap}
.dgut svg{position:absolute;top:0;left:0;width:100%;height:100%;pointer-events:none}
.dcol{display:flex;flex-direction:column;overflow:hidden;min-height:0;min-width:0;position:relative}
.dcol+.dcol{border-left:none}


.chd{padding:9px 14px;border-bottom:1px solid var(--border2);font-size:10px;font-weight:800;display:flex;align-items:center;gap:7px;letter-spacing:.8px;text-transform:uppercase;flex-shrink:0}
.chd.src{background:var(--amber2);color:var(--amber)}
.chd.wrk{background:var(--teal3);color:var(--teal)}
.ctag{font-size:8px;padding:2px 5px;border-radius:3px;font-weight:900;letter-spacing:.5px;background:white;color:#000;margin-right:2px}
.cscrl{flex:1 1 0;min-height:0;overflow-y:auto;overflow-x:hidden;padding-bottom:40px;contain:content;overscroll-behavior:contain;overflow-anchor:none}
.eof-marker{margin:14px 14px 0;padding-top:10px;border-top:1px dashed var(--border2);
  display:flex;align-items:center;gap:6px;font-size:10px;font-weight:800;letter-spacing:.8px;
  text-transform:uppercase;font-family:var(--mono)}
.eof-marker.eof-src{color:var(--amber)}
.eof-marker.eof-tinta{color:var(--teal)}
.eof-marker i{font-size:14px}
td.eof-marker{display:table-cell}
/* scrollbar-ul panoului sursa pe stanga, nu langa gutter/diagrama - trick:
   container in rtl (scrollbar trece in stanga), continutul inapoi in ltr */
#scSrc{direction:rtl}
#scSrc>#dSrc{direction:ltr}
/* ── EDITOR LIVE (dreapta) stil Araxis: textarea peste strat evidentiat ── */
.copy-src{margin-left:auto;display:flex;align-items:center;gap:4px;background:rgba(255,255,255,.06);border:1px solid var(--border2);color:var(--txt);font-size:10px;font-weight:700;padding:3px 8px;border-radius:5px;cursor:pointer;font-family:inherit;letter-spacing:.3px}
.copy-src:hover{background:var(--teal3);color:var(--teal);border-color:var(--teal)}
.edit-hint{margin-left:10px;font-size:9px;font-weight:700;color:var(--neon);letter-spacing:.5px;opacity:.8}
.leg-mini{margin-left:10px;display:flex;align-items:center;gap:6px;font-size:9px;color:var(--muted);font-weight:600}
.leg-mini .lm{width:9px;height:9px;border-radius:2px;display:inline-block;margin-right:2px}
.leg-mini .lm-mod{background:var(--amber)}
.leg-mini .lm-adus{background:var(--neon)}
.leg-mini .lm-user{background:var(--kw)}
.edit-wrap{position:relative}
.edit-hl,.edit-ta{margin:0;padding:9px 14px 9px 84px;font-family:var(--mono);font-size:11px;line-height:1.7;white-space:pre;tab-size:4;border:0;box-sizing:border-box;width:100%}
.edit-hl{position:absolute;inset:0;pointer-events:none;color:var(--txt);overflow:visible;z-index:1}
.edit-ta{position:relative;z-index:2;background:transparent;color:transparent;caret-color:var(--teal);resize:none;outline:none;overflow-x:auto;overflow-y:auto;display:block}
.edit-ta::selection{background:rgba(0,240,255,.25)}
/* markere pe blocuri, in stratul evidentiat */
.hl-line{display:block;width:calc(100% + 84px);min-height:1em;position:relative;margin-left:-84px;padding-left:84px;box-sizing:border-box}
/* numar de linie + marker de diferenta in gutter (stanga liniei) */
.hl-line .ln{position:absolute;left:6px;width:38px;text-align:right;color:var(--muted);opacity:.55;font-size:10px;user-select:none;pointer-events:none}
.hl-line .mk{position:absolute;left:50px;top:50%;transform:translateY(-50%);height:60%;width:3px;border-radius:2px}
.hl-line.d-mod .mk{background:var(--amber)}
.hl-line.d-adus .mk{background:var(--neon)}
.hl-line.d-user .mk{background:var(--kw)}
/* fundal plin pe toata latimea pt fiecare categorie - marcheaza clar TOATE
   tipurile de diferenta (modificat/adus din sursa/scris manual), fara chenar -
   doar culoare, ca sa nu arate ca o caseta plutitoare separata de restul textului.
   bara de accent pe stanga: box-shadow (nu afecteaza latimea/inaltimea reala a
   randului, deci nu poate rupe sincronizarea cu textarea) */
.hl-line.d-mod{background:var(--amber2);box-shadow:inset 3px 0 0 var(--amber)}
.hl-line.d-adus{background:var(--neon2);box-shadow:inset 3px 0 0 var(--neon)}
.hl-line.d-user{background:var(--kw2);box-shadow:inset 3px 0 0 var(--kw)}
/* chenar pe primul/ultimul rand al fiecarui grup de diferenta - separa vizual
   blocurile alaturate de acelasi tip, ca sa nu para o singura pata continua */
.hl-line.blk-first{border-top:1px solid var(--border2)}
.hl-line.blk-last{border-bottom:1px solid var(--border2)}
.hl-line.d-mod.blk-first{border-top-color:var(--amber)}
.hl-line.d-mod.blk-last{border-bottom-color:var(--amber)}
.hl-line.d-adus.blk-first{border-top-color:var(--neon)}
.hl-line.d-adus.blk-last{border-bottom-color:var(--neon)}
.hl-line.d-user.blk-first{border-top-color:var(--kw);border-top-style:dashed}
.hl-line.d-user.blk-last{border-bottom-color:var(--kw);border-bottom-style:dashed}
/* granita punctului de inserare in tinta: linie punctata + text explicativ,
   pe randul gol REAL (vezi _asigurareSpatiiPentruInserari) care exista acum
   garantat acolo - deci nu se mai poate suprapune peste cod real */
.hl-line.ins-marker{position:relative;background-image:repeating-linear-gradient(135deg, color-mix(in srgb, var(--coral) 16%, transparent) 0 6px, transparent 6px 12px)}
.hl-line.ins-marker::before{content:'';position:absolute;left:0;right:0;top:-1px;height:2px;z-index:2;
  background-image:repeating-linear-gradient(90deg, var(--coral) 0 6px, transparent 6px 10px);
  pointer-events:none}
.hl-line.ins-marker::after{content:'-- aici va fi adaugat (bloc nou din sursa)';position:absolute;
  left:84px;top:2px;z-index:2;font-style:italic;font-size:10px;font-family:var(--mono);
  color:var(--coral);pointer-events:none}
/* nuanta alternanta intre blocuri consecutive de acelasi tip - fara sa schimbe
   semantica de culoare (tipul ramane acelasi), doar ca sa nu se citeasca ca
   o singura pata continua cand 2+ blocuri de acelasi tip sunt alaturate */
/* al doilea bloc consecutiv de acelasi tip primeste o culoare diferita de-a
   binelea (nu doar mai inchisa/deschisa), ca doua blocuri MODIFICAT alaturate
   sa nu se citeasca drept unul singur continuu. Rosu/coral e evitat pt
   MODIFICAT (e deja folosit pt "sters"/DOAR SURSA in alta parte a UI) */
.hl-line.d-mod.grp-alt{background:var(--blue2);box-shadow:inset 3px 0 0 var(--blue)}
.hl-line.d-mod.grp-alt .mk{background:var(--blue)}
.hl-line.d-adus.grp-alt{background:var(--coral2);box-shadow:inset 3px 0 0 var(--coral)}
.hl-line.d-adus.grp-alt .mk{background:var(--coral)}
.hl-line.d-user.grp-alt{background:var(--teal2);box-shadow:inset 3px 0 0 var(--teal)}
.hl-line.d-user.grp-alt .mk{background:var(--teal)}
.grp-tag.tag-mod.grp-alt{color:var(--blue);border-color:var(--blue);background:var(--blue2)}
.grp-tag.tag-adus.grp-alt{color:var(--coral);border-color:var(--coral);background:var(--coral2)}
.grp-tag.tag-user.grp-alt{color:var(--teal);border-color:var(--teal);background:var(--teal2)}
/* eticheta n/total pe primul rand al fiecarui grup - acelasi limbaj vizual ca .blk-poz din stanga */
.grp-badge{position:absolute;right:6px;top:1px;font-size:9px;font-weight:800;color:var(--muted);
  background:var(--card);border:1px solid var(--border2);border-radius:4px;padding:1px 5px;
  font-family:var(--mono);z-index:3;line-height:1.3}
/* IMPORTANT: eticheta NU mai adauga inaltime liniei (nu mai exista padding-top pe .has-tag).
   Orice inaltime suplimentara aici ar dezalinia stratul evidentiat fata de textarea reala de
   dedesubt (care nu poate primi padding pe linii individuale) - exact bug-ul "scrisul se pune
   in alta parte": cursorul real ramane corect, dar vizualul aluneca progresiv in jos cu fiecare
   eticheta intalnita. Eticheta "pluteste" deasupra liniei, fara sa consume spatiu propriu. */
.grp-tag{display:inline-block;margin-left:10px;z-index:3;font-size:8px;font-weight:800;letter-spacing:.3px;
  text-transform:uppercase;border-radius:4px;padding:1px 5px;font-family:var(--mono);
  line-height:1.3;border:1px solid;white-space:nowrap;pointer-events:none;vertical-align:1px}
.grp-tag.tag-mod{color:var(--amber);border-color:var(--amber);background:var(--amber2)}
.grp-tag.tag-adus{color:var(--neon);border-color:var(--neon);background:var(--neon2)}
.grp-tag.tag-user{color:var(--kw);border-color:var(--kw);background:var(--kw3)}
/* stanga (blk) e un div liber, fara nicio sincronizare - poate primi padding, asa ca eticheta
   isi face loc singura sus, fara sa acopere prima linie de cod */
.blk.del,.blk.mods{padding-top:24px}
.blk-tag{position:absolute;left:6px;top:4px;z-index:3;font-size:8px;font-weight:800;letter-spacing:.3px;
  text-transform:uppercase;border-radius:4px;padding:1px 5px;font-family:var(--mono);
  line-height:1.3;border:1px solid;white-space:nowrap}
.blk-tag.tag-sters,.blk-tag.tag-similar{color:var(--coral);border-color:var(--coral);background:var(--coral2)}
.blk-tag.tag-mod{color:var(--amber);border-color:var(--amber);background:var(--amber2)}


#sesInfo {
  display: flex;
  flex-direction: column; /* Forțează span-urile să stea unul sub altul */
  gap: 4px;              /* Spațiul în pixeli dintre fiecare linie */
  flex: 1;
  min-width: 0;
}

#sesInfo span {
  font-size: 11px;
  /* Folosește un gri foarte deschis (aproape alb) pentru detalii */
  color: #e0e0e0; 
  white-space: normal;
  word-break: break-all;
  line-height: 1.3;
}

#sesInfo span strong {
  /* Folosește alb pur sau un cyan/teal deschis pentru etichete ca să iasă în evidență */
  color: #ffffff; 
  font-weight: 600;
}
/* highlight DOAR pe textul diferit (nu pe toata linia) - stil Araxis */
.wdiff{border-radius:2px;padding:0 1px}
.wdiff.w-mod{background:var(--amber2);box-shadow:inset 0 -2px 0 var(--amber)}
.wdiff.w-adus{background:var(--neon2);box-shadow:inset 0 -2px 0 var(--neon)}
.wdiff.w-user{background:var(--kw3);box-shadow:inset 0 -2px 0 var(--kw)}
/* stil diffchecker.com: cuvantul care lipseste in tinta (sters/inlocuit) -
   rosu + taiere de text, in panoul SURSA; cuvantul nou (tinta) - verde, fara
   taiere. Nuanta neta diferita de fundalul galben al liniei modificate, ca sa
   nu se piarda pe el. */
.wdiff.w-del{background:var(--coral2);box-shadow:inset 0 -2px 0 var(--coral);
  background-image:repeating-linear-gradient(135deg, color-mix(in srgb, var(--coral) 22%, transparent) 0 4px, transparent 4px 8px)}
.wdiff.w-add{background:var(--neon2);box-shadow:inset 0 -2px 0 var(--neon)}
body.guided-mode .wdiff.w-del,#pvSrc .wdiff.w-del,#pvDst .wdiff.w-del{box-shadow:0 0 0 1px var(--coral),inset 0 -2px 0 var(--coral)}
body.guided-mode .wdiff.w-add,#pvSrc .wdiff.w-add,#pvDst .wdiff.w-add{box-shadow:0 0 0 1px var(--neon),inset 0 -2px 0 var(--neon)}
body.guided-mode .wdiff,#pvSrc .wdiff,#pvDst .wdiff{font-weight:700}
body.guided-mode .wdiff.w-mod,#pvSrc .wdiff.w-mod,#pvDst .wdiff.w-mod{box-shadow:0 0 0 1px var(--amber),inset 0 -2px 0 var(--amber)}
body.guided-mode .wdiff.w-adus,#pvSrc .wdiff.w-adus,#pvDst .wdiff.w-adus{box-shadow:0 0 0 1px var(--neon),inset 0 -2px 0 var(--neon)}
body.guided-mode .wdiff.w-user,#pvSrc .wdiff.w-user,#pvDst .wdiff.w-user{box-shadow:0 0 0 1px var(--kw),inset 0 -2px 0 var(--kw)}
/* dot-marker in scrollbar-ul lateral (mini-harta diferentelor) */
.diffmap{position:absolute;top:0;right:2px;width:8px;bottom:0;z-index:4;pointer-events:none}
.diffmap i{position:absolute;right:0;width:6px;height:3px;border-radius:2px}
.diffmap i.mod{background:var(--amber)}
.diffmap i.adus{background:var(--neon)}
.diffmap i.user{background:var(--kw)}
/* numere de linie in panoul SURSA (blocuri) */
.blk .bl-ln{display:inline-block;width:30px;margin-left:-6px;margin-right:8px;text-align:right;color:var(--muted);opacity:.55;font-size:10px;user-select:none}
.blk .bl-row{display:block}
.bl-row.ln-uniq{background:rgba(198,40,40,.10)}
.bl-row.ln-sim{background:rgba(30,89,166,.08)}
/* NOU */
.sel-dest-wrap{flex-shrink:0;border:2px solid var(--kw);border-radius:4px;overflow:hidden}
.sel-dest-label{padding:6px 14px;font-size:10px;font-weight:900;letter-spacing:1px;text-transform:uppercase;background:var(--kw);color:#fff;text-align:center}
/* NOU */
.sel-dest-bloc{padding:8px 14px;font-family:var(--mono);font-size:11px;line-height:1.7;white-space:pre-wrap;background:var(--card)}
/* buton de mutare bloc stanga->dreapta */
.blk-push{position:absolute;top:6px;right:8px;width:24px;height:24px;border-radius:6px;
  display:none;align-items:center;justify-content:center;cursor:pointer;z-index:5;
  background:var(--teal3);border:1px solid var(--teal);color:var(--teal);font-size:14px;transition:all .15s}
.blk:hover .blk-push{display:flex}
.blk-push:hover{background:var(--teal);color:#000}
.blk.mutat{opacity:.5}
.blk.mutat .blk-push{background:var(--neon2);border-color:var(--neon);color:var(--neon)}
/* ── MOD PLASARE bloc (stanga -> dreapta) ── */
.plc-bar{display:none;align-items:center;gap:10px;padding:7px 20px;background:var(--amber2);border-bottom:1px solid var(--amber);font-size:11.5px;color:var(--amber);flex-shrink:0;position:relative;z-index:6}
.plc-bar.on{display:flex}
.plc-bar b{color:var(--txt);font-family:var(--mono)}
.plc-bar .plc-sp{margin-left:auto;display:flex;gap:8px}
.plc-bar button{border:1px solid;border-radius:6px;padding:5px 12px;font-size:11px;font-weight:700;cursor:pointer;font-family:inherit}
.plc-ok{background:var(--neon2);border-color:var(--neon);color:var(--neon)}
.plc-ok:hover{background:var(--neon);color:#fff}
.plc-cancel{background:var(--coral2);border-color:var(--coral);color:var(--coral)}
.plc-cancel:hover{background:var(--coral);color:#fff}
/* editorul din dreapta cand plaseaza: mascam textarea, aratam blocuri-tinta clickabile */
.plc-targets{display:none;flex-direction:row;height:100%}
.plc-targets.on{display:flex}
.edit-wrap.plc-hide{display:none}
/* panou stanga in modul plasare: blocul sursa + preview rezultat */
.plc-src-panel{width:45%;border-right:2px solid var(--amber);display:flex;flex-direction:column;flex-shrink:0;min-height:0;overflow:hidden}
.plc-src-hd{padding:6px 12px;font-size:9.5px;font-weight:800;text-transform:uppercase;letter-spacing:.8px;flex-shrink:0;display:flex;align-items:center;gap:6px}
.plc-src-hd.src-lbl{background:var(--coral2);color:var(--coral);border-bottom:1px solid var(--coral)}
.plc-src-hd.res-lbl{background:var(--neon2);color:var(--neon);border-bottom:1px solid var(--neon)}
.plc-src-body{padding:8px 12px;font-family:var(--mono);font-size:11px;line-height:1.7;white-space:pre-wrap;overflow-y:auto;flex:1 1 0;min-height:0;background:var(--coral2)}
.plc-res-body{padding:8px 12px;font-family:var(--mono);font-size:11px;line-height:1.7;white-space:pre-wrap;overflow-y:auto;flex:1 1 0;min-height:0;background:var(--neon2)}
.plc-res-empty{padding:20px 12px;font-size:11px;color:var(--muted);font-style:italic;text-align:center;flex:1;display:flex;align-items:center;justify-content:center}
/* panou dreapta: lista destinatii */
.plc-dest-panel{flex:1;display:flex;flex-direction:column;min-height:0;overflow:hidden}
.plc-dest-hd{padding:6px 12px;font-size:9.5px;font-weight:800;text-transform:uppercase;letter-spacing:.8px;background:var(--amber2);color:var(--amber);border-bottom:1px solid var(--amber);flex-shrink:0}
.plc-dest-list{flex:1 1 0;min-height:0;overflow-y:auto;padding:4px 0}
.pt-gap{min-height:16px;margin:2px 8px;border-radius:4px;cursor:pointer;position:relative;transition:all .12s;border:1px dashed var(--border);display:flex;align-items:center}
.pt-gap:hover,.pt-gap.pick,.pt-gap.dz-hot{min-height:26px;background:var(--neon2);border:1px solid var(--neon)}
.pt-gap-lbl{font-size:10px;color:var(--muted);font-weight:700;padding-left:10px;transition:all .12s}
.pt-gap:hover .pt-gap-lbl,.pt-gap.pick .pt-gap-lbl,.pt-gap.dz-hot .pt-gap-lbl{color:var(--neon)}
.pt-blk{padding:6px 14px;font-family:var(--mono);font-size:11px;line-height:1.6;white-space:pre-wrap;cursor:pointer;border-left:3px solid transparent;position:relative;transition:all .12s}
.pt-blk:hover,.pt-blk.dz-hot{background:var(--teal3);border-left-color:var(--teal)}
.pt-blk.pick{background:var(--amber2);border-left-color:var(--amber);box-shadow:inset 0 0 0 1px var(--amber)}
.pt-blk .pt-lbl{position:absolute;top:6px;right:8px;width:24px;height:24px;border-radius:6px;display:none;align-items:center;justify-content:center;border:1px solid var(--border2);background:#fff;color:var(--amber);cursor:pointer;transition:background .12s,color .12s}
.pt-blk:hover .pt-lbl,.pt-blk.pick .pt-lbl{display:flex}
.pt-blk.pick .pt-lbl{background:var(--amber);border-color:var(--amber);color:#fff}
.pt-lbl svg{width:14px;height:14px;stroke:currentColor;fill:none;stroke-width:2.2}
.pt-ln{display:block;position:relative;min-height:1em;padding-left:12px}
.pt-ln.has-mk{padding-left:12px}
.pt-mk{position:absolute;left:0;top:50%;transform:translateY(-50%);height:60%;width:3px;border-radius:2px}
.pt-mk.m-mod{background:var(--amber)}
.pt-mk.m-add{background:var(--neon)}
.pt-sugg{box-shadow:inset 0 0 0 2px var(--amber)}
/* preview destinatie (strat peste editor, la click pe bloc stanga) */
.plc-preview{display:none;flex-direction:column;position:absolute;inset:0;background:var(--navy2);z-index:10;overflow-y:auto;border:2px solid var(--amber);border-radius:4px}
.plc-preview.on{display:flex}
.prev-hint{padding:7px 14px;font-size:10.5px;color:var(--amber);font-weight:700;background:var(--amber2);border-bottom:1px solid var(--amber);flex-shrink:0;letter-spacing:.3px}
.prev-hint b{color:var(--txt);font-family:var(--mono)}
/* contextul din jur: estompat */
.prev-ctx-bloc{padding:6px 14px;font-family:var(--mono);font-size:11px;line-height:1.6;white-space:pre-wrap;opacity:.35;border-bottom:1px solid var(--border2)}
.prev-ctx-top{padding:5px 14px;font-size:10px;color:var(--muted);font-style:italic;opacity:.5}
/* destinatia sugerata: evidentiata puternic */
.prev-dest-wrap{flex-shrink:0}
.prev-dest-label{padding:6px 14px;font-size:10px;font-weight:900;letter-spacing:1px;text-transform:uppercase;background:var(--amber);color:#000;text-align:center}
.prev-dest-bloc{padding:8px 14px;font-family:var(--mono);font-size:11px;line-height:1.7;white-space:pre-wrap;background:var(--amber2);border-left:4px solid var(--amber)}
.prev-dest-replace{background:rgba(178,106,0,.08);border-left-color:var(--amber)}
/* bloc selectat in stanga (click pe el = selectie + preview dreapta) */
.blk.sel-src{outline:2px solid var(--teal)!important;outline-offset:-2px;background:var(--teal3)!important;box-shadow:inset 3px 0 0 var(--teal)!important}
.blk.trag{cursor:grab}.blk.trag:active{cursor:grabbing}
body.dragging .pt-gap{background:var(--neon2);border-style:solid;min-height:22px}
body.dragging .pt-gap-lbl{color:var(--neon)}
.blk{padding:9px 14px;border-bottom:1px solid var(--border2);font-family:var(--mono);font-size:11px;line-height:1.7;white-space:pre-wrap;position:relative;cursor:pointer}
.blk:hover{background:rgba(255,255,255,.05)}
.blk.sel{outline:2px solid var(--teal);outline-offset:-2px}
.blk.activ{outline:2px solid var(--teal);outline-offset:-2px;box-shadow:0 0 0 4px var(--teal3)}
.blk.eq{color:var(--muted)}
.blk.gol{min-height:20px;background:transparent}
.blk.del,.blk.add,.blk.mods,.blk.modc{padding-left:17px}
.blk.del::before,.blk.add::before,.blk.mods::before,.blk.modc::before{content:'';position:absolute;left:5px;top:50%;transform:translateY(-50%);height:40%;min-height:12px;width:3px;border-radius:2px}
.blk.del{background:var(--coral2)}
.blk.del::before{background:var(--coral)}
/* hasurare diagonala DOAR pe blocurile STERSE (lipsesc din tinta) - semnal in
   plus fata de simpla culoare de fundal, ca sa iasa clar in evidenta ce anume
   "lipseste" fata de ce e doar "modificat"; se adauga peste var(--coral2) de
   mai sus (nu inlocuieste - background-image e proprietate separata) */
.blk.del{background-image:repeating-linear-gradient(135deg, color-mix(in srgb, var(--coral) 16%, transparent) 0 6px, transparent 6px 12px)}
.blk.add{background:var(--neon2)}
.blk.add::before{background:var(--neon)}
.blk.mods{background:var(--amber2)}
.blk.mods::before{background:var(--amber)}
.blk.modc{background:var(--teal2)}
.blk.modc::before{background:var(--teal)}
.blk.del{border-bottom-color:var(--coral);border-bottom-width:2px}
.blk.add{border-bottom-color:var(--neon);border-bottom-width:2px}
.blk.mods{border-bottom-color:var(--amber);border-bottom-width:2px}
.blk.modc{border-bottom-color:var(--teal);border-bottom-width:2px}
.blk.miss{background:var(--border2);border-left:3px solid var(--border2);color:var(--muted);font-style:italic;background-image:repeating-linear-gradient(135deg, color-mix(in srgb, var(--muted) 14%, transparent) 0 6px, transparent 6px 12px)}
/* bloc sters fara candidat gasit in tinta - fara pozitie stabilita, deci il
   distantam vizual de vecini (nu e "lipit" de un loc anume, ca sa nu para ca
   apartine exact acolo unde a picat intamplator in text) */
.blk.no-match{margin:10px 0;border-bottom-style:dashed;border-top:1px dashed var(--coral)}
.blk.user-edit{background:var(--kw3)!important;border-left:3px solid var(--kw)!important;color:var(--kw)!important}
.blk.excl{background:rgba(0,0,0,.15);border-left:3px solid rgba(255,255,255,.06);opacity:.45}
.ibtn{position:absolute;top:7px;right:10px;width:22px;height:22px;border-radius:5px;display:flex;align-items:center;justify-content:center;font-size:13px;cursor:pointer;border:1px solid;z-index:10;transition:all .15s}
.ibtn.push{background:var(--neon2);border-color:var(--neon);color:var(--neon)}
.ibtn.push:hover{background:var(--neon);color:#000}
.ibtn.modif{background:var(--amber2);border-color:var(--amber);color:var(--amber)}
.ibtn.modif:hover{background:var(--amber);color:#000}
.ibtn.undo{background:var(--coral2);border-color:var(--coral);color:var(--coral);display:none}
.ibtn.undo:hover{background:var(--coral);color:#fff}
.ibtn.excl-btn{background:rgba(255,255,255,.05);border-color:rgba(255,255,255,.15);color:var(--muted)}
.ibtn.excl-btn:hover{background:rgba(255,0,85,.2);color:var(--coral)}
.ibtn.rest-btn{background:var(--teal3);border-color:var(--teal);color:var(--teal);display:none}
.kw{color:var(--kw);font-weight:700}
.v{color:var(--sqlvar);font-weight:600}
.hdel{background:rgba(255,0,85,.4);border-radius:2px;padding:0 2px}
.hins{background:rgba(57,255,20,.4);border-radius:2px;padding:0 2px}
.ch-del{background:rgba(255,0,85,.35);border-radius:2px;padding:0 2px;box-shadow:inset 0 0 0 1px var(--coral)}
.ch-ins{background:rgba(57,255,20,.35);border-radius:2px;padding:0 2px;box-shadow:inset 0 0 0 1px var(--neon)}

/* MODAL EDITOR */
.modal-overlay{position:fixed;inset:0;background:rgba(0,0,0,.9);z-index:100;display:none;align-items:center;justify-content:center}
.modal-overlay.open{display:flex}
.modal{background:var(--card);border:1px solid var(--border);border-radius:14px;width:90vw;max-width:1100px;height:88vh;display:flex;flex-direction:column;box-shadow:0 30px 80px rgba(0,0,0,.9)}
.modal-head{padding:12px 20px;background:var(--navy2);border-bottom:1px solid var(--border2);border-radius:14px 14px 0 0;display:flex;align-items:center;gap:12px;flex-shrink:0}
.modal-head h2{margin:0;font-size:14px;color:var(--teal);font-family:var(--mono)}
.modal-head .sub{font-size:11px;color:var(--muted)}
.modal-close{margin-left:auto;background:rgba(255,0,85,.15);color:var(--coral);border:1px solid rgba(255,0,85,.3);border-radius:6px;padding:6px 14px;cursor:pointer;font-size:13px;transition:all .2s}
.modal-close:hover{background:var(--coral);color:#fff}
.modal-tbar{padding:8px 20px;background:rgba(0,0,0,.2);border-bottom:1px solid var(--border2);flex-shrink:0;display:flex;gap:8px;align-items:center}
.modal-tbar button{border:none;padding:6px 12px;border-radius:6px;cursor:pointer;font-size:12px;font-family:inherit;font-weight:600}
.bsave{background:var(--neon2);color:var(--neon);border:1px solid var(--neon)!important}
.bcopy2{background:var(--teal3);color:var(--teal);border:1px solid var(--teal)!important}
#editStatus{font-size:11px;color:var(--amber);font-weight:700}
.legend{display:flex;gap:10px;font-size:11px;color:var(--muted);margin-left:auto;align-items:center}
.leg{display:flex;align-items:center;gap:4px}
.leg-dot{width:9px;height:9px;border-radius:2px}
.modal-editor{flex:1;overflow:hidden}
.CodeMirror{height:100%!important;font-family:var(--mono)!important;font-size:13px!important;line-height:1.6!important;background:var(--navy)!important}
.CodeMirror-scroll{height:100%}
.cm-hl-adus{background:rgba(57,255,20,.12)!important}
.cm-hl-mod{background:var(--blue2)!important}
.cm-marker-line{height:0!important;overflow:hidden!important;font-size:0!important;line-height:0!important;padding:0!important;margin:0!important;border:none!important}

/* ═══ RESPONSIVE - adaptare la orice rezolutie ═══ */
.prev-ctx-bloc .pt-ln,.prev-dest-bloc .pt-ln,.plc-res-body .pt-ln,.plc-src-body .pt-ln{display:block;min-height:1em}
@media (max-height:760px){
  .gm-formcol{padding:16px 18px}
  .brow{margin-bottom:8px}
  .aseg{margin-bottom:8px}
  .fld{margin-bottom:8px}
  .fld input{padding:7px 10px}
  .btn-test{padding:8px;margin-bottom:8px}
  .btn-run{padding:11px}
  .dbl{margin-bottom:10px;max-height:clamp(100px,22vh,240px)}
  .stbar{padding:6px 10px;margin-bottom:8px}
}
@media (max-height:600px){
  .bsub{display:none}
  .dbl{max-height:clamp(90px,20vh,180px)}
}
@media (max-width:900px){
  .gm-main{flex-direction:column}
  .gm-formcol{width:100%;max-height:52vh;border-right:none;border-bottom:1px solid rgba(255,255,255,.07)}
  .gm-termcol{min-height:160px}
}
@media (max-width:560px){
  .frow{grid-template-columns:1fr}
  .brow{flex-wrap:wrap}
}
@media (max-width:1080px){
  .lhd,.lrow{padding:0 12px}
  .finw{width:150px}
}
@media (max-width:1100px){
  .modal{width:96vw;height:92vh}
}

</style>
</head><body>
<div id="oldBrowserWarn" style="display:none;position:fixed;top:0;left:0;right:0;z-index:99999;background:#b91c1c;color:#ffffff;font-family:-apple-system,'Segoe UI',Arial,sans-serif;font-size:14px;font-weight:600;padding:12px 18px;text-align:center;box-shadow:0 2px 8px rgba(0,0,0,.35)">
  Browserul acesta este prea vechi si nu poate afisa corect aplicatia (lipseste suport pentru CSS variables).
  Deschide aplicatia intr-un Chrome, Edge sau Firefox actualizat.
</div>
<script>
(function(){
  // feature-detect CSS custom properties - daca lipsesc, toata tematizarea
  // aplicatiei (var(--...)) e ignorata de browser si pagina ramane goala,
  // fara nicio eroare in consola care sa explice de ce
  var suporta = !!(window.CSS && CSS.supports && CSS.supports('(--a: 0)'));
  if (!suporta){
    var b = document.getElementById('oldBrowserWarn');
    if (b) b.style.display = 'block';
  }
})();
</script>

<!-- ═══ ECRAN 1: CONECTARE ═══ -->
<div class="screen active" id="ec">
  <div class="gm-window">
    <div class="gm-body">
      <div class="gm-main">
        <div class="gm-formcol">
        <div class="lcard">
          <div class="lch">
            <div class="brow">
              <div class="brand-gm">
                <span class="gm-bracket">{</span>
                <div class="gm-dots">
                  <span class="gm-dot"></span><span class="gm-dot a"></span>
                  <span class="gm-dot a"></span><span class="gm-dot"></span>
                </div>
                <span style="color:var(--txt)">GMotion</span>
                <span class="gm-bracket">}</span>
              </div>
              <div>
                <div class="bnm">SQL Diff &amp; Deploy</div>
                <div class="bsub">Comparare structuri SQL Server</div>
              </div>
              <span class="alive" style="margin-left:auto"><span class="alive-d"></span>v3.4</span>
              <div class="tema-wrap" style="margin-left:10px">
                <button class="tema-btn" onclick="togTema(event)">
                  <i class="ti ti-palette" aria-hidden="true"></i>Tema
                  <i class="ti ti-chevron-down arr" aria-hidden="true"></i>
                </button>
                <div class="tema-dropdown">
                  <div class="tema-item active" data-tema="github" onclick="switchTheme('github')"><span class="tdot" style="background:#2f81f7;border:1px solid #d0d7de"></span>GitHub Light</div>
                  <div class="tema-item" data-tema="dracula" onclick="switchTheme('dracula')"><span class="tdot" style="background:#bd93f9"></span>Dracula</div>
                  <div class="tema-item" data-tema="nord" onclick="switchTheme('nord')"><span class="tdot" style="background:#88c0d0"></span>Nord</div>
                  <div class="tema-item" data-tema="onedark" onclick="switchTheme('onedark')"><span class="tdot" style="background:#61afef"></span>One Dark Pro</div>
                  <div class="tema-item" data-tema="gruvbox" onclick="switchTheme('gruvbox')"><span class="tdot" style="background:#fe8019"></span>Gruvbox Dark</div>
                  <div class="tema-item" data-tema="catppuccin" onclick="switchTheme('catppuccin')"><span class="tdot" style="background:#cba6f7"></span>Catppuccin Mocha</div>
                </div>
              </div>
            </div>
            <div class="aseg">
              <button class="abtn on" id="a-sql" onclick="setAuth('sql')"><i class="ti ti-lock" aria-hidden="true"></i>SQL Auth</button>
              <button class="abtn" id="a-win" onclick="setAuth('win')"><i class="ti ti-brand-windows" aria-hidden="true"></i>Windows</button>
            </div>
            <div class="lch-tabs">
              <button class="lchtab on" id="tab-conectare" onclick="switchLcbTab('conectare')"><i class="ti ti-plug" aria-hidden="true"></i>Conectare</button>
              <button class="lchtab" id="tab-sesiuni" onclick="switchLcbTab('sesiuni')"><i class="ti ti-device-floppy" aria-hidden="true"></i>Sesiuni<span class="lchtab-badge" id="sesBadge" style="display:none">0</span></button>
            </div>
          </div>
          <div class="lcb">
            <div id="panelSesiuni" style="display:none">
              <div id="sesLista"></div>
            </div>
            <div id="panelConectare">
            <div class="sesbanner sesbanner-activ" id="activeSesBanner" style="display:none">
              <div class="sesbanner-top">
                <i class="ti ti-device-floppy" aria-hidden="true"></i>
                <div class="sesbanner-txt">
                  <b>Sesiune gasita</b>
                  <span id="activeSesInfo" style="display:contents"></span>
                </div>
              </div>
              <div class="sesbanner-btns">
                <button class="ses-btn" onclick="anuleazaSesiuneActiva()"><i class="ti ti-x" aria-hidden="true"></i>Anuleaza</button>
              </div>
            </div>
            <div class="ses-hint" id="sesHint" style="display:none">
              <i class="ti ti-info-circle" aria-hidden="true"></i>
              <span id="sesHintTxt"></span>
            </div>
            <div class="fld">
              <label><i class="ti ti-server" aria-hidden="true"></i>Server SQL</label>
              <div class="srv-wrap" id="srvWrap">
                <div style="display:flex;gap:6px">
                  <input id="isrv" type="text" placeholder="ex: SERVER\INSTANTA" autocomplete="off" style="flex:1" onfocus="if(document.getElementById('srvDrop').childElementCount)toggleSrvDrop(true)">
                  <button class="btn-cauta" id="btnCauta" type="button" title="Cauta servere in retea" onclick="cautaServere()">
                    <i class="ti ti-database" aria-hidden="true"></i>
                  </button>
                </div>
                <div class="srv-dropdown" id="srvDrop"></div>
              </div>
            </div>
            <div class="frow" id="cred-row">
              <div class="fld">
                <label><i class="ti ti-user" aria-hidden="true"></i>Utilizator</label>
                <input id="iusr" type="text" placeholder="sa">
              </div>
              <div class="fld">
                <label><i class="ti ti-key" aria-hidden="true"></i>Parola</label>
                <input id="ipwd" type="password">
              </div>
            </div>
            <button class="btn-test" id="btnTest" onclick="testConexiune()">
              <i class="ti ti-plug" aria-hidden="true"></i>Testeaza conexiunea
            </button>
            <div class="stbar" id="stTest"></div>
            <div class="srv-details" id="srvDetails" style="display:none"></div>
            <div class="dbt" id="dbt" style="display:none">
              <div></div><div>Baza de date</div><div style="text-align:right">Referinta</div>
            </div>
            <div class="dbl" id="dbl"></div>
            <button class="btn-run" id="btnRun" disabled onclick="ruleaza()">
              <i class="ti ti-player-play-filled" aria-hidden="true"></i>Porneste verificarea
            </button>
            <button class="btn-test" id="btnOfflineTrigger" type="button" onclick="deschideOfflineModal()" style="margin-top:10px;margin-bottom:0">
              <i class="ti ti-git-diff" aria-hidden="true"></i>Comparare offline (copy/paste)
            </button>
            <div class="pgbar" id="pgbar"><div class="pgfill" id="pgfill"></div></div>
            <div class="pginfo" id="pginfo"><span id="pgmsg"></span><span id="pgtime"></span></div>
            <div class="lock-msg" id="lockMsg" style="display:none">
              <i class="ti ti-lock" style="font-size:12px"></i> Conexiune activa —
              <a href="#" onclick="event.preventDefault();unlockConexiune();verificaSesiuneSalvata();switchLcbTab('sesiuni');">Deconecteaza</a>
            </div>
            </div><!-- /panelConectare -->
          </div>
        </div>
        </div><!-- /gm-formcol -->
        <div class="gm-termcol">
          <div class="term-wrap" id="termWrap">
            <div class="term-hd">
              <i class="ti ti-terminal-2" style="font-size:13px" aria-hidden="true"></i>
              <span>Output backend</span>
              <button class="term-clr" onclick="termClear()" title="Sterge log-ul">
                <i class="ti ti-trash" aria-hidden="true"></i>
              </button>
            </div>
            <pre class="term-body" id="termBody"><div class="term-empty term-legend" id="termLegend">
              <div class="tl-corner tl-c1"></div><div class="tl-corner tl-c2"></div><div class="tl-corner tl-c3"></div><div class="tl-corner tl-c4"></div>
              <div class="tl-sec">
                <div class="tl-rail"><span class="tl-dot"></span><span class="tl-line"></span></div>
                <div class="tl-body">
                  <div class="tl-tag">§ 01</div>
                  <div class="tl-title">Ce face aplicatia</div>
                  <div class="tl-desc">Compara structurile SQL (tabele, view-uri, proceduri, functii, triggere) dintre baza de <b>REFERINTA</b> si una sau mai multe baze <b>TINTA</b>, evidentiaza diferentele si genereaza un script de deploy pentru obiectele rezolvate.</div>
                </div>
              </div>
              <div class="tl-sec">
                <div class="tl-rail"><span class="tl-dot"></span><span class="tl-line"></span></div>
                <div class="tl-body">
                  <div class="tl-tag">§ 02</div>
                  <div class="tl-title">Cum completezi ecranul de conectare</div>
                  <div class="tl-desc">
                    <b>SQL Auth / Windows</b> — alege modul de autentificare: utilizator+parola sau contul Windows curent.<br>
                    <b>SERVER SQL</b> — scrie numele serverului (ex: <b>SERVER\INSTANTA</b>) sau apasa <i class="ti ti-search" aria-hidden="true"></i> ca sa cauti automat serverele SQL din reteaua locala.<br>
                    <b>Utilizator / Parola</b> — necesare doar la SQL Auth.<br>
                    <b>Testeaza conexiunea</b> — verifica acreditarile si aduce lista bazelor de date disponibile pe server.<br>
                    <b>Alege o baza de REFERINTA</b> din lista aparuta — restul bazelor vor fi comparate fata de aceasta.<br>
                    <b>Porneste verificarea</b> — devine activ dupa ce ai ales referinta; incepe scanarea si compararea.
                  </div>
                </div>
              </div>
              <div class="tl-sec">
                <div class="tl-rail"><span class="tl-dot"></span><span class="tl-line"></span></div>
                <div class="tl-body">
                  <div class="tl-tag">§ 03</div>
                  <div class="tl-title">Cum functioneaza sesiunile</div>
                  <div class="tl-desc">
                    Progresul (ce e rezolvat, ce a fost trimis la deploy) se salveaza automat, <b>separat pentru fiecare baza de REFERINTA</b> — nu se suprascriu intre ele.<br>
                    Tab-ul <b>Sesiuni</b> arata toate sesiunile salvate; <b>Reia</b> o marcheaza ca activa si completeaza automat serverul si autentificarea; <b>Ignora</b> o sterge definitiv.<br>
                    Sesiunea activa apare mov, in tab-ul <b>Conectare</b>, ca sa stii clar in ce te conectezi — se poate anula oricand din acelasi banner.<br>
                    La <b>Deconectare</b>, progresul curent se salveaza automat si te intorci la lista de sesiuni.
                  </div>
                </div>
              </div>
              <div class="tl-sec">
                <div class="tl-rail"><span class="tl-dot"></span><span class="tl-line"></span></div>
                <div class="tl-body">
                  <div class="tl-tag">§ 04</div>
                  <div class="tl-title">Simbolurile din ecranul Lista</div>
                  <div class="tl-tokens">
                    <span class="tl-tok"><span class="d" style="background:var(--neon);box-shadow:0 0 6px var(--neon)"></span>Rezolvat</span>
                    <span class="tl-tok"><span class="d" style="background:transparent;border:2px solid var(--muted)"></span>Nerezolvat</span>
                    <span class="tl-tok"><span class="d" style="background:var(--muted);box-shadow:0 0 6px rgba(89,99,110,.4)"></span>Omis (analizat, exclus din deploy)</span>
                    <span class="tl-tok"><span class="d" style="background:var(--teal)"></span>Trimis la deploy</span>
                    <span class="tl-tok"><span class="d" style="background:var(--blue)"></span>Pregatit pentru deploy</span>
                    <span class="tl-tok"><span class="d" style="background:var(--amber)"></span>Versiune noua / modificat de atunci</span>
                    <span class="tl-tok"><span class="d" style="background:var(--muted)"></span>Neintrodus in deploy</span>
                  </div>
                </div>
              </div>
              <div class="tl-sec">
                <div class="tl-rail"><span class="tl-dot"></span><span class="tl-line"></span></div>
                <div class="tl-body">
                  <div class="tl-tag">§ 05</div>
                  <div class="tl-title">Simbolurile din Editorul de diferente</div>
                  <div class="tl-tokens">
                    <span class="tl-tok"><span class="d" style="background:var(--amber)"></span>Modificat</span>
                    <span class="tl-tok"><span class="d" style="background:var(--neon)"></span>Adus din sursa</span>
                    <span class="tl-tok"><span class="d" style="background:var(--kw)"></span>Scris de tine</span>
                  </div>
                </div>
              </div>
              <div class="tl-sec">
                <div class="tl-rail"><span class="tl-dot" style="background:var(--muted);box-shadow:none"></span></div>
                <div class="tl-body">
                  <div class="tl-hint">Apasa <b>Porneste verificarea</b><span class="tl-cursor"></span> — jurnalul de mai sus va fi inlocuit de activitatea reala.</div>
                </div>
              </div>
            </div></pre>
          </div>
        </div><!-- /gm-termcol -->
      </div><!-- /gm-main -->
    </div><!-- /gm-body -->
    <div class="gm-transition" id="gmTransition">
      <div class="gm-trans-spin"></div>
      <div class="gm-trans-msg" id="gmTransMsg">Se pregateste raportul...</div>
    </div>
  </div><!-- /gm-window -->
</div>

<!-- ═══ ECRAN 2: LISTA ═══ -->
<div class="screen" id="el">
  <div class="grid-bg"></div>
  <div class="ltop">
    <div class="ltop-row1">
    <div class="brand-gm" style="font-size:13px;padding:4px 10px">
      <span class="gm-bracket">[</span>
      <div class="gm-dots"><span class="gm-dot"></span><span class="gm-dot a"></span><span class="gm-dot a"></span><span class="gm-dot"></span></div>
      <span style="color:var(--txt)">GMOTION</span>
      <span class="gm-bracket">]</span>
    </div>
    <div class="ltop-title">
      <div class="ltitle">Diferente detectate</div>
      <div class="lsub" id="lsub">Se incarca...</div>
      <div class="ses-save-info" id="sesSaveInfo" style="display:none">
        <i class="ti ti-device-floppy" id="sesSaveIco" aria-hidden="true"></i><span id="sesSaveTxt"></span>
      </div>
    </div>
    <div class="ltop-stats">
      <div class="lstats" id="lstats"></div>
      <div class="lstats-prog">
        <div class="prog-chip" title="Procent obiecte marcate ca rezolvate">
          <i class="ti ti-circle-check" aria-hidden="true"></i>
          <div class="prog-mini"><div class="prog-mini-fill" id="progRezFill" style="width:0%"></div></div>
          <span class="prog-chip-val" id="progRezVal">0%</span> rezolvate
        </div>
        <div class="prog-chip" title="Procent obiecte trimise la deploy">
          <i class="ti ti-package-export" aria-hidden="true"></i>
          <div class="prog-mini"><div class="prog-mini-fill dep" id="progDepFill" style="width:0%"></div></div>
          <span class="prog-chip-val" id="progDepVal">0%</span> deploy
        </div>
      </div>
    </div>
    <div class="ltop-actions">
      <button class="tema-btn btn-deconectare" onclick="deconecteaza()" title="Inchide sesiunea si revino la ecranul de conectare">
        <i class="ti ti-plug-off" aria-hidden="true"></i>Deconectare
      </button>
      <div style="display:flex;gap:8px">
      <button class="tema-btn" onclick="deschideHelpLegenda()" title="Legenda si ajutor">
        <i class="ti ti-help-circle" aria-hidden="true"></i>
      </button>
      <div class="tema-wrap">
        <button class="tema-btn" id="temabtn" onclick="togTema(event)">
          <i class="ti ti-palette" aria-hidden="true"></i>Tema
          <i class="ti ti-chevron-down arr" aria-hidden="true"></i>
        </button>
        <div class="tema-dropdown" id="temadrop">
          <div class="tema-item active" data-tema="github" onclick="switchTheme('github')"><span class="tdot" style="background:#2f81f7;border:1px solid #d0d7de"></span>GitHub Light</div>
          <div class="tema-item" data-tema="dracula" onclick="switchTheme('dracula')"><span class="tdot" style="background:#bd93f9"></span>Dracula</div>
          <div class="tema-item" data-tema="nord" onclick="switchTheme('nord')"><span class="tdot" style="background:#88c0d0"></span>Nord</div>
          <div class="tema-item" data-tema="onedark" onclick="switchTheme('onedark')"><span class="tdot" style="background:#61afef"></span>One Dark Pro</div>
          <div class="tema-item" data-tema="gruvbox" onclick="switchTheme('gruvbox')"><span class="tdot" style="background:#fe8019"></span>Gruvbox Dark</div>
          <div class="tema-item" data-tema="catppuccin" onclick="switchTheme('catppuccin')"><span class="tdot" style="background:#cba6f7"></span>Catppuccin Mocha</div>
        </div>
      </div>
      </div>
    </div>
    </div><!-- /ltop-row1 -->
  </div><!-- /ltop -->
  <div class="lflt">
    <div class="finw"><i class="ti ti-search" aria-hidden="true"></i><input type="text" placeholder="Cauta obiect sau diferenta..." id="fNume" oninput="renderLista()"></div>
    <select class="fsel" id="fBaza" onchange="renderLista()"><option value="">(toate bazele)</option></select>
    <select class="fsel" id="fTip" onchange="renderLista()">
      <option value="">(toate tipurile)</option>
      <option value="P">Proceduri</option>
      <option value="V">View-uri</option>
      <option value="FN">Functii</option>
      <option value="TR">Triggere</option>
    </select>
    <select class="fsel" id="fCat" onchange="renderLista()">
      <option value="">(toate categoriile)</option>
      <option value="dif">Diferite</option>
      <option value="lipsa">Lipsesc complet</option>
      <option value="extra">Extra (doar in tinta)</option>
    </select>
    <select class="fsel" id="fStare" onchange="renderLista()">
      <option value="">(toate starile)</option>
      <option value="ne">Nerezolvate</option>
      <option value="da">Rezolvate</option>
      <option value="omis">Omise</option>
      <option value="dep">Deployate</option>
      <option value="ndep">Nedeployate</option>
    </select>
    <div class="frange" title="Interval de similaritate (%)">
      <i class="ti ti-percentage" aria-hidden="true"></i>
      <input type="number" id="fScMin" min="0" max="100" value="0" oninput="renderLista()">
      <span>—</span>
      <input type="number" id="fScMax" min="0" max="100" value="100" oninput="renderLista()">
    </div>
    <button class="bcsv" onclick="exportaCSV()" title="Exporta lista filtrata in CSV">
      <i class="ti ti-file-spreadsheet" aria-hidden="true"></i>CSV
    </button>
    <button class="bdep" id="btnDep" onclick="genereazaDeployer()" disabled>
      <i class="ti ti-package-export" aria-hidden="true"></i><span id="depTxt">Deployer (0)</span>
    </button>
  </div>
  <div class="lhd">
    <div class="lh">#</div>
    <div class="lh" onclick="sortLista('obiect')"><i class="ti ti-function" aria-hidden="true"></i>Obiect <span class="sarr" id="sa-obiect"></span><span class="col-grip" data-cw="--cw-obj" onmousedown="gripStart(event)" ondblclick="gripAutofit(event)" onclick="event.stopPropagation()"></span></div>
    <div class="lh" onclick="sortLista('baza')"><i class="ti ti-database" aria-hidden="true"></i>Baza <span class="sarr" id="sa-baza"></span><span class="col-grip" data-cw="--cw-baza" onmousedown="gripStart(event)" ondblclick="gripAutofit(event)" onclick="event.stopPropagation()"></span></div>
    <div class="lh" onclick="sortLista('tip')">Tip <span class="sarr" id="sa-tip"></span><span class="col-grip" data-cw="--cw-tip" onmousedown="gripStart(event)" ondblclick="gripAutofit(event)" onclick="event.stopPropagation()"></span></div>
    <div class="lh" onclick="sortLista('scor')"><i class="ti ti-percentage" aria-hidden="true"></i>Similaritate <span class="sarr" id="sa-scor"></span><span class="col-grip" data-cw="--cw-scor" onmousedown="gripStart(event)" ondblclick="gripAutofit(event)" onclick="event.stopPropagation()"></span></div>
    <div class="lh"><i class="ti ti-git-diff" aria-hidden="true"></i>Diferente<span class="col-grip" data-cw="--cw-dif" onmousedown="gripStart(event)" ondblclick="gripAutofit(event)" onclick="event.stopPropagation()"></span></div>
    <div class="lh" onclick="sortLista('rezolvat')" style="justify-content:center"><i class="ti ti-circle-check" aria-hidden="true"></i>Stare <span class="sarr" id="sa-rezolvat"></span></div>
    <div class="lh" onclick="sortLista('modificatLa')"><i class="ti ti-device-floppy" aria-hidden="true"></i>Ultima modif. <span class="sarr" id="sa-modificatLa"></span><span class="col-grip" data-cw="--cw-mod" onmousedown="gripStart(event)" ondblclick="gripAutofit(event)" onclick="event.stopPropagation()"></span></div>
    <div class="lh" onclick="sortLista('deployat')"><i class="ti ti-package-export" aria-hidden="true"></i>Deploy <span class="sarr" id="sa-deployat"></span><span class="col-grip" data-cw="--cw-dep" onmousedown="gripStart(event)" ondblclick="gripAutofit(event)" onclick="event.stopPropagation()"></span></div>
    <div class="lh">Actiune</div>
  </div>
  <div class="lbdy" id="tw">
    <div id="sp-sus"></div>
    <div id="rows"></div>
    <div id="sp-jos"></div>
  </div>
  <div class="bara-jos">
    <span class="cnt" id="cnt"></span>
    <span id="statBif" style="font-size:12px;color:var(--neon)"></span>
  </div>
  <div class="dep-overlay" id="depOverlay">
    <div class="dep-modal">
      <div class="dep-head"><i class="ti ti-package-export" aria-hidden="true"></i> Optiuni deployer</div>
      <div class="dep-body">
        <div id="depSumar" style="font-size:12px;background:var(--teal3);border:1px solid var(--border2);border-radius:8px;padding:10px 12px;margin-bottom:12px;line-height:1.7"></div>
        <div class="dep-q">Impartirea fisierelor</div>
        <label class="dep-opt"><input type="radio" name="depmod" value="obiect" checked onchange="actualizeazaDepInfo()"> <span><b>Un fisier per obiect</b></span></label>
        <label class="dep-opt"><input type="radio" name="depmod" value="baza" onchange="actualizeazaDepInfo()"> <span><b>Un fisier per baza</b></span></label>
        <label class="dep-opt"><input type="radio" name="depmod" value="n" onchange="actualizeazaDepInfo()"> <span><b>Grupat</b> — cate <input type="number" id="depN" value="10" min="1" max="500" oninput="actualizeazaDepInfo()" onclick="document.querySelector('input[value=n]').checked=true;actualizeazaDepInfo()"> obiecte pe fisier</span></label>
        <label class="dep-opt"><input type="radio" name="depmod" value="unul" onchange="actualizeazaDepInfo()"> <span><b>Totul intr-un singur fisier</b></span></label>
        <div class="dep-q" style="margin-top:8px">Obiecte incluse</div>
        <label class="dep-opt"><input type="checkbox" id="depInclDeployate" checked onchange="actualizeazaDepInfo()"> <span>Include si ce a fost deja trimis la deploy</span></label>
      </div>
      <div class="dep-foot">
        <button class="dep-cancel" onclick="inchideDeployModal()"><i class="ti ti-x" aria-hidden="true"></i> Anuleaza</button>
        <button class="dep-go" onclick="confirmaDeploy()"><i class="ti ti-package-export" aria-hidden="true"></i> Genereaza</button>
      </div>
    </div>
  </div>
  <div class="dep-overlay" id="confirmOverlay">
    <div class="dep-modal" style="width:min(420px,92vw)">
      <div class="dep-head"><i class="ti ti-plug-off" aria-hidden="true"></i> Deconectare</div>
      <div class="dep-body">
        <div style="font-size:13px;line-height:1.7;color:var(--txt)">Te deconectezi de la server?</div>
        <div style="font-size:11.5px;color:var(--muted);margin-top:6px">Progresul (rezolvate, editari, deploy) se salveaza automat si poate fi reluat oricand din tab-ul Sesiuni.</div>
      </div>
      <div class="dep-foot">
        <button class="dep-cancel" onclick="inchideConfirm()"><i class="ti ti-x" aria-hidden="true"></i> Ramai conectat</button>
        <button class="dep-go" onclick="confirmaDeconectare()"><i class="ti ti-plug-off" aria-hidden="true"></i> Deconecteaza</button>
      </div>
    </div>
  </div>
  <div class="dep-overlay" id="helpOverlay">
    <div class="dep-modal" style="width:min(640px,94vw);max-height:86vh;display:flex;flex-direction:column">
      <div class="dep-head"><i class="ti ti-help-circle" aria-hidden="true"></i> Legenda si ajutor</div>
      <div class="dep-body" id="helpBody" style="overflow-y:auto"></div>
      <div class="dep-foot">
        <button class="dep-cancel" onclick="inchideHelpLegenda()"><i class="ti ti-x" aria-hidden="true"></i> Inchide</button>
      </div>
    </div>
  </div>
</div>

<!-- ═══ ECRAN 3: DETALIU ═══ -->
<style id="featExtra">
.dcol{position:relative}
/* actiuni per bloc (aplica / preview / pozitie) */
.blk-act{position:absolute;top:4px;right:56px;display:none;align-items:center;gap:4px;z-index:3}
.blk:hover .blk-act,.blk.ghid-cur .blk-act{display:inline-flex}
body.guided-mode .blk-act{display:none!important}
/* n/total mereu vizibil, la fel ca in tinta - nu mai depinde de hover */
.blk-poz-perm{position:absolute;top:4px;right:6px;z-index:3;font-size:9px;font-weight:800;color:var(--muted);
  background:var(--card);border:1px solid var(--border2);border-radius:4px;padding:1px 5px;font-family:var(--mono)}
body.guided-mode .il-gut-btns{display:none!important}
.ba-btn{width:22px;height:22px;border-radius:5px;border:1px solid var(--border);background:var(--card);color:var(--txt);cursor:pointer;font-size:11px;line-height:1;display:inline-flex;align-items:center;justify-content:center;padding:0}
.ba-btn:hover{background:var(--teal);color:#fff;border-color:var(--teal)}
.ba-ok:hover{background:var(--neon);border-color:var(--neon)}
.blk.ghid-cur{outline:3px solid var(--teal)!important;outline-offset:-3px;
  box-shadow:inset 4px 0 0 var(--teal),0 0 0 4px var(--teal3),0 4px 14px rgba(0,0,0,.25)!important;
  z-index:4}
.sm-flash{animation:smflash .9s ease}
@keyframes smflash{0%{background:var(--teal2)}100%{background:transparent}}
/* bara de merge inline din modul ghidat (sub blocul activ) */
.gmb{display:flex;align-items:center;gap:10px;padding:8px 14px;background:var(--card);border:1px solid var(--teal);border-top-width:2px;border-bottom-width:2px;font-size:11.5px}
.gmb-pos{color:var(--teal);font-weight:800;white-space:nowrap}
.gmb-info{color:var(--muted);font-weight:600;margin-right:auto;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.gmb button{border:1px solid;border-radius:6px;padding:7px 14px;font-size:12px;font-weight:800;cursor:pointer;font-family:inherit;line-height:1}
.gmb .gmb-ok{background:var(--teal);border-color:var(--teal);color:#fff}
.gmb .gmb-ok:hover{filter:brightness(1.12)}
.gmb .gmb-sec{background:var(--card);border-color:var(--border2);color:var(--txt)}
.gmb .gmb-sec:hover{border-color:var(--teal);color:var(--teal)}
.gmb .gmb-icon{width:26px;height:26px;flex-shrink:0;border-radius:6px;border:1px solid var(--border2);background:var(--card);
  color:var(--txt);cursor:pointer;display:inline-flex;align-items:center;justify-content:center;padding:0}
.gmb .gmb-icon:hover{background:var(--teal);color:#fff;border-color:var(--teal)}
/* modal previzualizare marita + overlay finalizare ghidat */
@keyframes pvFadeIn{from{opacity:0}to{opacity:1}}
@keyframes pvPopIn{from{opacity:0;transform:translateY(14px) scale(.97)}to{opacity:1;transform:translateY(0) scale(1)}}
@keyframes pvDot{0%,100%{opacity:.4;transform:scale(.85)}50%{opacity:1;transform:scale(1.15)}}
.pv-overlay{display:none;position:fixed;inset:0;background:rgba(6,10,18,.8);
  z-index:60;align-items:center;justify-content:center;animation:pvFadeIn .1s ease}
.pv-overlay.on{display:flex}
.pv-box{background:var(--navy);border:1px solid var(--border);border-radius:14px;width:min(1100px,92vw);
  height:min(88vh,860px);display:flex;flex-direction:column;overflow:hidden;
  box-shadow:0 24px 70px rgba(0,0,0,.55),0 0 0 1px rgba(0,240,255,.06);
  animation:pvPopIn .14s cubic-bezier(.2,.9,.3,1.2)}
.pv-box.mica{height:auto;width:min(460px,92vw)}
.off-box{width:min(1000px,94vw)}
.off-body{flex:1 1 auto;display:flex;flex-direction:column;min-height:0;overflow-y:auto;padding:18px 20px}
.off-body .frow{flex:1;min-height:0}
.off-body .frow .fld{display:flex;flex-direction:column;min-height:0}
.off-ta{flex:1;min-height:220px!important;max-height:none!important;resize:none!important}
.pv-hd{display:flex;align-items:center;justify-content:space-between;padding:12px 16px;
  border-bottom:1px solid var(--border2);font-weight:800;font-size:13px;
  background:linear-gradient(90deg,var(--teal3),transparent 60%)}
.pv-hd-title{display:flex;align-items:center;gap:8px}
.pv-hd-ico{font-size:16px;color:var(--teal);flex-shrink:0}
.pv-x{background:transparent;border:1px solid var(--border2);color:var(--txt);border-radius:7px;
  width:26px;height:26px;cursor:pointer;display:inline-flex;align-items:center;justify-content:center;
  transition:transform .1s ease,background .1s,border-color .1s,color .1s}
.pv-x i{font-size:13px}
.pv-x:hover{background:var(--coral2);border-color:var(--coral);color:var(--coral);transform:rotate(90deg)}
.pv-body{flex:1 1 50%;display:grid;grid-template-columns:1fr 1fr;min-height:0}
.pv-col{display:flex;flex-direction:column;min-width:0;border-right:1px solid var(--border2)}
.pv-col:last-child{border-right:none}
.pv-col-hd{padding:7px 14px;font-size:10px;font-weight:900;letter-spacing:1px;border-bottom:1px solid var(--border2);
  display:flex;align-items:center;gap:6px}
.pv-col-hd.src{color:var(--coral);background:linear-gradient(90deg,var(--coral2),transparent 70%)}
.pv-col-hd.wrk{color:var(--neon);background:linear-gradient(90deg,var(--neon2),transparent 70%)}
.pv-scroll{flex:1;overflow:auto;padding:10px 14px;font-family:var(--mono);font-size:11px;line-height:1.7;white-space:pre-wrap}
.pv-final-wrap{flex:1 1 38%;display:flex;flex-direction:column;min-height:0;border-top:1px solid var(--border2);background:var(--card)}
.pv-final-hd{display:flex;align-items:center;justify-content:space-between;padding:7px 14px;font-size:10px;
  font-weight:900;letter-spacing:1px;border-bottom:1px solid var(--border2);color:var(--teal)}
.pv-final-hd>span:first-child{display:flex;align-items:center;gap:7px}
.pv-final-hd>span:first-child::before{content:'';width:7px;height:7px;border-radius:50%;background:var(--teal);
  box-shadow:0 0 6px var(--teal);animation:pvDot 1.8s ease-in-out infinite}
.pv-final-actions{display:flex;align-items:center;gap:6px}
.pv-final-hd small{font-weight:600;letter-spacing:0;text-transform:none;color:var(--muted);margin-left:6px}
.pv-final-ta{position:absolute;inset:0;width:100%;height:100%;resize:none;border:0;outline:none;
  background:transparent;color:transparent;caret-color:var(--teal);font-family:var(--mono);font-size:11px;
  line-height:1.7;padding:10px 14px;white-space:pre;tab-size:4;box-sizing:border-box;z-index:2;overflow:auto}
.pv-final-ta[readonly]{cursor:default;caret-color:transparent}
.pv-final-ta:not([readonly]){box-shadow:inset 0 0 0 1.5px var(--teal),inset 0 0 12px rgba(0,240,255,.08)}
.pv-final-editwrap{position:relative;flex:1;min-height:0}
.pv-final-hl{position:absolute;inset:0;margin:0;overflow:hidden;pointer-events:none;z-index:1;
  font-family:var(--mono);font-size:11px;line-height:1.7;padding:10px 14px;white-space:pre;tab-size:4;
  box-sizing:border-box;color:var(--txt)}
.pv-edit-btn{background:var(--card);border:1px solid var(--border2);color:var(--muted);font-size:10px;font-weight:700;
  padding:4px 10px;border-radius:6px;cursor:pointer;font-family:inherit;display:flex;align-items:center;gap:5px;
  transition:transform .07s ease,border-color .1s,color .1s,background .1s,box-shadow .1s}
.pv-edit-btn i{font-size:12px;flex-shrink:0}
.pv-edit-btn:hover{border-color:var(--teal);color:var(--teal);transform:translateY(-1px);box-shadow:0 3px 10px rgba(0,240,255,.15)}
.pv-edit-btn:active{transform:translateY(0)}
.pv-edit-btn.on{background:var(--teal3);border-color:var(--teal);color:var(--teal)}
.pv-edit-btn[disabled]{opacity:.35;cursor:default;pointer-events:none;transform:none;box-shadow:none}
#pvAddSrc:hover{border-color:var(--coral);color:var(--coral);box-shadow:0 3px 10px rgba(255,110,110,.15)}
#pvAddDst:hover{border-color:var(--neon);color:var(--neon);box-shadow:0 3px 10px rgba(80,230,150,.15)}
#pvUndoBtn:hover{border-color:var(--amber);color:var(--amber);box-shadow:0 3px 10px rgba(230,180,80,.15)}
.pv-ft{display:flex;justify-content:flex-end;align-items:center;gap:8px;padding:12px 16px;
  border-top:1px solid var(--border2);background:linear-gradient(0deg,var(--card),transparent)}
.pv-ft button{border:1px solid;border-radius:7px;padding:7px 15px;font-size:11.5px;font-weight:700;cursor:pointer;
  font-family:inherit;display:inline-flex;align-items:center;gap:6px;
  transition:transform .07s ease,box-shadow .1s,filter .1s}
.pv-ft button i{font-size:13px}
.pv-ft button:hover{transform:translateY(-1.5px)}
.pv-ft button:active{transform:translateY(0)}
.pv-ft .plc-ok{background:linear-gradient(135deg,var(--neon),#1fae6e);border-color:var(--neon);color:#04140b;
  font-weight:800;box-shadow:0 4px 16px rgba(80,230,150,.28)}
.pv-ft .plc-ok:hover{filter:brightness(1.08);box-shadow:0 6px 20px rgba(80,230,150,.4)}
.pv-ft .pv-skip-btn{border-color:var(--amber);color:var(--amber);background:var(--amber2)}
.pv-ft .pv-skip-btn:hover{background:var(--amber);color:#1a1300}
.pv-ft .pv-close-btn{background:transparent;border-color:var(--border2);color:var(--muted)}
.pv-ft .pv-close-btn:hover{border-color:var(--txt);color:var(--txt)}
.gd-body{padding:18px 16px;font-size:12.5px;line-height:1.8}
/* blocul tinta corespunzator, evidentiat in mod ghidat */
.hl-line.t-focus{background:var(--focus2);box-shadow:inset 5px 0 0 var(--focus),0 0 0 4px var(--focus3),0 4px 16px rgba(0,0,0,.3);border-left:4px solid var(--focus);border-right:4px solid var(--focus);position:relative;z-index:4}
.hl-line.t-focus.tf-first{border-top:4px solid var(--focus)}
.hl-line.t-focus.tf-last{border-bottom:4px solid var(--focus)}
/* vedere INLINE (diff unificat, read-only) - tabel cu coloane fixe, ca in
   editorul de referinta: nr-linie-stanga, nr-linie-dreapta, gutter (sageata
   de aplicare), divider, cod. Fara asta textul stanga/dreapta "aluneca" unul
   in altul si sageata se suprapune peste cod. */
.inlinewrap{display:none;flex:1 1 0;min-height:0;overflow:auto;position:relative;z-index:2;background:var(--navy)}
.inlinewrap.on{display:block}
.il-hint{position:sticky;top:0;z-index:3;padding:5px 16px;font-size:10.5px;color:var(--muted);background:var(--navy2);border-bottom:1px solid var(--border2)}
.il-table{width:100%;border-collapse:collapse;table-layout:fixed;font-family:var(--mono);font-size:11px;line-height:1.7}
.il-table col.il-col-ln{width:36px}
.il-table col.il-col-gut{width:58px}
.il-table col.il-col-div{width:1px}
.il-table td{padding:0 8px;white-space:pre-wrap;word-break:break-word;vertical-align:top}
.il-table tr.il-blk-first td{border-top:1px solid var(--border2)}
.il-ln{color:var(--muted);opacity:.55;text-align:right;user-select:none;font-size:10px;padding-top:2px}
.il-div{background:var(--border2);padding:0!important}
.il-gut{position:relative;text-align:center;padding:4px 0!important}
.il-gut-wrap{display:flex;flex-direction:column;align-items:center;justify-content:center;gap:3px;min-height:100%}
.il-mk{display:inline-block;width:14px;font-weight:900;user-select:none}
tr.il-same td.il-code{color:var(--txt)}
tr.il-rem td.il-code{background:var(--coral2)}
tr.il-rem .il-mk{color:var(--coral)}
tr.il-add td.il-code{background:var(--neon2)}
tr.il-add .il-mk{color:var(--neon)}
tr.il-mod-a td.il-code{background:var(--coral2)}
tr.il-mod-a .il-mk{color:var(--coral)}
tr.il-mod-b td.il-code{background:var(--amber2)}
tr.il-mod-b .il-mk{color:var(--amber)}
tr[data-dif="1"]{cursor:pointer}
tr[data-dif="1"]:hover td.il-code{filter:brightness(1.08)}
tr.il-ghid-cur td{box-shadow:inset 0 2px 0 var(--teal),inset 0 -2px 0 var(--teal),0 0 0 2px var(--teal3)}
tr.il-ghid-cur td.il-code:first-of-type{box-shadow:inset 4px 0 0 var(--teal)}
.il-poz{font-size:8.5px;font-weight:800;color:var(--muted);white-space:nowrap}
.il-gut-btns{display:flex;gap:4px;z-index:3}
.il-arrow,.il-eye{width:22px;height:22px;border-radius:6px;cursor:pointer;font-size:12px;font-weight:900;line-height:1;display:inline-flex;align-items:center;justify-content:center;padding:0}
.il-arrow{border:1px solid var(--teal);background:var(--teal2);color:var(--teal)}
.il-arrow:hover{background:var(--teal);color:#fff}
.il-eye{border:1px solid var(--border);background:var(--card);color:var(--txt);font-size:13px}
.il-eye:hover{background:var(--teal);color:#fff;border-color:var(--teal)}
.gmb-row td{padding:0!important}
.il-prev{display:none;position:fixed;z-index:70;max-width:520px;max-height:320px;overflow:auto;background:var(--navy);border:1px solid var(--border);border-radius:8px;box-shadow:0 12px 40px rgba(0,0,0,.45);padding:10px 12px;font-family:var(--mono);font-size:10.5px;line-height:1.6;white-space:pre-wrap}
.il-prev.on{display:block}
.il-prev .mp-t{font-family:-apple-system,'Segoe UI',sans-serif;font-size:10px;font-weight:900;letter-spacing:.5px;text-transform:uppercase;margin:6px 0 3px}
.il-prev .mp-t.mp-rem{color:var(--coral)}
.il-prev .mp-t.mp-add{color:var(--neon)}
.il-prev .mp-sec+.mp-sec{margin-top:8px;border-top:1px dashed var(--border2);padding-top:6px}
</style>
<script>
// ── COMPATIBILITATE MOTOARE FARA VARIABILE CSS (IE11/mshtml si similare) ──────
// Toata tema aplicatiei foloseste variabile CSS (var(--culoare)), suportate de
// orice Chromium/Edge/Firefox/Safari modern din 2016+. Pe un server unde
// WebView2 nu e disponibil, pywebview poate cadea automat pe motorul vechi
// Internet Explorer (mshtml/Trident), care NU intelege deloc var(--x) - de
// aceea aplicatia arata complet alb-negru (fiecare regula de culoare e pur si
// simplu ignorata de browser, layout-ul insa functioneaza pentru ca restul
// CSS-ului e standard). Scriptul de mai jos detecteaza acest caz si "rezolva"
// manual toate variabilele (le inlocuieste cu valoarea lor fixa, o singura
// data, in JS) inainte ca pagina sa fie afisata - astfel tema implicita
// (GitHub Light) apare corect si pe motoare fara suport var(), fara sa
// afecteze cu nimic randarea pe un motor modern (acolo scriptul nu face nimic).
(function(){
  var suportaVar=false;
  try{ suportaVar=!!(window.CSS && CSS.supports && CSS.supports('color','var(--x)')); }catch(e){}
  if(suportaVar) return;   // motor modern: variabilele CSS merg nativ, nu facem nimic

  function ruleazaPolyfill(){
    try{
      // 1) citim TOATE definitiile de variabile CSS (--nume:valoare;) din toate
      // blocurile de stil - nu doar din :root, pentru ca unele (ex: latimile de
      // coloana --cw-*) sunt definite pe alte selectoare (#el{--cw-idx:38px;...})
      var harta={};
      var reDef=/--([a-zA-Z0-9-]+)\s*:\s*([^;]+);/g;
      ['dtheme','mainStyles','featExtra'].forEach(function(id){
        var el=document.getElementById(id);
        if(!el) return;
        var txt=el.textContent, m;
        reDef.lastIndex=0;
        while((m=reDef.exec(txt))!==null){ harta['--'+m[1]]=m[2].trim(); }
      });

      function rezolva(text){
        return text.replace(/var\(\s*(--[a-zA-Z0-9-]+)\s*\)/g, function(orig, nume){
          return (harta[nume]!==undefined) ? harta[nume] : orig;
        });
      }

      // 2) inlocuim continutul blocurilor <style> care folosesc var(--x)
      ['mainStyles','featExtra'].forEach(function(id){
        var el=document.getElementById(id);
        if(el) el.textContent=rezolva(el.textContent);
      });

      // 3) inlocuim si style="..." inline care folosesc var(--x), prezente deja in pagina
      function rezolvaInline(radacina){
        var toate=radacina.querySelectorAll('[style*="var\\("]');
        for(var i=0;i<toate.length;i++){
          var at=toate[i].getAttribute('style');
          if(at) toate[i].setAttribute('style', rezolva(at));
        }
      }
      rezolvaInline(document);

      // 4) multe randuri (lista de obiecte, legenda terminalului, badge-uri) se
      // genereaza DINAMIC de JS dupa incarcarea paginii (inclusiv cu var(--x) in
      // atributul style) - urmarim DOM-ul in continuare si rezolvam si acelea
      var obsPolyfill=new MutationObserver(function(muts){
        for(var i=0;i<muts.length;i++){
          var m=muts[i];
          for(var j=0;j<m.addedNodes.length;j++){
            var n=m.addedNodes[j];
            if(n.nodeType===1){
              if(n.getAttribute && n.getAttribute('style') && n.getAttribute('style').indexOf('var(')!==-1){
                n.setAttribute('style', rezolva(n.getAttribute('style')));
              }
              if(n.querySelectorAll) rezolvaInline(n);
            }
          }
        }
      });
      obsPolyfill.observe(document.body,{childList:true,subtree:true});

      // 5) ascundem selectorul de teme - pe acest motor ramane fixa tema GitHub Light,
      // fiindca a rezolva variabilele o singura data nu mai permite comutarea live
      var temaWrap=document.querySelector('.tema-wrap');
      if(temaWrap) temaWrap.style.display='none';
    }catch(e){ /* nu blocam restul aplicatiei daca polyfill-ul esueaza */ }
  }
  if(document.readyState==='loading') document.addEventListener('DOMContentLoaded',ruleazaPolyfill);
  else ruleazaPolyfill();
})();
</script>
<div class="screen" id="ed">
  <div class="grid-bg"></div>
  <div class="dtp">
    <button class="bbk" onclick="inapoiLista()"><i class="ti ti-arrow-left" aria-hidden="true"></i><span id="bbkTxt">Lista</span></button>
    <div>
      <div class="det-obj" id="detN"></div>
      <div class="det-meta" id="detM"></div>
    </div>
    <div style="display:flex;align-items:center;gap:4px;margin-left:6px">
      <button class="tbb" id="btnObjPrev" onclick="navObiect(-1)" title="Obiectul anterior din lista (PageUp / Alt+Stanga)"><i class="ti ti-arrow-left" aria-hidden="true"></i></button>
      <span class="tinf" id="objNavI" style="min-width:46px;text-align:center;display:inline-block">-</span>
      <button class="tbb" id="btnObjNext" onclick="navObiect(1)" title="Obiectul urmator din lista (PageDown / Alt+Dreapta)"><i class="ti ti-arrow-right" aria-hidden="true"></i></button>
    </div>
    <button class="bconcl" id="btnConcl" onclick="toggleRezolvat()">
      <i class="ti ti-circle-dashed" id="conclIco" aria-hidden="true"></i>Am ajuns la o concluzie
    </button>
    <button class="bconcl bomite" id="btnOmite" onclick="toggleOmis()" title="Marcheaza ca analizat, dar fara sa introduci nimic in deploy pentru acest obiect">
      <i class="ti ti-circle-minus" id="omiteIco" aria-hidden="true"></i>Omite
    </button>
  </div>
  <div class="lvbar">
    <span class="lvdot"></span>
    Modificarile se aplica in timp real ·
    <span id="infMsg" style="font-weight:700;margin-left:3px">0 aplicate</span>
    <span id="savedTs" style="margin-left:10px;opacity:.75"></span>
  </div>
  <div class="dtb" id="dtbBar">
    <button class="tbb grn" onclick="adaugaTot()"><i class="ti ti-git-merge" aria-hidden="true"></i>Aplica toate</button>
    <button class="tbb" onclick="resetTot()"><i class="ti ti-refresh" aria-hidden="true"></i>Reset</button>
    <div class="tsep"></div>
    <button class="tbb" onclick="navDif(-1)" id="btnNavUp" title="Diferenta anterioara"><i class="ti ti-chevron-up" aria-hidden="true"></i></button>
    <span class="nind" id="navI">-</span>
    <button class="tbb" onclick="navDif(1)" id="btnNavDown" title="Diferenta urmatoare"><i class="ti ti-chevron-down" aria-hidden="true"></i></button>
    <div class="tsep"></div>
    <span class="tinf"><span class="tcnt" id="aplCnt">0</span> mod, <span id="totCnt">0</span> total</span>
    <div class="tsep"></div>
    <button class="tbb" id="btnGhid" onclick="toggleGhidat()" title="Mod ghidat: Enter = aplica, Esc = sari peste, Sus/Jos = navigare intre diferente"><i class="ti ti-player-play-filled" aria-hidden="true"></i><span id="ghidTxt">Mod ghidat</span></button>
    <span class="tinf" id="rezCnt" style="display:none"></span>
    <div class="tsep"></div>
    <button class="tbb grn" id="btnVSide" onclick="setVedere('side')" title="Doua panouri: sursa + tinta editabila">&#9776; Side-by-side</button>
    <button class="tbb" id="btnVInline" onclick="setVedere('inline')" title="Diff unificat intr-o singura coloana (read-only)">&#8801; Inline</button>
    <button class="tbb" id="btnModDif" onclick="toggleModDif()" style="margin-left:auto" title="Comuta afisarea diferentelor peste text">
      <i class="ti ti-eye" id="modIco" aria-hidden="true"></i><span id="modTxt">Arata diferentele</span>
    </button>
    <button class="tbb blu" id="btnResync" onclick="resyncDiff()" title="Re-compara acum">
      <i class="ti ti-refresh" aria-hidden="true"></i>Re-sincronizeaza
    </button>
    <button class="tbb" id="btnUnesteCursor" onclick="toggleUnesteCursor()" style="display:none" title="Uneste ciclul de cursor (DECLARE...OPEN...FETCH...WHILE...CLOSE...DEALLOCATE) intr-un singur bloc">
      <i class="ti ti-git-merge" aria-hidden="true"></i><span id="unesteCursorTxt">Unește ciclul cursor</span>
    </button>
    <button class="tbb" id="btnUndoPlasare" onclick="anuleazaUltimaPlasare()" style="display:none" title="Anuleaza ultima plasare de bloc (Ctrl+Z)">
      <i class="ti ti-rotate" aria-hidden="true"></i>Anuleaza plasarea
    </button>
  </div>
  <div class="plc-bar" id="plcBar">
    <i class="ti ti-arrow-right" aria-hidden="true"></i>
    Plasezi blocul <b id="plcNume">—</b> in dreapta. Alege pozitia (gol = insereaza, bloc = inlocuieste). Sugestia e evidentiata.
    <span class="plc-sp">
      <button class="plc-ok" onclick="confirmaPlasare()"><i class="ti ti-circle-check" aria-hidden="true"></i> Plaseaza aici</button>
      <button class="plc-cancel" onclick="anuleazaPlasare()"><i class="ti ti-x" aria-hidden="true"></i> Anuleaza</button>
    </span>
  </div>
  <div class="detWrap" id="detWrap">
  <div class="dcols">
    <div class="gmb-overlay" id="gmbOverlay"></div>
    <div class="dcol">
      <div class="chd src"><span class="ctag">SURSA</span><i class="ti ti-database" aria-hidden="true"></i><span id="hSrc"></span>
        <button class="copy-src" onclick="copiazaSursa()" title="Copiaza codul sursa (referinta)"><i class="ti ti-copy" aria-hidden="true"></i>Copiaza</button>
      </div>
      <div class="cscrl" id="scSrc" onscroll="_cereRedrawGutter();_pozitioneazaGmbOverlay();_cereSincScroll(true)"><div id="dSrc"></div></div>
    </div>
    <div class="dgut" id="dGut"><svg id="dGutSvg"></svg></div>
    <div class="dcol">
      <div class="chd wrk"><span class="ctag">BAZA</span><i class="ti ti-database" aria-hidden="true"></i><span id="hWork"></span>
        <span class="edit-hint">editabil · live</span>
        <span class="leg-mini" title="Ce inseamna culorile in modul diferente">
          <span class="lm lm-mod"></span>modificat
          <span class="lm lm-adus"></span>adus
          <span class="lm lm-user"></span>scris de tine
        </span>
        <button class="copy-src" onclick="copiazaRezultat()" title="Copiaza rezultatul final"><i class="ti ti-copy" aria-hidden="true"></i>Copiaza</button>
      </div>
      <div class="cscrl" id="scWork" onscroll="_cereRedrawGutter();_cereSincScroll(false)">
        <div class="edit-wrap" id="editWrap">
          <pre class="edit-hl" id="editHL" aria-hidden="true"><div id="editHLInner"></div></pre>
          <textarea class="edit-ta" id="editTA" spellcheck="false" wrap="off"
            oninput="onEdit()" onscroll="syncHL()" placeholder="(cod tinta - editeaza liber)"></textarea>
        </div>
        <div class="plc-preview" id="plcPreview"></div>
        <div class="plc-targets" id="plcTargets"></div>
        <div class="eof-marker eof-tinta"><i class="ti ti-circle-check-filled" aria-hidden="true"></i>Sfarsit procedura</div>
      </div>
      <div class="diffmap" id="diffMap"></div>
    </div>
  </div>
  <div class="inlinewrap" id="inlineView">
    <div class="il-hint">Vedere inline (read-only) &middot; aplici blocuri cu &#10003; sau modul ghidat &middot; pentru editare libera comuta pe Side-by-side</div>
    <table class="il-table">
      <colgroup>
        <col class="il-col-ln"><col class="il-col-ln"><col class="il-col-gut"><col class="il-col-div"><col>
      </colgroup>
      <tbody id="ilBody"></tbody>
    </table>
  </div>
  </div>
  <div class="il-prev" id="ilPrev"></div>
</div>

<div class="pv-overlay" id="offOverlay" onclick="if(event.target===this)inchideOfflineModal()">
  <div class="pv-box off-box">
    <div class="pv-hd"><span class="pv-hd-title"><i class="ti ti-git-diff pv-hd-ico" aria-hidden="true"></i>Comparare offline</span><button class="pv-x" onclick="inchideOfflineModal()" title="Inchide (Esc)"><i class="ti ti-x" aria-hidden="true"></i></button></div>
    <div class="off-body">
      <div class="fld" style="margin-bottom:12px;flex:0 0 auto">
        <label style="text-transform:none;font-size:11px;color:var(--muted);font-weight:600"><i class="ti ti-info-circle" aria-hidden="true"></i> Compara doua bucati de cod SQL lipite manual - fara conexiune la server, acelasi motor de diff.</label>
      </div>
      <div class="frow">
        <div class="fld">
          <div class="fld-lbl-row">
            <label><i class="ti ti-arrow-left" aria-hidden="true"></i>Referinta</label>
            <span style="display:flex;gap:5px">
              <span class="fld-mini-btn" onclick="_offLipeste('offRef')"><i class="ti ti-copy" aria-hidden="true"></i>Lipeste</span>
              <span class="fld-mini-btn" onclick="_offSterge('offRef')"><i class="ti ti-x" aria-hidden="true"></i>Sterge</span>
            </span>
          </div>
          <textarea id="offRef" class="off-ta" spellcheck="false" placeholder="CREATE PROCEDURE ..."></textarea>
        </div>
        <div class="fld">
          <div class="fld-lbl-row">
            <label><i class="ti ti-arrow-right" aria-hidden="true"></i>Tinta</label>
            <span style="display:flex;gap:5px">
              <span class="fld-mini-btn" onclick="_offLipeste('offCur')"><i class="ti ti-copy" aria-hidden="true"></i>Lipeste</span>
              <span class="fld-mini-btn" onclick="_offSterge('offCur')"><i class="ti ti-x" aria-hidden="true"></i>Sterge</span>
            </span>
          </div>
          <textarea id="offCur" class="off-ta" spellcheck="false" placeholder="Codul de comparat cu referinta..."></textarea>
        </div>
      </div>
      <div class="stbar" id="stOffline" style="flex:0 0 auto;margin-top:12px"></div>
    </div>
    <div class="pv-ft">
      <button class="plc-cancel pv-close-btn" onclick="inchideOfflineModal()"><i class="ti ti-x" aria-hidden="true"></i>Inchide</button>
      <button class="plc-ok" onclick="comparaOffline()"><i class="ti ti-git-diff" aria-hidden="true"></i>Compara</button>
    </div>
  </div>
</div>
<div class="pv-overlay" id="pvOverlay" onclick="if(event.target===this)inchidePv()">
  <div class="pv-box">
    <div class="pv-hd"><span class="pv-hd-title"><i class="ti ti-git-diff pv-hd-ico" aria-hidden="true"></i><span id="pvTitlu">Previzualizare diferenta</span></span><button class="pv-x" onclick="inchidePv()" title="Inchide (Esc)"><i class="ti ti-x" aria-hidden="true"></i></button></div>
    <div class="pv-body">
      <div class="pv-col"><div class="pv-col-hd src">SURSA (referinta)</div><div class="pv-scroll" id="pvSrc"></div></div>
      <div class="pv-col"><div class="pv-col-hd wrk">TINTA (dupa aplicare)</div><div class="pv-scroll" id="pvDst"></div></div>
    </div>
    <div class="pv-final-wrap">
      <div class="pv-final-hd"><span>REZULTAT FINAL<small>ce se aplica in tinta la apasarea butonului de mai jos - poti edita inainte</small></span>
        <span class="pv-final-actions">
          <button class="pv-edit-btn" id="pvAddSrc" onclick="_pvInsereaza('src')" title="Inlocuieste tot cu codul din SURSA (o singura data, refolosibil dupa Anuleaza)"><i class="ti ti-arrow-left" aria-hidden="true"></i>Din sursa</button>
          <button class="pv-edit-btn" id="pvAddDst" onclick="_pvInsereaza('dst')" title="Inlocuieste tot cu codul din TINTA (cel inlocuit) - o singura data, refolosibil dupa Anuleaza"><i class="ti ti-arrow-right" aria-hidden="true"></i>Din tinta</button>
          <button class="pv-edit-btn" id="pvUndoBtn" onclick="_pvUndo()" title="Anuleaza ultima inlocuire (Ctrl+Z)" disabled><i class="ti ti-rotate" aria-hidden="true"></i>Anuleaza</button>
          <button class="pv-edit-btn" id="pvEditBtn" onclick="_toggleEditPv()"><i class="ti ti-pencil" aria-hidden="true"></i><span id="pvEditBtnTxt">Editeaza</span></button>
        </span>
      </div>
      <div class="pv-final-editwrap" id="pvFinalWrap">
        <pre class="pv-final-hl" id="pvFinalHL" aria-hidden="true"></pre>
        <textarea class="pv-final-ta" id="pvFinal" readonly spellcheck="false" wrap="off"
          oninput="_pvRenderHL()" onscroll="_pvSyncHL()"></textarea>
      </div>
    </div>
    <div class="pv-ft">
      <button class="plc-cancel pv-close-btn" onclick="inchidePv()"><i class="ti ti-x" aria-hidden="true"></i>Inchide</button>
      <button class="plc-cancel pv-skip-btn" id="pvSari" style="display:none"><i class="ti ti-arrow-right" aria-hidden="true"></i>Sari peste</button>
      <button class="plc-ok pv-apply-btn" id="pvAplica"><i class="ti ti-circle-check-filled" aria-hidden="true"></i>Aplica blocul</button>
    </div>
  </div>
</div>
<div class="pv-overlay" id="ghidDone" onclick="if(event.target===this)inchideGhidDone()">
  <div class="pv-box mica">
    <div class="pv-hd"><span>Toate diferentele parcurse</span><button class="pv-x" onclick="inchideGhidDone()">&#10005;</button></div>
    <div class="gd-body" id="gdBody"></div>
    <div class="pv-ft"><button class="plc-ok" onclick="marcheazaRezolvat(true);inchideGhidDone()">&#10003; Marcheaza concluzie</button><button class="plc-cancel" onclick="inchideGhidDone()">Inchide</button></div>
  </div>
</div>

<!-- (modalul CodeMirror a fost eliminat - editarea se face live in panoul din dreapta) -->

<script>
// ── ICONITE INLINE (SVG) - fara font CDN, merge offline/RDP ───────────────────
var ICONS={"alert-circle":["<path d=\"M3 12a9 9 0 1 0 18 0a9 9 0 0 0 -18 0\" /> <path d=\"M12 8v4\" /> <path d=\"M12 16h.01\" />",0],"alert-triangle":["<path d=\"M12 9v4\" /> <path d=\"M10.363 3.591l-8.106 13.534a1.914 1.914 0 0 0 1.636 2.871h16.214a1.914 1.914 0 0 0 1.636 -2.87l-8.106 -13.536a1.914 1.914 0 0 0 -3.274 0\" /> <path d=\"M12 16h.01\" />",0],"arrow-left":["<path d=\"M5 12l14 0\" /> <path d=\"M5 12l6 6\" /> <path d=\"M5 12l6 -6\" />",0],"arrow-right":["<path d=\"M5 12l14 0\" /> <path d=\"M13 18l6 -6\" /> <path d=\"M13 6l6 6\" />",0],"brand-windows":["<path d=\"M17.8 20l-12 -1.5c-1 -.1 -1.8 -.9 -1.8 -1.9v-9.2c0 -1 .8 -1.8 1.8 -1.9l12 -1.5c1.2 -.1 2.2 .8 2.2 1.9v12.1c0 1.2 -1.1 2.1 -2.2 1.9l0 .1\" /> <path d=\"M12 5l0 14\" /> <path d=\"M4 12l16 0\" />",0],"chevron-down":["<path d=\"M6 9l6 6l6 -6\" />",0],"chevron-up":["<path d=\"M6 15l6 -6l6 6\" />",0],"circle-check":["<path d=\"M3 12a9 9 0 1 0 18 0a9 9 0 1 0 -18 0\" /> <path d=\"M9 12l2 2l4 -4\" />",0],"circle-dashed":["<path d=\"M8.56 3.69a9 9 0 0 0 -2.92 1.95\" /> <path d=\"M3.69 8.56a9 9 0 0 0 -.69 3.44\" /> <path d=\"M3.69 15.44a9 9 0 0 0 1.95 2.92\" /> <path d=\"M8.56 20.31a9 9 0 0 0 3.44 .69\" /> <path d=\"M15.44 20.31a9 9 0 0 0 2.92 -1.95\" /> <path d=\"M20.31 15.44a9 9 0 0 0 .69 -3.44\" /> <path d=\"M20.31 8.56a9 9 0 0 0 -1.95 -2.92\" /> <path d=\"M15.44 3.69a9 9 0 0 0 -3.44 -.69\" />",0],"circle-minus":["<path d=\"M3 12a9 9 0 1 0 18 0a9 9 0 1 0 -18 0\" /> <path d=\"M9 12l6 0\" />",0],"circle-x":["<path d=\"M3 12a9 9 0 1 0 18 0a9 9 0 1 0 -18 0\" /> <path d=\"M10 10l4 4m0 -4l-4 4\" />",0],"code":["<path d=\"M7 8l-4 4l4 4\" /> <path d=\"M17 8l4 4l-4 4\" /> <path d=\"M14 4l-4 16\" />",0],"copy":["<path d=\"M7 9.667a2.667 2.667 0 0 1 2.667 -2.667h8.666a2.667 2.667 0 0 1 2.667 2.667v8.666a2.667 2.667 0 0 1 -2.667 2.667h-8.666a2.667 2.667 0 0 1 -2.667 -2.667l0 -8.666\" /> <path d=\"M4.012 16.737a2.005 2.005 0 0 1 -1.012 -1.737v-10c0 -1.1 .9 -2 2 -2h10c.75 0 1.158 .385 1.5 1\" />",0],"database":["<path d=\"M4 6a8 3 0 1 0 16 0a8 3 0 1 0 -16 0\" /> <path d=\"M4 6v6a8 3 0 0 0 16 0v-6\" /> <path d=\"M4 12v6a8 3 0 0 0 16 0v-6\" />",0],"device-floppy":["<path d=\"M6 4h10l4 4v10a2 2 0 0 1 -2 2h-12a2 2 0 0 1 -2 -2v-12a2 2 0 0 1 2 -2\" /> <path d=\"M10 14a2 2 0 1 0 4 0a2 2 0 1 0 -4 0\" /> <path d=\"M14 4l0 4l-6 0l0 -4\" />",0],"dots":["<path d=\"M4 12a1 1 0 1 0 2 0a1 1 0 1 0 -2 0\" /> <path d=\"M11 12a1 1 0 1 0 2 0a1 1 0 1 0 -2 0\" /> <path d=\"M18 12a1 1 0 1 0 2 0a1 1 0 1 0 -2 0\" />",0],"eye":["<path d=\"M10 12a2 2 0 1 0 4 0a2 2 0 0 0 -4 0\" /> <path d=\"M21 12c-2.4 4 -5.4 6 -9 6c-3.6 0 -6.6 -2 -9 -6c2.4 -4 5.4 -6 9 -6c3.6 0 6.6 2 9 6\" />",0],"eye-off":["<path d=\"M10.585 10.587a2 2 0 0 0 2.829 2.828\" /> <path d=\"M16.681 16.673a8.717 8.717 0 0 1 -4.681 1.327c-3.6 0 -6.6 -2 -9 -6c1.272 -2.12 2.712 -3.678 4.32 -4.674m2.86 -1.146a9.055 9.055 0 0 1 1.82 -.18c3.6 0 6.6 2 9 6c-.666 1.11 -1.379 2.067 -2.138 2.87\" /> <path d=\"M3 3l18 18\" />",0],"file-spreadsheet":["<path d=\"M14 3v4a1 1 0 0 0 1 1h4\" /> <path d=\"M17 21h-10a2 2 0 0 1 -2 -2v-14a2 2 0 0 1 2 -2h7l5 5v11a2 2 0 0 1 -2 2\" /> <path d=\"M8 11h8v7h-8l0 -7\" /> <path d=\"M8 15h8\" /> <path d=\"M11 11v7\" />",0],"function":["<path d=\"M4 6.667a2.667 2.667 0 0 1 2.667 -2.667h10.666a2.667 2.667 0 0 1 2.667 2.667v10.666a2.667 2.667 0 0 1 -2.667 2.667h-10.666a2.667 2.667 0 0 1 -2.667 -2.667l0 -10.666\" /> <path d=\"M9 15.5v.25c0 .69 .56 1.25 1.25 1.25c.71 0 1.304 -.538 1.374 -1.244l.752 -7.512a1.381 1.381 0 0 1 1.374 -1.244c.69 0 1.25 .56 1.25 1.25v.25\" /> <path d=\"M9 12h6\" />",0],"git-merge":["<path d=\"M5 18a2 2 0 1 0 4 0a2 2 0 1 0 -4 0\" /> <path d=\"M5 6a2 2 0 1 0 4 0a2 2 0 1 0 -4 0\" /> <path d=\"M15 12a2 2 0 1 0 4 0a2 2 0 1 0 -4 0\" /> <path d=\"M7 8l0 8\" /> <path d=\"M7 8a4 4 0 0 0 4 4h4\" />",0],"key":["<path d=\"M16.555 3.843l3.602 3.602a2.877 2.877 0 0 1 0 4.069l-2.643 2.643a2.877 2.877 0 0 1 -4.069 0l-.301 -.301l-6.558 6.558a2 2 0 0 1 -1.239 .578l-.175 .008h-1.172a1 1 0 0 1 -.993 -.883l-.007 -.117v-1.172a2 2 0 0 1 .467 -1.284l.119 -.13l.414 -.414h2v-2h2v-2l2.144 -2.144l-.301 -.301a2.877 2.877 0 0 1 0 -4.069l2.643 -2.643a2.877 2.877 0 0 1 4.069 0\" /> <path d=\"M15 9h.01\" />",0],"lock":["<path d=\"M5 13a2 2 0 0 1 2 -2h10a2 2 0 0 1 2 2v6a2 2 0 0 1 -2 2h-10a2 2 0 0 1 -2 -2v-6\" /> <path d=\"M11 16a1 1 0 1 0 2 0a1 1 0 0 0 -2 0\" /> <path d=\"M8 11v-4a4 4 0 1 1 8 0v4\" />",0],"package-export":["<path d=\"M12 21l-8 -4.5v-9l8 -4.5l8 4.5v4.5\" /> <path d=\"M12 12l8 -4.5\" /> <path d=\"M12 12v9\" /> <path d=\"M12 12l-8 -4.5\" /> <path d=\"M15 18h7\" /> <path d=\"M19 15l3 3l-3 3\" />",0],"palette":["<path d=\"M12 21a9 9 0 0 1 0 -18c4.97 0 9 3.582 9 8c0 1.06 -.474 2.078 -1.318 2.828c-.844 .75 -1.989 1.172 -3.182 1.172h-2.5a2 2 0 0 0 -1 3.75a1.3 1.3 0 0 1 -1 2.25\" /> <path d=\"M7.5 10.5a1 1 0 1 0 2 0a1 1 0 1 0 -2 0\" /> <path d=\"M11.5 7.5a1 1 0 1 0 2 0a1 1 0 1 0 -2 0\" /> <path d=\"M15.5 10.5a1 1 0 1 0 2 0a1 1 0 1 0 -2 0\" />",0],"percentage":["<path d=\"M16 17a1 1 0 1 0 2 0a1 1 0 1 0 -2 0\" /> <path d=\"M6 7a1 1 0 1 0 2 0a1 1 0 1 0 -2 0\" /> <path d=\"M6 18l12 -12\" />",0],"info-circle":["<path d=\"M3 12a9 9 0 1 0 18 0a9 9 0 0 0 -18 0\" /> <path d=\"M12 9h.01\" /> <path d=\"M11 12h1v4h1\" />",0],"help-circle":["<path d=\"M3 12a9 9 0 1 0 18 0a9 9 0 0 0 -18 0\" /> <path d=\"M12 17l0 .01\" /> <path d=\"M12 13.5a1.5 1.5 0 0 1 1 -1.5a2.6 2.6 0 1 0 -3 -4\" />",0],"plug":["<path d=\"M9.785 6l8.215 8.215l-2.054 2.054a5.81 5.81 0 1 1 -8.215 -8.215l2.054 -2.054\" /> <path d=\"M4 20l3.5 -3.5\" /> <path d=\"M15 4l-3.5 3.5\" /> <path d=\"M20 9l-3.5 3.5\" />",0],"plug-off":["<path d=\"M3 3l18 18\" /> <path d=\"M9.785 6.007l4.017 4.017m2.318 2.318l1.897 1.897l-2.054 2.054a5.81 5.81 0 1 1 -8.215 -8.215l.487 -.487\" /> <path d=\"M4 20l3.5 -3.5\" /> <path d=\"M15 4l-1.5 1.5\" /> <path d=\"M20 9l-1.5 1.5\" />",0],"plus":["<path d=\"M12 5l0 14\" /> <path d=\"M5 12l14 0\" />",0],"refresh":["<path d=\"M20 11a8.1 8.1 0 0 0 -15.5 -2m-.5 -4v4h4\" /> <path d=\"M4 13a8.1 8.1 0 0 0 15.5 2m.5 4v-4h-4\" />",0],"rotate":["<path d=\"M19.95 11a8 8 0 1 0 -.5 4m.5 5v-5h-5\" />",0],"search":["<path d=\"M3 10a7 7 0 1 0 14 0a7 7 0 1 0 -14 0\" /> <path d=\"M21 21l-6 -6\" />",0],"server":["<path d=\"M3 7a3 3 0 0 1 3 -3h12a3 3 0 0 1 3 3v2a3 3 0 0 1 -3 3h-12a3 3 0 0 1 -3 -3\" /> <path d=\"M3 15a3 3 0 0 1 3 -3h12a3 3 0 0 1 3 3v2a3 3 0 0 1 -3 3h-12a3 3 0 0 1 -3 -3l0 -2\" /> <path d=\"M7 8l0 .01\" /> <path d=\"M7 16l0 .01\" />",0],"square":["<path d=\"M3 5a2 2 0 0 1 2 -2h14a2 2 0 0 1 2 2v14a2 2 0 0 1 -2 2h-14a2 2 0 0 1 -2 -2v-14\" />",0],"square-check":["<path d=\"M3 5a2 2 0 0 1 2 -2h14a2 2 0 0 1 2 2v14a2 2 0 0 1 -2 2h-14a2 2 0 0 1 -2 -2v-14\" /> <path d=\"M9 12l2 2l4 -4\" />",0],"star":["<path d=\"M12 17.75l-6.172 3.245l1.179 -6.873l-5 -4.867l6.9 -1l3.086 -6.253l3.086 6.253l6.9 1l-5 4.867l1.179 6.873l-6.158 -3.245\" />",0],"switch-horizontal":["<path d=\"M16 3l4 4l-4 4\" /> <path d=\"M10 7l10 0\" /> <path d=\"M8 13l-4 4l4 4\" /> <path d=\"M4 17l9 0\" />",0],"user":["<path d=\"M8 7a4 4 0 1 0 8 0a4 4 0 0 0 -8 0\" /> <path d=\"M6 21v-2a4 4 0 0 1 4 -4h4a4 4 0 0 1 4 4v2\" />",0],"x":["<path d=\"M18 6l-12 12\" /> <path d=\"M6 6l12 12\" />",0],"circle-check-filled":["<path d=\"M17 3.34a10 10 0 1 1 -14.995 8.984l-.005 -.324l.005 -.324a10 10 0 0 1 14.995 -8.336zm-1.293 5.953a1 1 0 0 0 -1.32 -.083l-.094 .083l-3.293 3.292l-1.293 -1.292l-.094 -.083a1 1 0 0 0 -1.403 1.403l.083 .094l2 2l.094 .083a1 1 0 0 0 1.226 0l.094 -.083l4 -4l.083 -.094a1 1 0 0 0 -.083 -1.32z\" />",1],"star-filled":["<path d=\"M8.243 7.34l-6.38 .925l-.113 .023a1 1 0 0 0 -.44 1.684l4.622 4.499l-1.09 6.355l-.013 .11a1 1 0 0 0 1.464 .944l5.706 -3l5.693 3l.1 .046a1 1 0 0 0 1.352 -1.1l-1.091 -6.355l4.624 -4.5l.078 -.085a1 1 0 0 0 -.633 -1.62l-6.38 -.926l-2.852 -5.78a1 1 0 0 0 -1.794 0l-2.853 5.78z\" />",1],"player-play-filled":["<path d=\"M6 4v16a1 1 0 0 0 1.524 .852l13 -8a1 1 0 0 0 0 -1.704l-13 -8a1 1 0 0 0 -1.524 .852z\" />",1],"pencil":["<path d=\"M4 20h4l10.5 -10.5a2.828 2.828 0 1 0 -4 -4l-10.5 10.5v4\" /> <path d=\"M13.5 6.5l4 4\" />",0],"git-diff":["<path d=\"M11 16h10\" /> <path d=\"M11 16l4 4\" /> <path d=\"M11 16l4 -4\" /> <path d=\"M13 8h-10\" /> <path d=\"M13 8l-4 4\" /> <path d=\"M13 8l-4 -4\" />",0]};
function _svgIcon(nume){
  var d=ICONS[nume];
  if(!d)return'';
  var p=d[0], filled=d[1];
  var attrs=filled
    ? 'viewBox="0 0 24 24" fill="currentColor" stroke="none"'
    : 'viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"';
  return '<svg class="gi" '+attrs+'>'+p+'</svg>';
}
// converteste toate <i class="ti ti-xxx"> in SVG inline. Idempotent (marcheaza ce a facut).
function renderIcons(root){
  root=root||document;
  var els=root.querySelectorAll('i.ti:not([data-ico])');
  for(var k=0;k<els.length;k++){
    var el=els[k], cls=el.className, m=/ti-([a-z0-9-]+)/.exec(cls);
    if(!m)continue;
    var svg=_svgIcon(m[1]);
    if(!svg){el.setAttribute('data-ico','1');continue;}
    el.innerHTML=svg;
    el.setAttribute('data-ico','1');
  }
}
// dupa orice innerHTML nou, apelam renderIcons pe acel container.
</script>
<script>
// ── TEME ─────────────────────────────────────────────────────────────────────
var THEMES={
  github:'--navy:#ffffff;--navy2:#f6f8fa;--card:#ffffff;--cardhi:#eaeef2;--teal:#2f81f7;--teal2:rgba(47,129,247,.14);--teal3:rgba(47,129,247,.06);--coral:#cf222e;--coral2:rgba(207,34,46,.12);--blue:#218bff;--blue2:rgba(33,139,255,.12);--neon:#2da44e;--neon2:rgba(45,164,78,.14);--amber:#9a6700;--amber2:rgba(154,103,0,.13);--border:rgba(47,129,247,.25);--border2:rgba(31,35,40,.09);--txt:#1f2328;--muted:#59636e;--inbg:#ffffff;--kw:#8250df;--kw2:rgba(130,80,223,.16);--kw3:rgba(130,80,223,.06);--sqlvar:#bf3989;--focus:#d6409f;--focus2:rgba(214,64,159,.16);--focus3:rgba(214,64,159,.07)',
  dracula:'--navy:#1e1f29;--navy2:#282a36;--card:#343746;--cardhi:#3d4055;--teal:#bd93f9;--teal2:rgba(189,147,249,.18);--teal3:rgba(189,147,249,.07);--coral:#ff5555;--coral2:rgba(255,85,85,.16);--blue:#8be9fd;--blue2:rgba(139,233,253,.14);--neon:#50fa7b;--neon2:rgba(80,250,123,.16);--amber:#f1fa8c;--amber2:rgba(241,250,140,.14);--border:rgba(189,147,249,.25);--border2:rgba(255,255,255,.13);--txt:#f8f8f2;--muted:#6272a4;--inbg:rgba(0,0,0,.25);--kw:#bd93f9;--kw2:rgba(189,147,249,.18);--kw3:rgba(189,147,249,.07);--sqlvar:#ff79c6;--focus:#ffb86c;--focus2:rgba(255,184,108,.18);--focus3:rgba(255,184,108,.08)',
  nord:'--navy:#242933;--navy2:#2e3440;--card:#3b4252;--cardhi:#434c5e;--teal:#88c0d0;--teal2:rgba(136,192,208,.18);--teal3:rgba(136,192,208,.07);--coral:#bf616a;--coral2:rgba(191,97,106,.16);--blue:#81a1c1;--blue2:rgba(129,161,193,.14);--neon:#a3be8c;--neon2:rgba(163,190,140,.16);--amber:#ebcb8b;--amber2:rgba(235,203,139,.14);--border:rgba(136,192,208,.22);--border2:rgba(255,255,255,.13);--txt:#e5e9f0;--muted:#7b88a1;--inbg:rgba(0,0,0,.22);--kw:#81a1c1;--kw2:rgba(129,161,193,.18);--kw3:rgba(129,161,193,.07);--sqlvar:#b48ead;--focus:#d08770;--focus2:rgba(208,135,112,.18);--focus3:rgba(208,135,112,.08)',
  onedark:'--navy:#21252b;--navy2:#282c34;--card:#2c313c;--cardhi:#333842;--teal:#61afef;--teal2:rgba(97,175,239,.18);--teal3:rgba(97,175,239,.07);--coral:#e06c75;--coral2:rgba(224,108,117,.16);--blue:#56b6c2;--blue2:rgba(86,182,194,.14);--neon:#98c379;--neon2:rgba(152,195,121,.16);--amber:#e5c07b;--amber2:rgba(229,192,123,.14);--border:rgba(97,175,239,.22);--border2:rgba(255,255,255,.13);--txt:#abb2bf;--muted:#5c6370;--inbg:rgba(0,0,0,.22);--kw:#c678dd;--kw2:rgba(198,120,221,.18);--kw3:rgba(198,120,221,.07);--sqlvar:#d19a66;--focus:#e06c9c;--focus2:rgba(224,108,156,.18);--focus3:rgba(224,108,156,.08)',
  gruvbox:'--navy:#1d2021;--navy2:#282828;--card:#3c3836;--cardhi:#504945;--teal:#fe8019;--teal2:rgba(254,128,25,.18);--teal3:rgba(254,128,25,.07);--coral:#fb4934;--coral2:rgba(251,73,52,.16);--blue:#83a598;--blue2:rgba(131,165,152,.14);--neon:#b8bb26;--neon2:rgba(184,187,38,.16);--amber:#fabd2f;--amber2:rgba(250,189,47,.15);--border:rgba(254,128,25,.25);--border2:rgba(255,255,255,.13);--txt:#ebdbb2;--muted:#a89984;--inbg:rgba(0,0,0,.22);--kw:#8ec07c;--kw2:rgba(142,192,124,.18);--kw3:rgba(142,192,124,.07);--sqlvar:#d3869b;--focus:#b16286;--focus2:rgba(177,98,134,.2);--focus3:rgba(177,98,134,.08)',
  catppuccin:'--navy:#181825;--navy2:#1e1e2e;--card:#313244;--cardhi:#45475a;--teal:#cba6f7;--teal2:rgba(203,166,247,.18);--teal3:rgba(203,166,247,.07);--coral:#f38ba8;--coral2:rgba(243,139,168,.16);--blue:#89b4fa;--blue2:rgba(137,180,250,.14);--neon:#a6e3a1;--neon2:rgba(166,227,161,.16);--amber:#f9e2af;--amber2:rgba(249,226,175,.14);--border:rgba(203,166,247,.22);--border2:rgba(255,255,255,.13);--txt:#cdd6f4;--muted:#a6adc8;--inbg:rgba(0,0,0,.2);--kw:#89dceb;--kw2:rgba(137,220,235,.18);--kw3:rgba(137,220,235,.07);--sqlvar:#f5c2e7;--focus:#fab387;--focus2:rgba(250,179,135,.18);--focus3:rgba(250,179,135,.08)'
};
var temaCur='github';
function switchTheme(k){
  temaCur=k;
  document.getElementById('dtheme').textContent=':root{'+THEMES[k]+";--mono:ui-monospace,'Cascadia Code','Consolas','Courier New',monospace}";
  document.querySelectorAll('.tema-item').forEach(function(b){b.classList.toggle('active',b.getAttribute('data-tema')===k);});
  inchideTemele();
}
function inchideTemele(){
  document.querySelectorAll('.tema-dropdown').forEach(function(d){d.classList.remove('open');});
  document.querySelectorAll('.tema-btn').forEach(function(b){b.classList.remove('open');});
}
function togTema(ev){
  var wrap=(ev&&ev.currentTarget)?ev.currentTarget.closest('.tema-wrap'):document.querySelector('#el .tema-wrap');
  if(!wrap)return;
  var drop=wrap.querySelector('.tema-dropdown'), btn=wrap.querySelector('.tema-btn');
  var deschis=!drop.classList.contains('open');
  inchideTemele();
  drop.classList.toggle('open',deschis);
  btn.classList.toggle('open',deschis);
}
document.addEventListener('click',function(e){
  if(!e.target.closest('.tema-wrap')) inchideTemele();
});

// ── STARE GLOBALA ────────────────────────────────────────────────────────────
var META={referinta:'',baze:[],total:0,ident:0,lipsa:0};
var LISTA=[];
var SESIUNE_DATA=null;          // ultima sesiune citita de pe disc (raw)
var SESIUNE_RESTAURARE=null;    // {progres:{'baza::obiect':{...}}, baze:[...], referinta:''}
var sortCol='scor',sortDir=1;
var auth='sql';
var DET_ID=null,EL=null,DIF=[],nav=0;
var modificateManuale={},editorSalvat=null,cmEditor=null;
var eqAscunse=false,conclDone=false;
var ROW_H=38,BUF=8;
var _rows=[];

// ── CONECTARE ────────────────────────────────────────────────────────────────
function setAuth(a){
  auth=a;
  document.getElementById('a-sql').classList.toggle('on',a==='sql');
  document.getElementById('a-win').classList.toggle('on',a==='win');
  document.getElementById('cred-row').style.display=a==='win'?'none':'grid';
}
function arata(el,tip,html){
  el.className='stbar '+tip;
  el.innerHTML=html;
}
function toggleSrvDrop(deschis){
  document.getElementById('srvDrop').classList.toggle('open',deschis);
}
document.addEventListener('click', function(e){
  var wrap=document.getElementById('srvWrap');
  if(wrap && !wrap.contains(e.target)) toggleSrvDrop(false);
});
function alegeServer(nume){
  document.getElementById('isrv').value=nume;
  toggleSrvDrop(false);
}
function cautaServere(){
  var btn=document.getElementById('btnCauta');
  var drop=document.getElementById('srvDrop');
  btn.classList.add('spinning');
  btn.disabled=true;
  drop.innerHTML='<div class="srv-scan"><span class="spin"></span>Se cauta servere in retea...</div>';
  toggleSrvDrop(true);
  pywebview.api.cauta_servere().then(function(r){
    btn.classList.remove('spinning');
    btn.disabled=false;
    drop.innerHTML='';
    if(r.ok && r.servere.length){
      r.servere.forEach(function(s){
        var it=document.createElement('div');
        it.className='srv-item';
        it.onclick=function(){alegeServer(s.server);};
        var raw=s.raw||{};
        var ver=raw.Version?('SQL Server '+raw.Version+' · '):'';
        var portTxt=s.port?('TCP '+s.port):(raw.np?raw.np:'named pipes');
        var esteImplicita=!raw.InstanceName||raw.InstanceName==='MSSQLSERVER';
        var badges=(raw.IsClustered==='Yes'?'<span class="srv-badge" title="Instanta clusterizata (AlwaysOn / FCI)">CLUSTER</span>':'')
          +(esteImplicita?'<span class="srv-badge" style="color:var(--muted);background:var(--border2);border-color:var(--border2)" title="Instanta implicita (MSSQLSERVER)">IMPLICITA</span>':'');
        it.innerHTML='<i class="ti ti-server" aria-hidden="true"></i>'+
          '<div class="srv-info">'+
            '<span class="srv-nume">'+esc(s.server)+'</span>'+
            '<span class="srv-sub">'+esc(ver)+esc(s.ip)+' · '+esc(portTxt)+'</span>'+
          '</div>'+badges;
        drop.appendChild(it);
      });
    } else if(r.ok){
      drop.innerHTML='<div class="srv-empty"><i class="ti ti-alert-circle"></i> Niciun server gasit.<br>SQL Browser oprit sau firewall blocheaza UDP 1434.</div>';
    } else {
      drop.innerHTML='<div class="srv-empty"><i class="ti ti-circle-x"></i> '+esc(r.mesaj)+'</div>';
    }
  }).catch(function(e){
    btn.classList.remove('spinning');
    btn.disabled=false;
    drop.innerHTML='<div class="srv-empty"><i class="ti ti-circle-x"></i> Eroare la cautare: '+esc(String(e))+'</div>';
  });
}
function _formatUptime(sec){
  if(sec==null)return null;
  var z=Math.floor(sec/86400),o=Math.floor((sec%86400)/3600),m=Math.floor((sec%3600)/60);
  if(z>0)return z+'z '+o+'h';
  if(o>0)return o+'h '+m+'m';
  return m+'m';
}
function afiseazaDetaliiServer(info){
  var box=document.getElementById('srvDetails');
  if(!box)return;
  if(!info || !Object.keys(info).length){box.style.display='none';box.innerHTML='';return;}
  var h='';
  if(info.editie)h+='<span class="srv-detail-chip"><i class="ti ti-server" aria-hidden="true"></i><b>'+esc(info.editie)+'</b></span>';
  if(info.os)h+='<span class="srv-detail-chip"><i class="ti ti-brand-windows" aria-hidden="true"></i>'+esc(info.os)+'</span>';
  var up=_formatUptime(info.uptime_sec);
  if(up)h+='<span class="srv-detail-chip"><i class="ti ti-refresh" aria-hidden="true"></i>up '+up+'</span>';
  if(info.nr_baze!=null)h+='<span class="srv-detail-chip"><i class="ti ti-database" aria-hidden="true"></i><b>'+info.nr_baze+'</b> baze</span>';
  if(!h){box.style.display='none';box.innerHTML='';return;}
  box.innerHTML=h;
  box.style.display='flex';
  renderIcons(box);
}
function testConexiune(){
  var srv=document.getElementById('isrv').value.trim();
  var usr=document.getElementById('iusr').value.trim();
  var pwd=document.getElementById('ipwd').value;
  var st=document.getElementById('stTest');
  var btn=document.getElementById('btnTest');
  if(!srv){arata(st,'err','<i class="ti ti-alert-circle"></i> Completeaza serverul.');return;}
  if(auth==='sql'&&!usr){arata(st,'err','<i class="ti ti-alert-circle"></i> Completeaza utilizatorul.');return;}
  btn.disabled=true;
  arata(st,'load','<span class="spin"></span> Conectare la '+srv+'...');
  termInit();
  termAppend('Se cauta serverul "'+srv+'"...','t-info');
  termAppend('Metoda de autentificare: '+(auth==='sql'?'SQL Server Auth (utilizator '+usr+')':'Windows Auth (integrata)'),'t-info');
  termAppend('Se verifica acreditarile si disponibilitatea instantei...','t-info');
  pywebview.api.test_conexiune(srv,auth,usr,pwd).then(function(r){
    btn.disabled=false;
    if(r.ok){
      arata(st,'ok','<i class="ti ti-circle-check-filled"></i> '+esc(r.mesaj));
      termAppend('Conexiune stabilita cu succes catre "'+srv+'".','t-ok');
      afiseazaDetaliiServer(r.info);
      termAppend('Se solicita lista bazelor de date disponibile pe server...','t-info');
      incarcaBaze(r.baze);
    } else {
      document.getElementById('srvDetails').style.display='none';
      arata(st,'err','<i class="ti ti-circle-x"></i> '+esc(r.mesaj));
      termAppend('Conexiune esuata: '+r.mesaj,'t-err');
    }
  }).catch(function(e){
    btn.disabled=false;
    arata(st,'err','<i class="ti ti-circle-x"></i> Eroare neasteptata: '+esc(String(e)));
    termAppend('Eroare neasteptata la conectare: '+String(e),'t-err');
  });
}
function incarcaBaze(baze){
  var l=document.getElementById('dbl');
  var dt=document.getElementById('dbt');
  if(!baze||!baze.length){l.innerHTML='<div style="padding:12px;color:var(--muted);font-size:12px">Nicio baza gasita.</div>';termAppend('Nicio baza de date gasita pe acest server.','t-warn');return;}
  var h='<div class="dbrow" style="cursor:default"><span class="chkb lock"><i class="ti ti-lock" aria-hidden="true"></i></span><span class="dbnm ref" id="refNm">—</span><span class="radb on"><i class="ti ti-star-filled" aria-hidden="true"></i></span></div>';
  baze.forEach(function(b,i){
    h+='<div class="dbrow" onclick="togBaza('+i+')" id="dbr'+i+'">'
      +'<span class="chkb" id="chk'+i+'"><i class="ti ti-square" aria-hidden="true"></i></span>'
      +'<span class="dbnm">'+esc(b)+'</span>'
      +'<span class="radb" id="rad'+i+'" onclick="event.stopPropagation();setRef('+i+')"><i class="ti ti-star" aria-hidden="true"></i></span>'
      +'</div>';
  });
  l.innerHTML=h;
  l.classList.add('activ');
  dt.style.display='grid';
  window._baze_list=baze;
  window._baze_sel=baze.map(function(){return false;});
  window._baze_ref=-1;
  updRun();
  termAppend('S-au gasit '+baze.length+' baze de date: '+baze.join(', ')+'.','t-ok');
  termAppend('Nicio baza nu e inclusa implicit - bifeaza-le pe cele dorite si alege o baza de REFERINTA pentru a putea porni.','t-info');
  if(SESIUNE_RESTAURARE){
    var rr=SESIUNE_RESTAURARE;
    var norm=function(x){return String(x||'').trim().toLowerCase();};
    var toate=(rr.baze||[]).concat(rr.referinta?[rr.referinta]:[]).map(norm);
    var refIdx=-1;
    baze.forEach(function(b,i){
      var vrei=toate.indexOf(norm(b))!==-1;
      if(vrei!==window._baze_sel[i]) togBaza(i);
      if(norm(b)===norm(rr.referinta)) refIdx=i;
    });
    if(refIdx>=0) setRef(refIdx,baze[refIdx]);
    termAppend('Sesiune restaurata: baze si referinta preselectate automat.','t-ok');
    if(refIdx>=0){
      termAppend('Pornesc automat verificarea cu setarile din sesiune...','t-info');
      setTimeout(function(){ if(!document.getElementById('btnRun').disabled) ruleaza(); },150);
    }
  }
}
function togBaza(i){
  window._baze_sel[i]=!window._baze_sel[i];
  var chk=document.getElementById('chk'+i);
  chk.innerHTML=window._baze_sel[i]?'<i class="ti ti-square-check" aria-hidden="true"></i>':'<i class="ti ti-square" aria-hidden="true"></i>';
  chk.classList.toggle('on',window._baze_sel[i]);
  updRun();
  var nume=window._baze_list?window._baze_list[i]:'';
  termAppend((window._baze_sel[i]?'Inclusa in verificare: ':'Exclusa din verificare: ')+nume,'t-info');
}
function setRef(i,nm){
  if(nm==null) nm=window._baze_list?window._baze_list[i]:'';
  window._baze_ref=i;
  window._baze_list.forEach(function(_,j){
    var r=document.getElementById('rad'+j);
    if(r){
      r.innerHTML=j===i?'<i class="ti ti-star-filled" aria-hidden="true"></i>':'<i class="ti ti-star" aria-hidden="true"></i>';
      r.classList.toggle('on',j===i);
    }
    var row=document.getElementById('dbr'+j);
    if(row)row.classList.toggle('ref-sel',j===i);
  });
  var el=document.getElementById('refNm');
  if(el)el.textContent=nm;
  updRun();
  termAppend('Baza de REFERINTA setata: '+nm+' (toate celelalte vor fi comparate fata de aceasta).','t-ok');
}
function updRun(){
  var bif=window._baze_sel?window._baze_sel.filter(Boolean).length:0;
  var hasRef=window._baze_ref>=0;
  document.getElementById('btnRun').disabled=!(bif>=1&&hasRef);
}
var t0=0,timerId=null,runSes=0;
function ruleaza(){
  var srv=document.getElementById('isrv').value.trim();
  var usr=document.getElementById('iusr').value.trim();
  var pwd=document.getElementById('ipwd').value;
  var baze=[];
  if(window._baze_list){
    window._baze_list.forEach(function(b,i){if(window._baze_sel[i])baze.push(b);});
  }
  var ref=window._baze_list&&window._baze_ref>=0?window._baze_list[window._baze_ref]:'';
  document.getElementById('btnRun').disabled=true;
  document.getElementById('pgbar').classList.add('show');
  document.getElementById('pginfo').classList.add('show');
  termAppend('Se initiaza sesiunea de comparare pe '+baze.length+' baze: '+baze.join(', ')+'.','t-info');
  termAppend('Baza de referinta: '+ref+'. Se compara structurile (tabele, view-uri, proceduri, functii, triggere) ale celorlalte baze fata de aceasta.','t-info');
  termAppend('Se porneste scanarea...','t-info');
  t0=Date.now();
  timerId=setInterval(function(){
    var s=Math.floor((Date.now()-t0)/1000);
    document.getElementById('pgtime').textContent=s<60?s+'s':Math.floor(s/60)+'m '+(s%60)+'s';
  },500);
  pywebview.api.ruleaza(srv,auth,usr,pwd,baze,ref).then(function(r){
    if(!r.ok){
      clearInterval(timerId);
      document.getElementById('btnRun').disabled=false;
      arata(document.getElementById('stTest'),'err','<i class="ti ti-circle-x"></i> '+esc(r.mesaj));
      termAppend('Scanarea nu a putut porni: '+r.mesaj,'t-err');
      return;
    }
    lockConexiune();
    runSes++;
    _pollFail=0;
    pollProgres(runSes);
  }).catch(function(e){
    clearInterval(timerId);
    document.getElementById('btnRun').disabled=false;
    document.getElementById('pgbar').classList.remove('show');
    document.getElementById('pginfo').classList.remove('show');
    arata(document.getElementById('stTest'),'err','<i class="ti ti-circle-x"></i> Eroare la pornirea scanarii: '+esc(String(e)));
    termAppend('Eroare la pornirea scanarii: '+String(e),'t-err');
  });
}
function showTransition(msg){
  var t=document.getElementById('gmTransition');if(!t)return;
  document.getElementById('gmTransMsg').textContent=msg||'Se incarca...';
  t.classList.add('show');
}
function hideTransition(){
  var t=document.getElementById('gmTransition');if(t)t.classList.remove('show');
}
var _pollFail=0;
function pollProgres(ses){
  if(ses!==runSes)return;   // sesiune veche (s-a dat Deconecteaza) -> oprire
  pywebview.api.progres().then(function(p){
    if(ses!==runSes)return;
    _pollFail=0;
    document.getElementById('pgfill').style.width=p.procent+'%';
    var msg=p.mesaj;
    if(p.procent>5&&p.procent<100){
      var el=(Date.now()-t0)/1000;
      var eta=Math.round(el*(100-p.procent)/p.procent);
      msg+=' · ~'+(eta<60?eta+'s ramase':Math.floor(eta/60)+'m '+(eta%60)+'s ramase');
    }
    document.getElementById('pgmsg').textContent=msg;
    if(p.eroare){
      clearInterval(timerId);
      arata(document.getElementById('stTest'),'err','<i class="ti ti-circle-x"></i> '+esc(p.eroare));
      document.getElementById('btnRun').disabled=false;
      document.getElementById('pgbar').classList.remove('show');
      document.getElementById('pginfo').classList.remove('show');
      unlockConexiune();
      return;
    }
    termPollLog();
    if(p.gata){
      clearInterval(timerId);
      termAppend('Verificare finalizata. Se genereaza raportul de diferente...','t-ok');
      document.getElementById('termWrap').classList.add('term-pulse');
      showTransition('Se pregateste raportul de diferente...');
      setTimeout(function(){incarcaRez();},900);
      return;
    }
    setTimeout(function(){pollProgres(ses);},600);
  }).catch(function(e){
    if(ses!==runSes)return;
    _pollFail++;
    if(_pollFail>=5){
      // backend-ul nu mai raspunde: oprim curat in loc sa lasam UI-ul agatat
      clearInterval(timerId);
      arata(document.getElementById('stTest'),'err','<i class="ti ti-circle-x"></i> Conexiunea cu procesul de scanare s-a pierdut: '+esc(String(e)));
      termAppend('Polling intrerupt dupa 5 esecuri consecutive: '+String(e),'t-err');
      unlockConexiune();
      return;
    }
    // reincercare cu pauza crescatoare (1.2s, 2.4s, ...)
    setTimeout(function(){pollProgres(ses);},600*(_pollFail+1));
  });
}
function lockConexiune(){
  ['isrv','iusr','ipwd'].forEach(function(id){
    var el=document.getElementById(id);if(el){el.readOnly=true;el.style.opacity='.5';}
  });
  document.querySelectorAll('.aseg .abtn').forEach(function(b){b.disabled=true;b.style.opacity='.4';});
  document.getElementById('btnTest').style.display='none';
  var bc=document.getElementById('btnCauta');if(bc){bc.disabled=true;bc.style.opacity='.4';bc.style.pointerEvents='none';}
  document.getElementById('dbl').querySelectorAll('.chkb:not(.lock),.radb').forEach(function(x){x.style.pointerEvents='none';x.style.opacity='.4';});
  document.getElementById('dbl').classList.add('rulat');
  document.getElementById('lockMsg').style.display='block';
  document.getElementById('sesLista').querySelectorAll('.ses-btn').forEach(function(b){
    b.disabled=true;b.style.pointerEvents='none';b.style.opacity='.4';
  });
  document.querySelectorAll('#activeSesBanner .ses-btn').forEach(function(b){
    b.disabled=true;b.style.pointerEvents='none';b.style.opacity='.4';
  });
  ['tab-conectare','tab-sesiuni'].forEach(function(id){
    var b=document.getElementById(id);if(b){b.disabled=true;b.style.pointerEvents='none';b.style.opacity='.5';}
  });
}
function _listaHelpHTML(){
  return '<div class="term-legend">'
    +'<div class="tl-corner tl-c1"></div><div class="tl-corner tl-c2"></div><div class="tl-corner tl-c3"></div><div class="tl-corner tl-c4"></div>'
    +'<div class="tl-sec"><div class="tl-rail"><span class="tl-dot"></span><span class="tl-line"></span></div><div class="tl-body">'
      +'<div class="tl-tag">§ 01</div>'
      +'<div class="tl-title">Ce arata acest ecran</div>'
      +'<div class="tl-desc">Fiecare rand e un obiect (procedura, view, functie, trigger) dintr-o baza <b>TINTA</b>, comparat cu cel din baza de <b>REFERINTA</b>. Coloana <b>Similaritate</b> arata cat de apropiat e codul; <b>Diferente</b> rezuma ce s-a schimbat; <b>Stare</b> arata daca ai tras o concluzie.</div>'
    +'</div></div>'
    +'<div class="tl-sec"><div class="tl-rail"><span class="tl-dot"></span><span class="tl-line"></span></div><div class="tl-body">'
      +'<div class="tl-tag">§ 02</div>'
      +'<div class="tl-title">Cauta, filtreaza, sorteaza</div>'
      +'<div class="tl-desc">'
        +'<b>Cauta obiect sau diferenta</b> filtreaza direct in lista, pe masura ce tastezi.<br>'
        +'Dropdown-urile de <b>baza / tip / categorie / stare</b> se pot combina intre ele.<br>'
        +'Intervalul <b>%</b> arata doar obiectele cu similaritate in acel interval.<br>'
        +'Click pe un antet de coloana (ex: Similaritate) sorteaza lista; trage marginea coloanei ca sa-i schimbi latimea, dublu-click pe margine = auto-fit.'
      +'</div>'
    +'</div></div>'
    +'<div class="tl-sec"><div class="tl-rail"><span class="tl-dot"></span><span class="tl-line"></span></div><div class="tl-body">'
      +'<div class="tl-tag">§ 03</div>'
      +'<div class="tl-title">Simbolurile din aceasta lista</div>'
      +'<div class="tl-tokens">'
        +'<span class="tl-tok"><span class="d" style="background:var(--neon);box-shadow:0 0 6px var(--neon)"></span>Rezolvat</span>'
        +'<span class="tl-tok"><span class="d" style="background:transparent;border:2px solid var(--muted)"></span>Nerezolvat</span>'
        +'<span class="tl-tok"><span class="d" style="background:var(--muted);box-shadow:0 0 6px rgba(89,99,110,.4)"></span>Omis (analizat, exclus din deploy)</span>'
        +'<span class="tl-tok"><span class="d" style="background:var(--teal)"></span>Trimis la deploy</span>'
        +'<span class="tl-tok"><span class="d" style="background:var(--blue)"></span>Pregatit pentru deploy</span>'
        +'<span class="tl-tok"><span class="d" style="background:var(--amber)"></span>Versiune noua / modificat de atunci</span>'
        +'<span class="tl-tok"><span class="d" style="background:var(--muted)"></span>Neintrodus in deploy</span>'
      +'</div>'
      +'<div class="tl-desc" style="margin-top:8px">Obiectele marcate <b>EXTRA</b> exista doar in baza tinta, nu si in referinta — nu pot fi niciodata deployate, indiferent de stare.</div>'
    +'</div></div>'
    +'<div class="tl-sec"><div class="tl-rail"><span class="tl-dot"></span><span class="tl-line"></span></div><div class="tl-body">'
      +'<div class="tl-tag">§ 04</div>'
      +'<div class="tl-title">In editorul de diferente (Diff)</div>'
      +'<div class="tl-desc">Apasa <b>Diff</b> pe un rand ca sa vezi codul side-by-side si sa-l editezi. Cand esti gata, apasa <b>Am ajuns la o concluzie</b> (sau <b>Aplica toate</b>, care o bifeaza automat) inainte de a te intoarce la lista.</div>'
      +'<div class="tl-tokens">'
        +'<span class="tl-tok"><span class="d" style="background:var(--amber)"></span>Modificat</span>'
        +'<span class="tl-tok"><span class="d" style="background:var(--neon)"></span>Adus din sursa</span>'
        +'<span class="tl-tok"><span class="d" style="background:var(--kw)"></span>Scris de tine</span>'
      +'</div>'
    +'</div></div>'
    +'<div class="tl-sec"><div class="tl-rail"><span class="tl-dot" style="background:var(--muted);box-shadow:none"></span></div><div class="tl-body">'
      +'<div class="tl-hint">Butonul <b>Deployer</b> genereaza scriptul .sql doar pentru obiectele rezolvate (fara EXTRA) — poti alege un fisier per obiect, per baza, grupat, sau totul intr-unul singur.<span class="tl-cursor"></span></div>'
    +'</div></div>'
    +'</div>';
}
function deschideHelpLegenda(){
  var body=document.getElementById('helpBody');
  if(body) body.innerHTML=_listaHelpHTML();
  document.getElementById('helpOverlay').classList.add('on');
  renderIcons(document.getElementById('helpOverlay'));
}
function inchideHelpLegenda(){
  document.getElementById('helpOverlay').classList.remove('on');
}
function deconecteaza(){
  document.getElementById('confirmOverlay').classList.add('on');
}
function inchideConfirm(){
  document.getElementById('confirmOverlay').classList.remove('on');
}
function confirmaDeconectare(){
  inchideConfirm();
  if(META&&LISTA&&LISTA.length)salveazaSesiune();
  unlockConexiune();
  verificaSesiuneSalvata();   // reafiseaza sesiunile salvate in tab-ul dedicat
  switchLcbTab('sesiuni');
}
function unlockConexiune(){
  // opreste rularea in curs (daca exista) si invalideaza poll-ul vechi
  runSes++;
  clearInterval(timerId);
  try{pywebview.api.anuleaza();}catch(x){}
  var sd=document.getElementById('srvDetails');if(sd){sd.style.display='none';sd.innerHTML='';}
  ['isrv','iusr','ipwd'].forEach(function(id){
    var el=document.getElementById(id);if(el){el.readOnly=false;el.style.opacity='';}
  });
  document.querySelectorAll('.aseg .abtn').forEach(function(b){b.disabled=false;b.style.opacity='';});
  document.getElementById('btnTest').style.display='';
  var bc=document.getElementById('btnCauta');if(bc){bc.disabled=false;bc.style.opacity='';bc.style.pointerEvents='';}
  document.getElementById('dbl').querySelectorAll('.chkb:not(.lock),.radb').forEach(function(x){x.style.pointerEvents='';x.style.opacity='';});
  document.getElementById('dbl').classList.remove('rulat');
  document.getElementById('dbl').classList.remove('activ');
  document.getElementById('dbt').style.display='none';
  document.getElementById('lockMsg').style.display='none';
  SESIUNE_RESTAURARE=null;
  actualizeazaBannerSesiuneActiva(null);
  document.getElementById('sesLista').querySelectorAll('.ses-btn').forEach(function(b){
    b.disabled=false;b.style.pointerEvents='';b.style.opacity='';
  });
  document.querySelectorAll('#activeSesBanner .ses-btn').forEach(function(b){
    b.disabled=false;b.style.pointerEvents='';b.style.opacity='';
  });
  ['tab-conectare','tab-sesiuni'].forEach(function(id){
    var b=document.getElementById(id);if(b){b.disabled=false;b.style.pointerEvents='';b.style.opacity='';}
  });
  document.getElementById('btnRun').disabled=true;
  document.getElementById('stTest').className='stbar';
  document.getElementById('pgbar').classList.remove('show');
  document.getElementById('pginfo').classList.remove('show');
  document.getElementById('pgfill').style.width='0%';
  document.getElementById('pgmsg').textContent='';
  document.getElementById('pgtime').textContent='';
  document.getElementById('ec').classList.add('active');
  document.getElementById('el').classList.remove('active');
  document.getElementById('ed').classList.remove('active');
  hideTransition();
  document.getElementById('termWrap').classList.remove('term-pulse');
  try{pywebview.api.win_resize('mic');}catch(e){}
}
function incarcaRez(){
  pywebview.api.rezultate().then(function(res){
    META=res.meta;LISTA=res.lista;
    document.getElementById('ec').classList.remove('active');
    document.getElementById('el').classList.add('active');
    hideTransition();
    document.getElementById('termWrap').classList.remove('term-pulse');
    try{pywebview.api.win_resize('mare');}catch(e){}
    var _srvNume=document.getElementById('isrv').value.trim();
    var _bazeCmp=(META.baze||[]).map(_esc).join(', ');
    document.getElementById('lsub').innerHTML=
      (_srvNume?'<i class="ti ti-server" aria-hidden="true" style="font-size:12px;vertical-align:-1px"></i> <b style="color:var(--teal)">'+_esc(_srvNume)+'</b> &bull; ':'')
      +'Referinta: <b style="color:var(--teal)">'+_esc(META.referinta)+'</b>'
      +' &bull; Compara: <span title="Bazele tinta verificate fata de referinta">'+_bazeCmp+'</span>'
      +' &bull; '+META.total+' obiecte';
    var dif=LISTA.filter(function(x){return x.cat==='dif';}).length;
    var extraH=(META.extra||0)>0?'<div class="scp scp-a" title="Exista in tinta, dar nu in referinta"><i class="ti ti-plus" aria-hidden="true"></i>'+META.extra+' extra</div>':'';
    document.getElementById('lstats').innerHTML=
      '<div class="scp scp-r"><i class="ti ti-alert-triangle" aria-hidden="true"></i>'+dif+' dif</div>'
      +'<div class="scp scp-a"><i class="ti ti-circle-minus" aria-hidden="true"></i>'+META.lipsa+' lipsa</div>'
      +extraH
      +'<div class="scp scp-g"><i class="ti ti-circle-check" aria-hidden="true"></i>'+META.ident+' ok</div>';
    var sel=document.getElementById('fBaza');
    sel.innerHTML='<option value="">(toate bazele)</option>';
    META.baze.forEach(function(b){
      var o=document.createElement('option');o.value=b;o.textContent=b;sel.appendChild(o);
    });
    document.getElementById('fNume').value='';
    document.getElementById('fTip').value='';
    document.getElementById('fCat').value='';
    document.getElementById('fStare').value='';
    document.getElementById('fScMin').value=0;
    document.getElementById('fScMax').value=100;
    if(SESIUNE_RESTAURARE && SESIUNE_RESTAURARE.progres){
      var prog=SESIUNE_RESTAURARE.progres;
      LISTA.forEach(function(d){
        var p=prog[d.baza+'::'+d.obiect];
        if(!p) return;
        var neschimbat = (p.ref_hash===d.ref_hash) && (p.cur_hash===d.cur_hash) && d.ref_hash!=null && d.cur_hash!=null;
        if(neschimbat){
          d.rezolvat=!!p.rezolvat;
          d.omis=!!p.omis;
          d.adaugat=p.adaugat||{};
          if(p.rezultatEditat!=null) d.rezultatEditat=p.rezultatEditat;
          if(p.deployat) d.deployat=p.deployat;
          if(p.modificatLa) d.modificatLa=p.modificatLa;
        } else if(p.deployat || p.rezolvat || p.omis){
          // obiectul s-a schimbat intre timp (referinta sau tinta) - NU refolosim
          // starea veche (ar putea aplica editari gresite pe un diff diferit).
          // Pastram doar un semnal vizual ca a existat progres anterior, invechit.
          d.sesiuneInvechita={deployat:p.deployat||null,rezolvat:!!p.rezolvat,omis:!!p.omis};
        }
      });
      var nrInvechite=LISTA.filter(function(x){return x.sesiuneInvechita;}).length;
      var nrAplicate=LISTA.filter(function(x){return x.rezolvat && prog[x.baza+'::'+x.obiect];}).length;
      termAppend('Progresul din sesiunea salvata a fost aplicat pe '+nrAplicate+' obiecte neschimbate.','t-ok');
      if(nrInvechite){
        termAppend(nrInvechite+' obiecte s-au modificat intre timp - progresul vechi NU a fost reaplicat automat, verifica-le din nou.','t-warn');
      }
    }
    SESIUNE_RESTAURARE=null;
    actualizeazaBannerSesiuneActiva(null);
    initVirtualScroll();
    renderLista();
    salveazaSesiune();
  }).catch(function(e){
    hideTransition();
    document.getElementById('termWrap').classList.remove('term-pulse');
    arata(document.getElementById('stTest'),'err','<i class="ti ti-circle-x"></i> Eroare la incarcarea rezultatelor: '+esc(String(e)));
    termAppend('Eroare la incarcarea rezultatelor: '+String(e),'t-err');
    unlockConexiune();
  });
}
// ── SESIUNE (progres criptat, persistent local) ───────────────────────────────
// ── SESIUNE (progres criptat, persistent local) ───────────────────────────────
function switchLcbTab(tab){
  var isC=tab==='conectare';
  document.getElementById('tab-conectare').classList.toggle('on',isC);
  document.getElementById('tab-sesiuni').classList.toggle('on',!isC);
  document.getElementById('panelConectare').style.display=isC?'':'none';
  document.getElementById('panelSesiuni').style.display=isC?'none':'';
  var lcb=document.querySelector('.lcb');
  if(lcb) lcb.scrollTop=0;
}
function _pct(n,total){
  if(!total||!n)return '0';
  var raw=n/total*100;
  var rotunjit=Math.round(raw);
  return rotunjit===0?raw.toFixed(2):String(rotunjit);
}
var SESIUNE_MAP={}; // {referinta: sesiune completa}
var _nrSesiuniCache=0;
function actualizeazaSesHint(){
  var el=document.getElementById('sesHint');
  var txt=document.getElementById('sesHintTxt');
  if(!el||!txt)return;
  if(SESIUNE_RESTAURARE){ el.style.display='none'; return; }
  if(!_nrSesiuniCache){
    el.className='ses-hint';
    txt.textContent='Nu exista sesiuni disponibile.';
  } else {
    el.className='ses-hint warn';
    txt.textContent='Nicio sesiune selectata — la conectare se creeaza o sesiune noua (sau se suprascrie cea existenta pentru aceeasi referinta).';
  }
  el.style.display='flex';
  renderIcons(el);
}
function verificaSesiuneSalvata(){
  pywebview.api.lista_sesiuni().then(function(r){
    var cont=document.getElementById('sesLista');
    cont.innerHTML='';
    SESIUNE_MAP={};
    var lst=(r.ok?(r.sesiuni||[]):[]);
    _nrSesiuniCache=lst.length;
    actualizeazaSesHint();
    var badge=document.getElementById('sesBadge');
    if(badge){
      if(lst.length){badge.textContent=lst.length;badge.style.display='inline-flex';}
      else{badge.style.display='none';}
    }
    if(!lst.length){
      cont.innerHTML='<div class="ses-empty">Nicio sesiune salvata inca.</div>';
      return;
    }
    lst.forEach(function(s){
      var cand=s.salvat_la?new Date(s.salvat_la).toLocaleString('ro-RO'):'?';
      var div=document.createElement('div');
      div.className='sesbanner';
      div.innerHTML=
        '<div class="sesbanner-top">'+
          '<i class="ti ti-device-floppy" aria-hidden="true"></i>'+
          '<div class="sesbanner-txt">'+
            '<b>Sesiune gasita</b>'+
            '<span><strong>Server:</strong> '+esc(s.server||'?')+'</span>'+
            '<span><strong>Referinta:</strong> '+esc(s.referinta||'?')+'</span>'+
            '<span><strong>Progres:</strong> '+s.total+' obiecte — <strong>'+_pct(s.rezolvate,s.total)+'%</strong> rezolvate, <strong>'+_pct(s.deploy,s.total)+'%</strong> deploy</span>'+
            '<span><strong>Salvat la:</strong> '+cand+'</span>'+
          '</div>'+
        '</div>'+
        '<div class="sesbanner-btns">'+
          '<button class="ses-btn ses-btn-on" data-ref="'+esc(s.referinta)+'"><i class="ti ti-player-play-filled" aria-hidden="true"></i>Reia</button>'+
          '<button class="ses-btn" data-ref="'+esc(s.referinta)+'"><i class="ti ti-x" aria-hidden="true"></i>Ignora</button>'+
        '</div>';
      div.querySelector('.ses-btn-on').onclick=function(){ reiaSesiune(s.referinta); };
      div.querySelector('.ses-btn:not(.ses-btn-on)').onclick=function(){ stergeSesiune(s.referinta); };
      cont.appendChild(div);
    });
    renderIcons(cont);
  });
}
function actualizeazaBannerSesiuneActiva(s){
  var ban=document.getElementById('activeSesBanner');
  if(!ban)return;
  var tab=document.getElementById('tab-conectare');
  if(!s){ban.style.display='none';if(tab)tab.classList.remove('has-ses');actualizeazaSesHint();return;}
  var lst=s.lista||[];
  var rez=lst.filter(function(x){return x.rezolvat;}).length;
  var dep=lst.filter(function(x){return x.deployat;}).length;
  var cand=s.salvat_la?new Date(s.salvat_la).toLocaleString('ro-RO'):'?';
  document.getElementById('activeSesInfo').innerHTML=
    '<span><strong>Server:</strong> '+esc(s.server||'?')+'</span>'+
    '<span><strong>Referinta:</strong> '+esc(s.referinta||'?')+'</span>'+
    '<span><strong>Progres:</strong> '+lst.length+' obiecte — <strong>'+_pct(rez,lst.length)+'%</strong> rezolvate, <strong>'+_pct(dep,lst.length)+'%</strong> deploy</span>'+
    '<span><strong>Salvat la:</strong> '+cand+'</span>';
  ban.style.display='flex';
  if(tab)tab.classList.add('has-ses');
  actualizeazaSesHint();
  renderIcons(ban);
}
function anuleazaSesiuneActiva(){
  SESIUNE_RESTAURARE=null;
  actualizeazaBannerSesiuneActiva(null);
}
function reiaSesiune(referinta){
  pywebview.api.incarca_sesiune(referinta).then(function(r){
    if(!r.ok || !r.exista || !r.sesiune) return;
    var s=r.sesiune;
    SESIUNE_DATA=s;
    document.getElementById('isrv').value=s.server||'';
    document.getElementById('iusr').value=s.user||'';
    setAuth(s.auth==='win'?'win':'sql');
    var map={};
    (s.lista||[]).forEach(function(x){
      map[x.baza+'::'+x.obiect]={rezolvat:!!x.rezolvat,adaugat:x.adaugat||{},
        rezultatEditat:(x.rezultatEditat!=null?x.rezultatEditat:null),
        deployat:x.deployat||null,
        modificatLa:x.modificatLa||null,ref_hash:x.ref_hash||null,cur_hash:x.cur_hash||null};
    });
    SESIUNE_RESTAURARE={progres:map,baze:s.baze||[],referinta:s.referinta||''};
    actualizeazaBannerSesiuneActiva(s);
    switchLcbTab('conectare');
    var st=document.getElementById('stTest');
    arata(st,'ok','<i class="ti ti-circle-check-filled"></i> Completeaza parola si testeaza conexiunea pentru a relua sesiunea.');
  });
}
function stergeSesiune(referinta){
  pywebview.api.sterge_sesiune(referinta).then(function(){
    delete SESIUNE_MAP[referinta];
    verificaSesiuneSalvata();
  });
}
var ULTIMA_SALVARE_OK=null;
function actualizeazaIndicatorSalvare(stare, mesaj){
  var el=document.getElementById('sesSaveInfo');
  var ic=document.getElementById('sesSaveIco');
  var txt=document.getElementById('sesSaveTxt');
  if(!el||!ic||!txt)return;
  el.style.display='flex';
  el.className='ses-save-info st-'+stare;
  if(stare==='salvand'){
    txt.textContent='Se salveaza sesiunea...';
    ic.className='ti ti-refresh';
  } else if(stare==='ok'){
    txt.textContent='Sesiune salvata si verificata, la '+ULTIMA_SALVARE_OK.toLocaleTimeString('ro-RO');
    ic.className='ti ti-circle-check-filled';
    el.classList.add('pulsand');
  } else {
    var cand=ULTIMA_SALVARE_OK?(' (ultima reusita: '+ULTIMA_SALVARE_OK.toLocaleTimeString('ro-RO')+')'):'';
    txt.textContent='Eroare la salvarea sesiunii'+cand+': '+(mesaj||'necunoscuta');
    ic.className='ti ti-alert-triangle';
  }
  ic.removeAttribute('data-ico'); renderIcons(ic.parentNode);
}
function salveazaSesiune(){
  try{ _salveazaSesiuneImpl(); }catch(e){ actualizeazaIndicatorSalvare('eroare', String(e)); }
}
function _salveazaSesiuneImpl(){
  if(!META || !LISTA) return;
  var srv=document.getElementById('isrv').value.trim();
  var usr=document.getElementById('iusr').value.trim();
  var lst=LISTA.map(function(d){
    return {baza:d.baza,obiect:d.obiect,rezolvat:!!d.rezolvat,omis:!!d.omis,adaugat:d.adaugat||{},
            rezultatEditat:(d.rezultatEditat!=null?d.rezultatEditat:null),
            deployat:d.deployat||null,modificatLa:d.modificatLa||null,
            ref_hash:d.ref_hash||null,cur_hash:d.cur_hash||null};
  });
  actualizeazaIndicatorSalvare('salvand');
  pywebview.api.salveaza_sesiune({
    server:srv,auth:auth,user:usr,
    baze:META.baze||[],referinta:META.referinta||'',
    lista:lst
  }).then(function(r){
    if(!r || !r.ok){ actualizeazaIndicatorSalvare('eroare', r&&r.mesaj); return; }
    // verificare reala: recitim fisierul de pe disc (decriptare + parsare) si
    // confirmam ca numarul de obiecte salvate corespunde - abia atunci e "corect"
    pywebview.api.incarca_sesiune(META.referinta||'').then(function(r2){
      if(r2 && r2.ok && r2.exista && r2.sesiune && (r2.sesiune.lista||[]).length===lst.length){
        ULTIMA_SALVARE_OK=new Date();
        actualizeazaIndicatorSalvare('ok');
      } else {
        actualizeazaIndicatorSalvare('eroare','verificarea dupa salvare a esuat (fisierul citit nu corespunde).');
      }
    }).catch(function(e){ actualizeazaIndicatorSalvare('eroare', String(e)); });
  }).catch(function(e){ actualizeazaIndicatorSalvare('eroare', String(e)); });
}

// ── LISTA VIRTUALIZATA ────────────────────────────────────────────────────────
var TIP_NUME={'P':'PROC','V':'VIEW','FN':'FN','IF':'FN','TF':'FN','TR':'TRIG'};
var TIP_GRUP={'P':'P','V':'V','FN':'FN','IF':'FN','TF':'FN','TR':'TR'};
function filtrate(){
  var q=document.getElementById('fNume').value.toLowerCase();
  var baza=document.getElementById('fBaza').value;
  var tip=document.getElementById('fTip').value;
  var cat=document.getElementById('fCat').value;
  var stare=document.getElementById('fStare').value;
  var smin=parseFloat(document.getElementById('fScMin').value);if(isNaN(smin))smin=0;
  var smax=parseFloat(document.getElementById('fScMax').value);if(isNaN(smax))smax=100;
  return LISTA.filter(function(d){
    if(q&&d.obiect.toLowerCase().indexOf(q)<0&&(d.rezumat||'').toLowerCase().indexOf(q)<0)return false;
    if(baza&&d.baza!==baza)return false;
    if(tip&&TIP_GRUP[d.tip]!==tip)return false;
    if(cat&&d.cat!==cat)return false;
    if(stare==='da'&&!d.rezolvat)return false;
    if(stare==='ne'&&(d.rezolvat||d.omis))return false;
    if(stare==='omis'&&!d.omis)return false;
    if(stare==='dep'&&!d.deployat)return false;
    if(stare==='ndep'&&d.deployat)return false;
    if(d.scor!==null&&(d.scor<smin||d.scor>smax))return false;
    if(d.scor===null&&smin>0)return false;  // LIPSESTE/EXTRA = sub orice prag minim
    return true;
  });
}
function sortLista(col){if(sortCol===col)sortDir=-sortDir;else{sortCol=col;sortDir=1;}renderLista();}
// ── REDIMENSIONARE COLOANE LISTA ──────────────────────────────────────────────
var _grip=null;
function gripStart(ev){
  ev.preventDefault(); ev.stopPropagation();
  var g=ev.currentTarget;
  var cw=g.getAttribute('data-cw');
  var el=document.getElementById('el');
  var start=parseFloat(getComputedStyle(el).getPropertyValue(cw))||100;
  // latimea celorlalte coloane (ca sa nu depasim ecranul)
  var vars=['--cw-idx','--cw-obj','--cw-baza','--cw-tip','--cw-scor','--cw-dif','--cw-stare','--cw-mod','--cw-dep','--cw-act'];
  var alte=0;
  vars.forEach(function(v){ if(v!==cw) alte+=parseFloat(getComputedStyle(el).getPropertyValue(v))||0; });
  var latLista=document.querySelector('.lhd').clientWidth - 40; // minus padding 20+20
  var maxW=Math.max(40, latLista - alte);   // cat mai poate creste coloana curenta
  _grip={cw:cw, x0:ev.clientX, w0:start, el:el, maxW:maxW};
  g.classList.add('drag');
  document.body.style.cursor='col-resize';
  document.addEventListener('mousemove',gripMove);
  document.addEventListener('mouseup',gripEnd);
}
function gripMove(ev){
  if(!_grip)return;
  var w=_grip.w0 + (ev.clientX-_grip.x0);
  w=Math.max(40, Math.min(_grip.maxW, w));   // limitat intre 40 si maxim (nu depaseste ecranul)
  _grip.el.style.setProperty(_grip.cw, w+'px');
}
function gripEnd(){
  document.removeEventListener('mousemove',gripMove);
  document.removeEventListener('mouseup',gripEnd);
  document.querySelectorAll('.col-grip.drag').forEach(function(g){g.classList.remove('drag');});
  document.body.style.cursor='';
  _grip=null;
}
// dublu-click pe maner = auto-fit: coloana creste cat sa incapa cel mai lung text,
// dar limitat de spatiul ramas pe ecran.
function gripAutofit(ev){
  ev.preventDefault(); ev.stopPropagation();
  var cw=ev.currentTarget.getAttribute('data-cw');
  var el=document.getElementById('el');
  // mapare cw -> selector celula pt masurare
  var sel={'--cw-obj':'.onm','--cw-baza':'.baza-nm','--cw-tip':'.tip-chip','--cw-scor':'.scv','--cw-dif':'.rezm','--cw-mod':'.mod-chip','--cw-dep':'.dep-chip'}[cw];
  var maxTxt=60;
  if(sel){
    document.querySelectorAll('#rows '+sel).forEach(function(n){
      maxTxt=Math.max(maxTxt, n.scrollWidth+24);
    });
  }
  // limiteaza la spatiul disponibil
  var vars=['--cw-idx','--cw-obj','--cw-baza','--cw-tip','--cw-scor','--cw-dif','--cw-stare','--cw-mod','--cw-dep','--cw-act'];
  var alte=0; vars.forEach(function(v){ if(v!==cw) alte+=parseFloat(getComputedStyle(el).getPropertyValue(v))||0; });
  var latLista=document.querySelector('.lhd').clientWidth - 40;
  var maxW=Math.max(40, latLista - alte);
  el.style.setProperty(cw, Math.min(maxTxt, maxW)+'px');
}
function actualizeazaProgresGlobal(){
  if(!LISTA)return;
  var total=LISTA.filter(function(d){return d.cat!=='extra';}).length;
  var rez=LISTA.filter(function(d){return d.rezolvat && d.cat!=='extra';}).length;
  var dep=LISTA.filter(function(d){return d.deployat && d.cat!=='extra';}).length;
  var pr=_pct(rez,total),pd=_pct(dep,total);
  var fr=document.getElementById('progRezFill');if(fr)fr.style.width=pr+'%';
  var vr=document.getElementById('progRezVal');if(vr)vr.textContent=pr+'%';
  var fd=document.getElementById('progDepFill');if(fd)fd.style.width=pd+'%';
  var vd=document.getElementById('progDepVal');if(vd)vd.textContent=pd+'%';
}
function renderLista(pastreazaPozitia){
  _rows=filtrate();
  _rows.sort(function(a,b){
    var va=a[sortCol],vb=b[sortCol];
    if(sortCol==='scor'){va=(va===null)?-1:va;vb=(vb===null)?-1:vb;}
    if(sortCol==='rezolvat'){va=va?1:0;vb=vb?1:0;}
    // ISO 8601 se sorteaza corect lexicografic - evitam new Date() la fiecare comparatie
    if(sortCol==='deployat'||sortCol==='modificatLa'){va=va||'';vb=vb||'';}
    if(va<vb)return -sortDir;if(va>vb)return sortDir;
    // criteriu secundar stabil: numele obiectului
    return a.obiect<b.obiect?-1:a.obiect>b.obiect?1:0;
  });
  ['obiect','baza','tip','scor','rezolvat','deployat','modificatLa'].forEach(function(c){
    var el=document.getElementById('sa-'+c);
    if(el)el.textContent=sortCol===c?(sortDir>0?'▲':'▼'):'';
  });
  _vizLast={s:-1,e:-1,total:-1};   // setul de randuri s-a schimbat -> forteaza redraw
  if(!pastreazaPozitia) document.getElementById('tw').scrollTop=0;
  _renderViz();   // lista se randeaza mereu, indiferent ce se intampla mai jos cu scroll-ul
  if(pastreazaPozitia && DET_ID!=null){
    // revenim din detaliu: incercam sa centram scroll-ul pe randul ultimei proceduri deschise.
    // Facut dupa un frame + cu try/catch, ca o eventuala problema aici (ex. container inca
    // nemasurat corect chiar dupa comutarea ecranului) sa nu poata impiedica afisarea listei.
    try{
      var idx=_rows.findIndex(function(d){return d.id===DET_ID;});
      if(idx!==-1){
        requestAnimationFrame(function(){
          try{
            var tw=document.getElementById('tw');
            var tinta=Math.max(0,(idx*ROW_H)-Math.floor((tw.clientHeight||500)/2)+Math.floor(ROW_H/2));
            tw.scrollTop=tinta;
            _renderViz();
          }catch(e){}
        });
      }
    }catch(e){}
  }
  var n=LISTA.filter(function(d){return d.rezolvat && d.cat!=='extra';}).length;
  document.getElementById('cnt').textContent=_rows.length+' obiecte (din '+LISTA.length+')';
  var sbi=document.getElementById('statBif');
  if(sbi)sbi.innerHTML=n>0?'<i class="ti ti-circle-check-filled" style="font-size:14px;vertical-align:-2px;margin-right:3px" aria-hidden="true"></i>'+n+' cu concluzie':'';
  document.getElementById('depTxt').textContent='Deployer ('+n+')';
  document.getElementById('btnDep').disabled=(n===0);
  actualizeazaProgresGlobal();
}
function _esc(s){return String(s==null?'':s).replace(/&/g,'&amp;').replace(/</g,'&lt;').replace(/>/g,'&gt;').replace(/"/g,'&quot;');}
var _vizLast={s:-1,e:-1,total:-1};
function _renderViz(){
  var tw=document.getElementById('tw');
  var total=_rows.length;
  var st=tw.scrollTop,vh=tw.clientHeight||500;
  var s=Math.max(0,Math.floor(st/ROW_H)-BUF);
  var e=Math.min(total,Math.ceil((st+vh)/ROW_H)+BUF);
  // daca fereastra vizibila nu s-a schimbat, nu reface DOM-ul (evita saltul la scroll)
  if(s===_vizLast.s&&e===_vizLast.e&&total===_vizLast.total)return;
  _vizLast={s:s,e:e,total:total};
  document.getElementById('sp-sus').style.height=(s*ROW_H)+'px';
  document.getElementById('sp-jos').style.height=(Math.max(0,total-e)*ROW_H)+'px';
  var h='';
  for(var i=s;i<e;i++){
    var d=_rows[i];
    var col=d.scor===null?'var(--coral)':d.scor<60?'var(--coral)':d.scor<90?'var(--amber)':'var(--neon)';
    var sc;
    if(d.cat==='extra')sc='<span style="color:var(--amber);font-weight:700;font-size:10.5px;display:flex;align-items:center;gap:4px;overflow:hidden;white-space:nowrap;min-width:0"><i class="ti ti-plus" style="font-size:13px;flex-shrink:0" aria-hidden="true"></i><span style="overflow:hidden;text-overflow:ellipsis">EXTRA</span></span>';
    else if(d.scor===null)sc='<span style="color:var(--coral);font-weight:700;font-size:10.5px;display:flex;align-items:center;gap:4px;overflow:hidden;white-space:nowrap;min-width:0"><i class="ti ti-circle-x" style="font-size:13px;flex-shrink:0" aria-hidden="true"></i><span style="overflow:hidden;text-overflow:ellipsis">LIPSESTE</span></span>';
    else sc='<div class="scbar"><div class="scbg"><div class="scf" style="width:'+Math.max(2,d.scor)+'%;background:'+col+'"></div></div><span class="scv" style="color:'+col+'">'+d.scor.toFixed(1)+'%</span></div>';
    var stareCls=d.omis?'omis':(d.rezolvat?'on':'off');
    var stareTitle=d.omis?'Omis (analizat, exclus din deploy)':(d.rezolvat?'Rezolvat':'Nerezolvat');
    var stare='<span class="dot-rez '+stareCls+'" title="'+stareTitle+'"></span>';
    var modCell=d.modificatLa
      ?'<span class="mod-chip mod-are" title="Ultima modificare (rezolvat/editat): '+new Date(d.modificatLa).toLocaleString('ro-RO')+'"><i class="ti ti-device-floppy" aria-hidden="true"></i>'+new Date(d.modificatLa).toLocaleString('ro-RO',{day:'2-digit',month:'2-digit',year:'2-digit',hour:'2-digit',minute:'2-digit'})+'</span>'
      :'<span class="mod-chip">Neactualizat</span>';
    var depCell;
    if(d.cat==='extra'){
      depCell='<div class="dep-cell"><span class="dep-chip dep-nu" title="Obiectul exista doar in baza tinta, nu si in referinta - nu poate fi generat un script de deploy pentru el"><i class="ti ti-circle-minus" aria-hidden="true"></i>Extra — nu se deployeaza</span></div>';
    } else if(d.deployat){
      var editatDupa=d.modificatLa && new Date(d.modificatLa)>new Date(d.deployat);
      if(editatDupa){
        depCell='<div class="dep-cell"><span class="dep-chip dep-stale" title="A fost trimis la deploy pe '+new Date(d.deployat).toLocaleString('ro-RO')+', dar a fost modificat din nou dupa aceea. S-ar putea sa fie nevoie de un nou deploy."><i class="ti ti-alert-triangle" aria-hidden="true"></i>Versiune noua fata de deploy</span></div>';
      } else {
        depCell='<div class="dep-cell"><span class="dep-chip dep-da" title="Trimis la deploy pe '+new Date(d.deployat).toLocaleString('ro-RO')+'"><i class="ti ti-package-export" aria-hidden="true"></i>'+new Date(d.deployat).toLocaleString('ro-RO',{day:'2-digit',month:'2-digit',year:'2-digit',hour:'2-digit',minute:'2-digit'})+'</span></div>';
      }
    } else if(d.rezolvat){
      depCell='<div class="dep-cell"><span class="dep-chip dep-ready" title="Marcat ca rezolvat, dar inca netrimis la deploy"><i class="ti ti-circle-dashed" aria-hidden="true"></i>Pregatit pentru deploy</span></div>';
    } else if(d.omis){
      depCell='<div class="dep-cell"><span class="dep-chip dep-omis" title="Analizat si marcat ca omis - nu va fi introdus in deploy"><i class="ti ti-circle-minus" aria-hidden="true"></i>Omis - nu se deployeaza</span></div>';
    } else {
      depCell='<div class="dep-cell"><span class="dep-chip dep-nu">Nu a fost introdus in Deploy</span></div>';
    }
    if(d.sesiuneInvechita){
      depCell+='<div class="dep-cell"><span class="dep-chip dep-stale" title="Acest obiect s-a modificat (referinta sau baza tinta) de la ultima sesiune salvata. Avea inainte statusul: '+(d.sesiuneInvechita.deployat?'deployat':'rezolvat')+'. Progresul vechi NU a fost reaplicat automat - verifica diff-ul din nou, s-ar putea sa fie nevoie de un nou deploy."><i class="ti ti-alert-triangle" aria-hidden="true"></i>Modificat de atunci</span></div>';
    }
    var tipIcon={'P':'ti-brand-databricks','V':'ti-eye','FN':'ti-math-function','IF':'ti-math-function','TF':'ti-math-function','TR':'ti-bolt'}[d.tip]||'ti-function';
    h+='<div class="lrow'+(d.id===DET_ID?' activ':'')+'" onclick="deschide('+d.id+')">'
      +(d.id===DET_ID?'<div class="activ-badge" title="Ultima procedura vizualizata"><i class="ti ti-eye" aria-hidden="true"></i></div>':'')
      +'<div class="ld lidx">'+(i+1)+'</div>'
      +'<div class="ld"><i class="ti '+tipIcon+'" style="color:'+col+';font-size:13px;flex-shrink:0" aria-hidden="true"></i><span class="onm" title="'+_esc(d.obiect)+'">'+_esc(d.obiect)+'</span></div>'
      +'<div class="ld"><span class="baza-nm" title="'+_esc(d.baza)+'">'+_esc(d.baza)+'</span></div>'
      +'<div class="ld"><span class="tip-chip">'+(TIP_NUME[d.tip]||_esc(d.tip)||'?')+'</span></div>'
      +'<div class="ld">'+sc+'</div>'
      +'<div class="ld" style="font-size:11px;color:var(--muted);overflow:hidden;text-overflow:ellipsis;white-space:nowrap" title="'+_esc(d.rezumat)+'">'+_esc(d.rezumat)+'</div>'
      +'<div class="ld" style="justify-content:center">'+stare+'</div>'
      +'<div class="ld">'+modCell+'</div>'
      +'<div class="ld">'+depCell+'</div>'
      +'<div class="ld"><button class="bta" onclick="event.stopPropagation();deschide('+d.id+')"><i class="ti ti-eye" aria-hidden="true"></i>Diff</button></div>'
      +'</div>';
  }
  document.getElementById('rows').innerHTML=h;
}
var _rafId=null,_scrollBound=false;
function initVirtualScroll(){
  _vizLast={s:-1,e:-1,total:-1};   // forteaza un render la urmatoarea rulare
  if(_scrollBound)return;          // ataseaza listener-ul O SINGURA DATA (nu stivui)
  _scrollBound=true;
  var tw=document.getElementById('tw');
  tw.addEventListener('scroll',function(){
    if(_rafId)return;              // un singur render pe frame; nu anula/reprograma
    _rafId=requestAnimationFrame(function(){_rafId=null;_renderViz();});
  },{passive:true});
}

// ── MOTOR DIFF (identic cu motor.py, ruleaza local pt editare live) ───────────
"use strict";

/* ==========================================================================
   MOTORUL DE COMPARARE — port FIDEL din motor.py.
   Include o reimplementare a lui difflib.SequenceMatcher (Ratcliff-Obershelp,
   autojunk=False), NU un LCS aproximativ — ca alinierea sa fie IDENTICA cu Python.

   Corespondenta 1:1: _norm, normalizeaza, pregateste_afisare, scor_similaritate,
   _split_statements, _cheie_bloc, rezumat_rapid, _diff_intra, diff_aliniat,
   rezumat_diff, diff_to_elemente
   ========================================================================== */

/* ---- difflib.SequenceMatcher (fidel, autojunk=False) ---- */
function SequenceMatcher(a, b){
  this.a = a; this.b = b;
  this.b2j = {};
  for (var i = 0; i < b.length; i++){
    var key = typeof b[i] + '\u0000' + b[i];
    if (!this.b2j[key]) this.b2j[key] = [];
    this.b2j[key].push(i);
  }
}
SequenceMatcher.prototype._idx = function(el){
  return this.b2j[typeof el + '\u0000' + el] || [];
};
SequenceMatcher.prototype.find_longest_match = function(alo, ahi, blo, bhi){
  var a=this.a, b=this.b;
  var besti=alo, bestj=blo, bestsize=0;
  var j2len={};
  for (var i=alo; i<ahi; i++){
    var newj2len={};
    var idxs=this._idx(a[i]);
    for (var x=0;x<idxs.length;x++){
      var j=idxs[x];
      if (j<blo) continue;
      if (j>=bhi) break;
      var k=(j2len[j-1]||0)+1;
      newj2len[j]=k;
      if (k>bestsize){ besti=i-k+1; bestj=j-k+1; bestsize=k; }
    }
    j2len=newj2len;
  }
  while (besti>alo && bestj>blo && a[besti-1]===b[bestj-1]){ besti--;bestj--;bestsize++; }
  while (besti+bestsize<ahi && bestj+bestsize<bhi && a[besti+bestsize]===b[bestj+bestsize]){ bestsize++; }
  return [besti,bestj,bestsize];
};
SequenceMatcher.prototype.get_matching_blocks = function(){
  if (this._mb) return this._mb;
  var la=this.a.length, lb=this.b.length;
  var queue=[[0,la,0,lb]], matching=[];
  while (queue.length){
    var q=queue.pop();
    var m=this.find_longest_match(q[0],q[1],q[2],q[3]);
    var i=m[0],j=m[1],k=m[2];
    if (k){
      matching.push([i,j,k]);
      if (q[0]<i && q[2]<j) queue.push([q[0],i,q[2],j]);
      if (i+k<q[1] && j+k<q[3]) queue.push([i+k,q[1],j+k,q[3]]);
    }
  }
  matching.sort(function(x,y){return x[0]-y[0]||x[1]-y[1];});
  var i1=0,j1=0,k1=0, out=[];
  for (var t=0;t<matching.length;t++){
    var i2=matching[t][0],j2=matching[t][1],k2=matching[t][2];
    if (i1+k1===i2 && j1+k1===j2){ k1+=k2; }
    else { if (k1) out.push([i1,j1,k1]); i1=i2;j1=j2;k1=k2; }
  }
  if (k1) out.push([i1,j1,k1]);
  out.push([la,lb,0]);
  this._mb=out; return out;
};
SequenceMatcher.prototype.get_opcodes = function(){
  if (this._op) return this._op;
  var i=0,j=0,ans=[], mb=this.get_matching_blocks();
  for (var t=0;t<mb.length;t++){
    var ai=mb[t][0],bj=mb[t][1],size=mb[t][2], tag='';
    if (i<ai && j<bj) tag='replace';
    else if (i<ai) tag='delete';
    else if (j<bj) tag='insert';
    if (tag) ans.push([tag,i,ai,j,bj]);
    i=ai+size; j=bj+size;
    if (size) ans.push(['equal',ai,i,bj,j]);
  }
  this._op=ans; return ans;
};
SequenceMatcher.prototype.ratio = function(){
  var mb=this.get_matching_blocks(), matches=0;
  for (var t=0;t<mb.length;t++) matches+=mb[t][2];
  var len=this.a.length+this.b.length;
  return len ? 2.0*matches/len : 1.0;
};

/* ---- _norm ---- */
var _HINTS = "NOLOCK|READPAST|ROWLOCK|TABLOCK|TABLOCKX|UPDLOCK|XLOCK|HOLDLOCK|"
           + "READUNCOMMITTED|READCOMMITTED|REPEATABLEREAD|SERIALIZABLE|PAGLOCK|NOWAIT";
function _norm(s){
  if (!s) return '';
  s = s.replace(/\/\*[\s\S]*?\*\//g, ' ');
  s = s.replace(/--[^\n]*/g, ' ');
  s = s.replace(new RegExp("\\bWITH\\s*\\(\\s*(?:"+_HINTS+")(?:\\s*,\\s*(?:"+_HINTS+"))*\\s*\\)","gi"), ' ');
  s = s.replace(new RegExp("\\b(?:"+_HINTS+")\\b","gi"), ' ');
  s = s.replace(/\[dbo\]\./gi, '');
  s = s.replace(/\bdbo\./gi, '');
  s = s.replace(/\bCOLLATE\s+database_default\b/gi, '');
  s = s.replace(/\[([^\[\]]+)\]/g, '$1');
  s = s.replace(/\s+/g, ' ');
  // dupa eliminarea hint-urilor de lock / COLLATE database_default poate ramane
  // un spatiu orfan inainte de virgula/paranteza inchisa (ex: "char(24) ,") -
  // identic cu fix-ul din motor.py, ca cele doua motoare (Python si JS) sa
  // dea mereu acelasi rezultat.
  s = s.replace(/\s+([,;)])/g, '$1');
  return s.trim().toLowerCase();
}
function normalizeaza(cod){ if (!cod) return ''; return _norm(cod.replace(/\r\n/g,'\n').replace(/\r/g,'\n')); }
function pregateste_afisare(cod){
  if (cod == null) return '';
  cod = cod.replace(/\r\n/g,'\n').replace(/\r/g,'\n');
  var linii = cod.split('\n').map(function(l){return l.replace(/\s+$/,'');});
  while (linii.length && !linii[0].trim()) linii.shift();
  while (linii.length && !linii[linii.length-1].trim()) linii.pop();
  return linii.join('\n');
}
// round-half-to-even (banker's rounding) — identic cu round() din Python.
// JS Math.round face half-up, deci 98.125 -> 98.13 in JS dar 98.12 in Python.
// Aceasta versiune reproduce fidel comportamentul Python.
function _round_py(x, ndigits){
  var m = Math.pow(10, ndigits);
  var v = x * m;
  var f = Math.floor(v);
  var diff = v - f;
  var r;
  // toleranta pt erori de virgula mobila la exact .5
  if (Math.abs(diff - 0.5) < 1e-9){
    r = (f % 2 === 0) ? f : f + 1;      // half -> spre par
  } else {
    r = Math.round(v);
  }
  return r / m;
}
function scor_similaritate(ref, cur){
  var sm=new SequenceMatcher(normalizeaza(ref).split(' '), normalizeaza(cur).split(' '));
  return _round_py(sm.ratio()*100, 2);
}

/* ---- _split_statements ---- */
var _STMT_START = new RegExp(
  "^\\s*(?:SELECT\\b|INSERT\\b|UPDATE\\b|DELETE\\b|MERGE\\b|SET\\s+@|EXEC(?:UTE)?\\b|"
  +"IF\\b|ELSE\\b|WHILE\\b|RETURN\\b|PRINT\\b|RAISERROR\\b|THROW\\b|OPEN\\b|CLOSE\\b|"
  +"FETCH\\b|DEALLOCATE\\b|TRUNCATE\\b|DROP\\b|GRANT\\b|DENY\\b|REVOKE\\b|UNION\\b|"
  +"CREATE\\b|DECLARE\\b)", "i");

var _splitCache={};   // memo: acelasi text nu se re-segmenteaza (ca _split_cached din Python)
function _split_statements(cod){
  if (!cod) return [];
  if (_splitCache[cod]) return _splitCache[cod];
  var rez=_split_statements_impl(cod);
  // limiteaza cache-ul ca sa nu creasca nelimitat in editare live
  if (Object.keys(_splitCache).length>200) _splitCache={};
  _splitCache[cod]=rez;
  return rez;
}
// Calculeaza linia REALA (0-based, in textul original) unde incepe fiecare bloc.
// NU se poate obtine prin insumarea .split('\n').length pe blocuri (asa cum se
// facea inainte) - _split_statements_impl arunca liniile goale folosite ca
// separator intre instructiuni, deci suma subestimeaza pozitia reala cu cate o
// linie pentru fiecare gol intalnit pana acolo. Cautam in schimb blocul, ca
// substring literal, direct in textul original.
function _liniiStartBlocuri(cod, blocuri){
  var out=[], pos=0;
  for(var k=0;k<blocuri.length;k++){
    var idx=cod.indexOf(blocuri[k], pos);
    if(idx===-1){ out.push(out.length?out[out.length-1]:0); continue; }
    out.push(cod.slice(0, idx).split('\n').length-1);
    pos=idx+blocuri[k].length;
  }
  return out;
}
function _lineScan(line, st){
  // elimina string-uri si comentarii (spatii), calculeaza parenDelta, pt scanare keyword-uri sigura
  var i=0, n=line.length, out='', delta=0;
  while(i<n){
    if(st.inBlockComment){
      var end=line.indexOf('*/',i);
      if(end===-1){ out+=new Array(n-i+1).join(' '); i=n; break; }
      out+=new Array(end+2-i+1).join(' '); i=end+2; st.inBlockComment=false; continue;
    }
    if(st.inStr){
      var qi=line.indexOf("'",i);
      if(qi===-1){ out+=new Array(n-i+1).join(' '); i=n; break; }
      if(line[qi+1]==="'"){ out+='  '; i=qi+2; continue; }
      st.inStr=false; out+=new Array(qi+1-i+1).join(' '); i=qi+1; continue;
    }
    var c=line[i];
    if(c==='-'&&line[i+1]==='-'){ out+=new Array(n-i+1).join(' '); i=n; break; }
    if(c==='/'&&line[i+1]==='*'){ st.inBlockComment=true; out+='  '; i+=2; continue; }
    if(c==="'"){ st.inStr=true; out+=' '; i+=1; continue; }
    if(c==='('){ delta++; out+=c; i++; continue; }
    if(c===')'){ delta--; out+=c; i++; continue; }
    out+=c; i++;
  }
  return {clean:out, delta:delta};
}
var _KWSTACK_RE=/\b(case|begin|end)\b/gi;
function _updateCaseStack(cleanLine, stack){
  var m; _KWSTACK_RE.lastIndex=0;
  while((m=_KWSTACK_RE.exec(cleanLine))!==null){
    var kw=m[1].toLowerCase();
    if(kw==='case'||kw==='begin') stack.push(kw);
    else if(stack.length) stack.pop();
  }
}
function _blocEsteContinuat(curentJoin){
  var n=_norm(curentJoin);
  if(!n) return false;
  if(/\bcursor\b/.test(n) && /\bfor\s*$/.test(n)) return true;
  if(/^insert\b/.test(n) && n.indexOf('values')===-1 && !/;\s*$/.test(n)){
    // INSERT INTO x (cols) SELECT ... - ramane "continuat" doar cat timp SELECT-ul
    // propriu al acestui INSERT inca nu a aparut. Odata ce a aparut, urmatoarea
    // linie care porneste o noua instructiune (alt SELECT/INSERT/CREATE/etc) e
    // sigur o instructiune noua, nu o continuare - altfel INSERT-ul "inghite"
    // tot restul obiectului pana la capat (nu exista VALUES/; in acest dialect).
    return !/\bselect\b/.test(n);
  }
  return false;
}
var _unesteCursorActiv=false;
function _numeCursorDinBloc(blocText){
  var m=/\bdeclare\s+(\w+)\s+cursor\b/i.exec(blocText);
  return m?m[1].toLowerCase():null;
}
function _areCursorDetectabil(text){
  if(!text) return false;
  var blocs=_split_statements_impl(text);
  for(var i=0;i<blocs.length;i++){
    if(_numeCursorDinBloc(blocs[i])) return true;
  }
  return false;
}
function _unesteCicluriCursor(blocuri){
  var out=[], i=0;
  while(i<blocuri.length){
    var nume=_numeCursorDinBloc(blocuri[i]);
    if(nume){
      var j=i+1, gasitEnd=-1;
      for(; j<blocuri.length; j++){
        if(new RegExp('\\bdeallocate\\s+'+nume+'\\b').test(_norm(blocuri[j]))){ gasitEnd=j; break; }
      }
      if(gasitEnd!==-1){ out.push(blocuri.slice(i,gasitEnd+1).join('\n')); i=gasitEnd+1; continue; }
    }
    out.push(blocuri[i]); i++;
  }
  return out;
}
function toggleUnesteCursor(){
  _unesteCursorActiv=!_unesteCursorActiv;
  _splitCache={};
  document.getElementById('btnUnesteCursor').classList.toggle('grn',_unesteCursorActiv);
  document.getElementById('unesteCursorTxt').textContent=_unesteCursorActiv?'Desparte ciclul cursor':'Unește ciclul cursor';
  resyncDiff();
}
function _adancimeBeginEnd(text){
  var n=_norm(text);
  var opens=(n.match(/\bbegin\b(?!\s*tran(?:saction)?\b)/g)||[]).length;
  var closes=(n.match(/\bend\b/g)||[]).length;
  return opens-closes;
}
function _incepeCuIfWhile(text){
  var s=(text||'').replace(/^\s+/,'');
  return /^(if|while)\b/i.test(s);
}
function _unesteBlocuriCompuse(blocuri){
  var out=[], i=0;
  while(i<blocuri.length){
    var d=_adancimeBeginEnd(blocuri[i]);
    if(d>0 && _incepeCuIfWhile(blocuri[i])){
      var depth=d, j=i+1;
      while(j<blocuri.length && depth>0){ depth+=_adancimeBeginEnd(blocuri[j]); j++; }
      if(depth<=0){ out.push(blocuri.slice(i,j).join('\n')); i=j; continue; }
      // nu s-a inchis corect (structura malformata) - nu fortam merge pana la capat
    }
    out.push(blocuri[i]); i++;
  }
  return out;
}
function _split_statements_impl(cod){
  if (!cod) return [];
  var linii=cod.split('\n'), blocuri=[], curent=[], in_header=true, prev_strip='';
  var pstate={inStr:false, inBlockComment:false}, parenDepth=0, caseStack=[];
  for (var idx=0; idx<linii.length; idx++){
    var linie=linii[idx], s=linie.trim();
    var insideCase = caseStack.indexOf('case')!==-1;
    var insideConstruct = parenDepth>0 || pstate.inStr || pstate.inBlockComment || insideCase;
    var scan=_lineScan(linie, pstate);
    parenDepth+=scan.delta; if(parenDepth<0)parenDepth=0;
    if (!s){
      if (insideConstruct){ curent.push(linie); prev_strip=''; _updateCaseStack(scan.clean, caseStack); continue; }
      if (!in_header && curent.length){ blocuri.push(curent.join('\n')); curent=[]; }
      else if (in_header && curent.length){ curent.push(linie); }
      prev_strip=''; _updateCaseStack(scan.clean, caseStack); continue;
    }
    var corp_control = /^(if|else|while)\b/i.test(prev_strip);
    var continuat = insideConstruct || _blocEsteContinuat(curent.join('\n'));
    if (_STMT_START.test(s) && !corp_control && !continuat){
      if (in_header) in_header=false;
      if (curent.length) blocuri.push(curent.join('\n'));
      curent=[linie];
    } else { curent.push(linie); }
    prev_strip=s;
    _updateCaseStack(scan.clean, caseStack);
  }
  if (curent.length) blocuri.push(curent.join('\n'));

  if(_unesteCursorActiv){
    blocuri=_unesteBlocuriCompuse(blocuri);
    blocuri=_unesteCicluriCursor(blocuri);
  }

  var rezultat=[], i=0;
  while (i<blocuri.length){
    var n=_norm(blocuri[i]);
    if (/\bunion(?:\s+all)?\s*$/.test(n) && i+1<blocuri.length){
      rezultat.push(blocuri[i]+'\n'+blocuri[i+1]); i+=2;
    } else { rezultat.push(blocuri[i]); i+=1; }
  }
  return rezultat.filter(function(b){ return _norm(b); });
}

/* ---- MD5 (pentru _cheie_bloc) ---- */
function md5(str){
  function rl(n,c){return (n<<c)|(n>>>(32-c));}
  function ac(q,a,b,x,s,t){a=(a+q+x+t)|0;return (rl(a,s)+b)|0;}
  function ff(a,b,c,d,x,s,t){return ac((b&c)|(~b&d),a,b,x,s,t);}
  function gg(a,b,c,d,x,s,t){return ac((b&d)|(c&~d),a,b,x,s,t);}
  function hh(a,b,c,d,x,s,t){return ac(b^c^d,a,b,x,s,t);}
  function ii(a,b,c,d,x,s,t){return ac(c^(b|~d),a,b,x,s,t);}
  var utf8=unescape(encodeURIComponent(str)), bytes=[];
  for (var i=0;i<utf8.length;i++) bytes.push(utf8.charCodeAt(i));
  var n=bytes.length, words=[];
  for (var i=0;i<n;i++) words[i>>2]|=bytes[i]<<((i%4)*8);
  words[n>>2]|=0x80<<((n%4)*8);
  words[(((n+8)>>6)+1)*16-2]=n*8;
  var a=1732584193,b=-271733879,c=-1732584194,d=271733878;
  for (var i=0;i<words.length;i+=16){
    var oa=a,ob=b,oc=c,od=d;
    a=ff(a,b,c,d,words[i+0]||0,7,-680876936);d=ff(d,a,b,c,words[i+1]||0,12,-389564586);
    c=ff(c,d,a,b,words[i+2]||0,17,606105819);b=ff(b,c,d,a,words[i+3]||0,22,-1044525330);
    a=ff(a,b,c,d,words[i+4]||0,7,-176418897);d=ff(d,a,b,c,words[i+5]||0,12,1200080426);
    c=ff(c,d,a,b,words[i+6]||0,17,-1473231341);b=ff(b,c,d,a,words[i+7]||0,22,-45705983);
    a=ff(a,b,c,d,words[i+8]||0,7,1770035416);d=ff(d,a,b,c,words[i+9]||0,12,-1958414417);
    c=ff(c,d,a,b,words[i+10]||0,17,-42063);b=ff(b,c,d,a,words[i+11]||0,22,-1990404162);
    a=ff(a,b,c,d,words[i+12]||0,7,1804603682);d=ff(d,a,b,c,words[i+13]||0,12,-40341101);
    c=ff(c,d,a,b,words[i+14]||0,17,-1502002290);b=ff(b,c,d,a,words[i+15]||0,22,1236535329);
    a=gg(a,b,c,d,words[i+1]||0,5,-165796510);d=gg(d,a,b,c,words[i+6]||0,9,-1069501632);
    c=gg(c,d,a,b,words[i+11]||0,14,643717713);b=gg(b,c,d,a,words[i+0]||0,20,-373897302);
    a=gg(a,b,c,d,words[i+5]||0,5,-701558691);d=gg(d,a,b,c,words[i+10]||0,9,38016083);
    c=gg(c,d,a,b,words[i+15]||0,14,-660478335);b=gg(b,c,d,a,words[i+4]||0,20,-405537848);
    a=gg(a,b,c,d,words[i+9]||0,5,568446438);d=gg(d,a,b,c,words[i+14]||0,9,-1019803690);
    c=gg(c,d,a,b,words[i+3]||0,14,-187363961);b=gg(b,c,d,a,words[i+8]||0,20,1163531501);
    a=gg(a,b,c,d,words[i+13]||0,5,-1444681467);d=gg(d,a,b,c,words[i+2]||0,9,-51403784);
    c=gg(c,d,a,b,words[i+7]||0,14,1735328473);b=gg(b,c,d,a,words[i+12]||0,20,-1926607734);
    a=hh(a,b,c,d,words[i+5]||0,4,-378558);d=hh(d,a,b,c,words[i+8]||0,11,-2022574463);
    c=hh(c,d,a,b,words[i+11]||0,16,1839030562);b=hh(b,c,d,a,words[i+14]||0,23,-35309556);
    a=hh(a,b,c,d,words[i+1]||0,4,-1530992060);d=hh(d,a,b,c,words[i+4]||0,11,1272893353);
    c=hh(c,d,a,b,words[i+7]||0,16,-155497632);b=hh(b,c,d,a,words[i+10]||0,23,-1094730640);
    a=hh(a,b,c,d,words[i+13]||0,4,681279174);d=hh(d,a,b,c,words[i+0]||0,11,-358537222);
    c=hh(c,d,a,b,words[i+3]||0,16,-722521979);b=hh(b,c,d,a,words[i+6]||0,23,76029189);
    a=hh(a,b,c,d,words[i+9]||0,4,-640364487);d=hh(d,a,b,c,words[i+12]||0,11,-421815835);
    c=hh(c,d,a,b,words[i+15]||0,16,530742520);b=hh(b,c,d,a,words[i+2]||0,23,-995338651);
    a=ii(a,b,c,d,words[i+0]||0,6,-198630844);d=ii(d,a,b,c,words[i+7]||0,10,1126891415);
    c=ii(c,d,a,b,words[i+14]||0,15,-1416354905);b=ii(b,c,d,a,words[i+5]||0,21,-57434055);
    a=ii(a,b,c,d,words[i+12]||0,6,1700485571);d=ii(d,a,b,c,words[i+3]||0,10,-1894986606);
    c=ii(c,d,a,b,words[i+10]||0,15,-1051523);b=ii(b,c,d,a,words[i+1]||0,21,-2054922799);
    a=ii(a,b,c,d,words[i+8]||0,6,1873313359);d=ii(d,a,b,c,words[i+15]||0,10,-30611744);
    c=ii(c,d,a,b,words[i+6]||0,15,-1560198380);b=ii(b,c,d,a,words[i+13]||0,21,1309151649);
    a=ii(a,b,c,d,words[i+4]||0,6,-145523070);d=ii(d,a,b,c,words[i+11]||0,10,-1120210379);
    c=ii(c,d,a,b,words[i+2]||0,15,718787259);b=ii(b,c,d,a,words[i+9]||0,21,-343485551);
    a=(a+oa)|0;b=(b+ob)|0;c=(c+oc)|0;d=(d+od)|0;
  }
  function hex(x){var s='';for(var i=0;i<4;i++)s+=('0'+((x>>(i*8))&255).toString(16)).slice(-2);return s;}
  return hex(a)+hex(b)+hex(c)+hex(d);
}

/* ---- _cheie_bloc ---- */
function _cheie_bloc(s){
  var n=_norm(s), tip='other';
  var kws=['select','insert','update','delete','set','exec','if','while','return','create','begin','end','declare','union'];
  for (var i=0;i<kws.length;i++){ if (n.indexOf(kws[i])===0){ tip=kws[i]; break; } }
  return tip+':'+n.slice(0,40)+':'+md5(n).slice(0,8);
}

/* ---- rezumat_rapid ---- */
function rezumat_rapid(text_ref, text_cur){
  var sv=_split_statements(text_ref||''), sn=_split_statements(text_cur||'');
  var kv=sv.map(_cheie_bloc), kn=sn.map(_cheie_bloc);
  var ops=new SequenceMatcher(kv,kn).get_opcodes();
  var mod=0,add=0,st=0;
  ops.forEach(function(o){
    var tag=o[0],i1=o[1],i2=o[2],j1=o[3],j2=o[4];
    if (tag==='replace'){
      var comun=Math.min(i2-i1,j2-j1);
      for (var k=0;k<comun;k++){ if (_norm(sv[i1+k])!==_norm(sn[j1+k])) mod++; }
      st+=(i2-i1)-comun; add+=(j2-j1)-comun;
    } else if (tag==='delete'){ st+=i2-i1; }
    else if (tag==='insert'){ add+=j2-j1; }
  });
  var p=[];
  if (mod) p.push(mod+' modificate');
  if (add) p.push(add+' adaugate');
  if (st)  p.push(st+' sterse');
  return p.length ? p.join(', ') : 'diferente minore';
}

/* ---- segmente cosmetice (ignorate la evidentiere, desi difera caracter-cu-caracter) ----
   Aceeasi problema ca in motor.py: cand blocul e "modificat" dintr-un motiv real
   aflat oriunde in acelasi bloc unit, diff-ul pe token-uri reda ca "diferente"
   si parantezele [dbo].[...] / COLLATE database_default / hint-urile de lock,
   desi userul le vede deja ca zgomot (sunt ignorate la clasificarea egal/modificat). */
var _COSMETIC_RE = new RegExp(
  "^(?:\\[dbo\\]\\.?|dbo\\.|\\[|\\]|collate\\s+database_default|"
  +"with\\s*\\(\\s*(?:"+_HINTS+")(?:\\s*,\\s*(?:"+_HINTS+"))*\\s*\\)|(?:"+_HINTS+")|\\s)+$", "i");
function _e_cosmetic(s){ return !!s && _COSMETIC_RE.test(s); }

/* ---- _diff_intra (pe CARACTERE) ---- */
function _diff_intra(a, b){
  var ca=_tokenize(a), cb=_tokenize(b), ops=new SequenceMatcher(ca,cb).get_opcodes(), sr=[],sc=[];
  ops.forEach(function(o){
    var tag=o[0], va=ca.slice(o[1],o[2]).join(''), vb=cb.slice(o[3],o[4]).join('');
    var schimbat = tag!=='equal';
    if(schimbat && (va===''||_e_cosmetic(va)) && (vb===''||_e_cosmetic(vb))){
      schimbat=false;   // diferenta e doar cosmetica (paranteze/dbo./COLLATE/hint) - nu o evidentiem
    }
    if (va) sr.push([va, schimbat?'del':null]);
    if (vb) sc.push([vb, schimbat?'add':null]);
  });
  return [sr,sc];
}

/* ---- diff_aliniat ---- */
/* imperechere pe similaritate pentru segmente 'replace' (aliniere DP, fara
   incrucisari). Blocurile prea diferite raman sters+adaugat, nu perechi fortate. */
function _perechiReplace(bv, bn){
  var n=bv.length, m=bn.length, i, j;
  if(n*m>144){ // segment mare: pastram imperecherea pozitionala (comportamentul vechi)
    var out=[], c=Math.min(n,m);
    for(i=0;i<c;i++)out.push([i,i]);
    return out;
  }
  var S=[];
  for(i=0;i<n;i++){ S.push([]); for(j=0;j<m;j++) S[i].push(scor_similaritate(bv[i],bn[j])); }
  var PRAG=44;
  var D=[], P=[];
  for(i=0;i<=n;i++){ D.push(new Array(m+1).fill(0)); P.push(new Array(m+1).fill(0)); }
  for(i=1;i<=n;i++)for(j=1;j<=m;j++){
    var diag=D[i-1][j-1]+(S[i-1][j-1]>=PRAG?S[i-1][j-1]:0);
    var sus=D[i-1][j], stg=D[i][j-1];
    if(diag>=sus&&diag>=stg){D[i][j]=diag;P[i][j]=1;}
    else if(sus>=stg){D[i][j]=sus;P[i][j]=2;}
    else{D[i][j]=stg;P[i][j]=3;}
  }
  var pairs=[]; i=n; j=m;
  while(i>0&&j>0){
    if(P[i][j]===1){ if(S[i-1][j-1]>=PRAG)pairs.push([i-1,j-1]); i--; j--; }
    else if(P[i][j]===2)i--; else j--;
  }
  pairs.reverse();
  return pairs;
}

function diff_aliniat(text_ref, text_cur){
  // `i` e ID STABIL, nu index secvential recalculat: pt intrarile ancorate in
  // sursa (egal/sters/modificat) foloseste pozitia absoluta in sv (sursa e
  // fixa intr-o sesiune, deci acelasi bloc primeste mereu acelasi i, chiar
  // daca recalculam diff-ul dupa fiecare editare a tintei). Intrarile
  // doar-adaugate (fara ancora in sursa) primesc id-uri negative, unice pe
  // randare, dintr-un namespace separat (nu se pot ciocni cu pozitiile din sv).
  var sv=_split_statements(text_ref||''), sn=_split_statements(text_cur||'');
  var kv=sv.map(_cheie_bloc), kn=sn.map(_cheie_bloc);
  var ops=new SequenceMatcher(kv,kn).get_opcodes();
  var randuri=[], insC=0;
  function nextIns(){ insC++; return -insC; }
  ops.forEach(function(o){
    var tag=o[0],i1=o[1],i2=o[2],j1=o[3],j2=o[4];
    if (tag==='equal'){
      for (var k=0;k<i2-i1;k++){ randuri.push({i:i1+k,tip:'egal',st:sv[i1+k],dr:sn[j1+k],ss:null,ds:null}); }
    } else if (tag==='replace'){
      var bv=sv.slice(i1,i2), bn=sn.slice(j1,j2);
      var pairs=_perechiReplace(bv,bn), pi=0, ii=0, jj=0;
      while(ii<bv.length || jj<bn.length){
        var pr=pairs[pi];
        if(pr && pr[0]===ii && pr[1]===jj){
          if(_norm(bv[ii])===_norm(bn[jj])){
            randuri.push({i:i1+ii,tip:'egal',st:bv[ii],dr:bn[jj],ss:null,ds:null});
          } else {
            var dd=_diff_intra(bv[ii],bn[jj]);
            randuri.push({i:i1+ii,tip:'modificat',st:bv[ii],dr:bn[jj],ss:dd[0],ds:dd[1]});
          }
          ii++; jj++; pi++; continue;
        }
        var nextI=pr?pr[0]:bv.length, nextJ=pr?pr[1]:bn.length;
        if(ii<nextI){ randuri.push({i:i1+ii,tip:'sters',st:bv[ii],dr:null,ss:null,ds:null}); ii++; continue; }
        if(jj<nextJ){ randuri.push({i:nextIns(),tip:'adaugat',st:null,dr:bn[jj],ss:null,ds:null}); jj++; continue; }
        break;
      }
    } else if (tag==='delete'){
      for (var k=i1;k<i2;k++){ randuri.push({i:k,tip:'sters',st:sv[k],dr:null,ss:null,ds:null}); }
    } else if (tag==='insert'){
      for (var k=j1;k<j2;k++){ randuri.push({i:nextIns(),tip:'adaugat',st:null,dr:sn[k],ss:null,ds:null}); }
    }
  });
  return randuri;
}

/* ---- rezumat_diff / diff_to_elemente ---- */
function rezumat_diff(r){
  var mod=r.filter(function(x){return x.tip==='modificat';}).length;
  var add=r.filter(function(x){return x.tip==='adaugat';}).length;
  var st =r.filter(function(x){return x.tip==='sters';}).length;
  var p=[];
  if (mod) p.push(mod+' modificate');
  if (add) p.push(add+' adaugate');
  if (st)  p.push(st+' sterse');
  return p.length ? p.join(', ') : 'diferente minore';
}
function diff_to_elemente(r){
  return r.map(function(x){return {i:x.i,tip:x.tip,st:x.st,dr:x.dr,ss:x.ss,ds:x.ds};});
}

// ── DETALIU (rescris: editare live in dreapta, stil Araxis) ───────────────────
// Model de memorie: `rezultatEditat` = textul complet al procedurii tinta,
// asa cum l-a lasat userul. `modificateManuale` ramane pt compatibilitate cu
// deployer-ul (cheie=index bloc -> text/'__EXCLUS__'), reconstruit din diff live.
var _srcText='', _debId=null;
var _modOffline=false;   // true cat timp editorul de diferente arata o comparare offline (copy/paste)

// ── COMPARARE OFFLINE (copy/paste, fara conexiune la server) ──────────────────
function deschideOfflineModal(){
  document.getElementById('stOffline').textContent='';
  document.getElementById('stOffline').className='stbar';
  document.getElementById('offOverlay').classList.add('on');
  setTimeout(function(){document.getElementById('offRef').focus();},50);
}
function inchideOfflineModal(){
  document.getElementById('offOverlay').classList.remove('on');
}
function _offLipeste(id){
  var ta=document.getElementById(id);
  if(navigator.clipboard && navigator.clipboard.readText){
    navigator.clipboard.readText().then(function(t){ ta.value=t; ta.focus(); })
      .catch(function(){ ta.focus(); });
  } else { ta.focus(); }
}
function _offSterge(id){
  var ta=document.getElementById(id); ta.value=''; ta.focus();
}
// Foloseste exact acelasi motor de diff (diff_aliniat / SequenceMatcher / etc.,
// reimplementat mai sus in JS) ca si compararea normala prin server - deci
// rezultatul (navigare, aplicare bloc cu bloc, side-by-side/inline, copiere)
// e identic din punct de vedere functional.
function comparaOffline(){
  var ref=document.getElementById('offRef').value;
  var cur=document.getElementById('offCur').value;
  var st=document.getElementById('stOffline');
  if(!ref.trim() && !cur.trim()){
    if(st){st.textContent='Lipeste cod in cel putin unul din cele doua campuri.';st.className='stbar err';}
    return;
  }
  ref=pregateste_afisare(ref); cur=pregateste_afisare(cur);
  if(st){st.textContent='';st.className='stbar';}
  inchideOfflineModal();
  if(typeof _incheiePlasare==='function')_incheiePlasare();
  DET_ID='__offline__'; EL=diff_to_elemente(diff_aliniat(ref,cur)); DIF=[]; nav=0; modificateManuale={}; editorSalvat=null;
  modDif=true; _selBloc=null; _blocuriAduse={}; _istoricPlasari=[];
  _ghid=null; _pvI=null; _tintaFocus=null; _modOffline=true;
  if(typeof _pozitioneazaGmbOverlay==='function')_pozitioneazaGmbOverlay();
  if(typeof _vedere!=='undefined' && _vedere==='inline')setVedere('side');
  var _bg=document.getElementById('btnGhid');
  if(_bg){_bg.classList.remove('grn');document.getElementById('ghidTxt').textContent='Mod ghidat';}
  var _po=document.getElementById('pvOverlay'); if(_po)_po.classList.remove('on');
  var _gd=document.getElementById('ghidDone'); if(_gd)_gd.classList.remove('on');
  var btnU0=document.getElementById('btnUndoPlasare'); if(btnU0)btnU0.style.display='none';
  document.getElementById('modIco').className='ti ti-eye-off';
  document.getElementById('modTxt').textContent='Ascunde diferentele';
  document.getElementById('btnModDif').classList.add('grn');
  // ascundem controale care nu au sens fara sesiune/lista (concluzie, navigare intre obiecte)
  var btnConcl=document.getElementById('btnConcl'); if(btnConcl)btnConcl.style.display='none';
  var bp=document.getElementById('btnObjPrev'), bn=document.getElementById('btnObjNext'), oi=document.getElementById('objNavI');
  if(bp)bp.style.display='none'; if(bn)bn.style.display='none'; if(oi)oi.style.display='none';
  document.getElementById('ec').classList.remove('active');
  document.getElementById('ed').classList.add('active');
  var bbkTxt=document.getElementById('bbkTxt'); if(bbkTxt)bbkTxt.textContent='Comparare offline';
  document.getElementById('detN').textContent='Comparare offline';
  document.getElementById('detM').innerHTML=scor_similaritate(ref,cur)+'% similar &bull; comparare locala, fara server';
  document.getElementById('hSrc').textContent='REFERINTA (lipita)';
  document.getElementById('hWork').textContent='TINTA (lipita)';
  document.getElementById('infMsg').textContent='Se pregateste...';
  document.getElementById('dSrc').innerHTML='';
  _srcText=ref;
  var ta=document.getElementById('editTA');
  ta.value=cur;
  _unesteCursorActiv=false; _splitCache={};
  var btnUC=document.getElementById('btnUnesteCursor');
  if(btnUC){
    var detectat=_areCursorDetectabil(_srcText) || _areCursorDetectabil(cur);
    btnUC.style.display=detectat?'':'none';
    btnUC.classList.remove('grn');
    document.getElementById('unesteCursorTxt').textContent='Unește ciclul cursor';
  }
  recalcDiff();
  renderSursa();
  actualizaNav();
  autoResize();
}

function deschide(id){
  var rec=LISTA.find(function(d){return d.id===id;});if(!rec)return;
  if(typeof _incheiePlasare==='function')_incheiePlasare();
  _modOffline=false;
  var _bc0=document.getElementById('btnConcl'); if(_bc0)_bc0.style.display='';
  var _bp0=document.getElementById('btnObjPrev'), _bn0=document.getElementById('btnObjNext'), _oi0=document.getElementById('objNavI');
  if(_bp0)_bp0.style.display=''; if(_bn0)_bn0.style.display=''; if(_oi0)_oi0.style.display='';
  DET_ID=id;EL=null;DIF=[];nav=0;modificateManuale={};editorSalvat=null;
  modDif=true;   // implicit ON - diferentele se vad de la inceput
  _selBloc=null;
  _blocuriAduse={};   // reset urmarire blocuri aduse din sursa
  _istoricPlasari=[];
  _ghid=null; _pvI=null; _tintaFocus=null;
  if(typeof _pozitioneazaGmbOverlay==='function')_pozitioneazaGmbOverlay();
  if(typeof _vedere!=='undefined' && _vedere==='inline')setVedere('side');
  var _bg=document.getElementById('btnGhid');
  if(_bg){_bg.classList.remove('grn');document.getElementById('ghidTxt').textContent='Mod ghidat';}
  var _po=document.getElementById('pvOverlay'); if(_po)_po.classList.remove('on');
  var _gd=document.getElementById('ghidDone'); if(_gd)_gd.classList.remove('on');
  var btnU0=document.getElementById('btnUndoPlasare');
  if(btnU0)btnU0.style.display='none';
  document.getElementById('modIco').className='ti ti-eye-off';
  document.getElementById('modTxt').textContent='Ascunde diferentele';
  document.getElementById('btnModDif').classList.add('grn');
  document.getElementById('el').classList.remove('active');
  document.getElementById('ed').classList.add('active');
  var bbkTxt0=document.getElementById('bbkTxt'); if(bbkTxt0)bbkTxt0.textContent='Lista';
  document.getElementById('detN').textContent=rec.obiect;
  var scTxt=rec.scor!==null?rec.scor.toFixed(2)+'%':(rec.cat==='extra'?'EXTRA — exista doar in aceasta baza':'LIPSESTE in aceasta baza');
  document.getElementById('detM').innerHTML='Baza: <b style="color:var(--teal)">'+_esc(rec.baza)+'</b> &bull; '+scTxt;
  document.getElementById('hSrc').textContent=META.referinta;
  document.getElementById('hWork').textContent=rec.baza;
  document.getElementById('btnConcl').classList.toggle('done',!!rec.rezolvat);
  var icD=document.getElementById('conclIco');
  icD.className=rec.rezolvat?'ti ti-circle-check-filled':'ti ti-circle-dashed';
  icD.removeAttribute('data-ico'); renderIcons(icD.parentNode);
  _sincronizeazaBtnOmite(!!rec.omis);
  document.getElementById('infMsg').textContent='Se incarca...';
  document.getElementById('dSrc').innerHTML='';
  document.getElementById('editTA').value='';
  document.getElementById('editHLInner').innerHTML='';
  pywebview.api.elemente(id).then(function(el_data){
    EL=el_data;
    // reconstruiesc textele integrale din elemente (blocuri)
    _srcText=EL.map(function(e){return e.st;}).filter(function(x){return x!=null;}).join('\n');
    // daca userul a mai editat obiectul (revenit din lista), refolosim ce a lasat
    var textTinta;
    if(rec.rezultatEditat!==undefined && rec.rezultatEditat!==null){
      textTinta=rec.rezultatEditat;
    } else {
      textTinta=EL.map(function(e){return e.dr;}).filter(function(x){return x!=null;}).join('\n');
    }
    var ta=document.getElementById('editTA');
    ta.value=textTinta;
    _unesteCursorActiv=false; _splitCache={};
    var btnUC=document.getElementById('btnUnesteCursor');
    if(btnUC){
      var detectat=_areCursorDetectabil(_srcText) || _areCursorDetectabil(textTinta);
      btnUC.style.display=detectat?'':'none';
      btnUC.classList.remove('grn');
      document.getElementById('unesteCursorTxt').textContent='Unește ciclul cursor';
    }
    recalcDiff();          // primul diff (calculeaza DIF_EL + linii + highlight dreapta)
    renderSursa();         // consuma DIF_EL -> panoul stanga
    actualizaNav();
    actualizeazaObjNav();
    autoResize();
  }).catch(function(e){
    document.getElementById('infMsg').textContent='Eroare la incarcarea obiectului: '+String(e);
  });
}
function inapoiLista(){
  if(_modOffline){
    _modOffline=false;
    var bbkTxt=document.getElementById('bbkTxt'); if(bbkTxt)bbkTxt.textContent='Lista';
    var btnConcl=document.getElementById('btnConcl'); if(btnConcl)btnConcl.style.display='';
    var bp=document.getElementById('btnObjPrev'), bn=document.getElementById('btnObjNext'), oi=document.getElementById('objNavI');
    if(bp)bp.style.display=''; if(bn)bn.style.display=''; if(oi)oi.style.display='';
    document.getElementById('ed').classList.remove('active');
    document.getElementById('ec').classList.add('active');
    return;
  }
  try{
    var rec=LISTA.find(function(d){return d.id===DET_ID;});
    if(rec){
      rec.rezolvat=document.getElementById('btnConcl').classList.contains('done');
      var _btnOm=document.getElementById('btnOmite');
      rec.omis=_btnOm?_btnOm.classList.contains('activ-omis'):false;
      rec.rezultatEditat=document.getElementById('editTA').value;  // pastram textul editat
      rec.adaugat=construiesteAdaugat();                            // pt deployer
      rec.modificatLa=new Date().toISOString();
      delete rec.sesiuneInvechita;   // a fost revizuit acum, avertismentul vechi nu mai e relevant
    }
  }catch(e){ console.error('inapoiLista: eroare la salvarea obiectului curent', e); }
  document.getElementById('ed').classList.remove('active');
  document.getElementById('el').classList.add('active');
  try{ renderLista(true); }catch(e){ console.error('inapoiLista: eroare la renderLista', e); renderLista(); }
  try{ salveazaSesiune(); }catch(e){ console.error('inapoiLista: eroare la salveazaSesiune', e); }
}
function _salveazaObiectCurent(){
  var rec=LISTA.find(function(d){return d.id===DET_ID;});
  if(rec){
    rec.rezolvat=document.getElementById('btnConcl').classList.contains('done');
    var _btnOm=document.getElementById('btnOmite');
    rec.omis=_btnOm?_btnOm.classList.contains('activ-omis'):false;
    rec.rezultatEditat=document.getElementById('editTA').value;
    rec.adaugat=construiesteAdaugat();
    rec.modificatLa=new Date().toISOString();
    delete rec.sesiuneInvechita;
  }
}
function navObiect(delta){
  if(!_rows||!_rows.length||DET_ID==null)return;
  var idx=_rows.findIndex(function(d){return d.id===DET_ID;});
  if(idx===-1)return;
  var nou=idx+delta;
  if(nou<0||nou>=_rows.length)return;
  _salveazaObiectCurent();
  salveazaSesiune();
  deschide(_rows[nou].id);
}
function actualizeazaObjNav(){
  var el=document.getElementById('objNavI');
  var prev=document.getElementById('btnObjPrev'), next=document.getElementById('btnObjNext');
  if(!el)return;
  if(!_rows||!_rows.length||DET_ID==null){ el.textContent='-'; if(prev)prev.disabled=true; if(next)next.disabled=true; return; }
  var idx=_rows.findIndex(function(d){return d.id===DET_ID;});
  if(idx===-1){ el.textContent='-'; if(prev)prev.disabled=true; if(next)next.disabled=true; return; }
  el.textContent=(idx+1)+'/'+_rows.length;
  if(prev)prev.disabled=(idx<=0);
  if(next)next.disabled=(idx>=_rows.length-1);
}
function marcheazaRezolvat(done){
  var btn=document.getElementById('btnConcl');
  if(btn.classList.contains('done')===done)return;
  btn.classList.toggle('done',done);
  var ic=document.getElementById('conclIco');
  ic.className=done?'ti ti-circle-check-filled':'ti ti-circle-dashed';
  ic.removeAttribute('data-ico'); renderIcons(ic.parentNode);
  var rec=LISTA.find(function(d){return d.id===DET_ID;});
  if(rec){
    rec.rezolvat=done;
    // Rezolvat si Omis se exclud reciproc - marcarea ca rezolvat anuleaza omiterea
    if(done && rec.omis){ rec.omis=false; _sincronizeazaBtnOmite(false); }
  }
  _arataSalvat();
}
function toggleRezolvat(){
  marcheazaRezolvat(!document.getElementById('btnConcl').classList.contains('done'));
}
function _sincronizeazaBtnOmite(activ){
  var btn=document.getElementById('btnOmite'); if(!btn)return;
  btn.classList.toggle('activ-omis',activ);
  var ic=document.getElementById('omiteIco'); if(!ic)return;
  ic.className=activ?'ti ti-circle-x':'ti ti-circle-minus';
  ic.removeAttribute('data-ico'); renderIcons(ic.parentNode);
}
function _sincronizeazaBtnConcl(activ){
  var btn=document.getElementById('btnConcl'); if(!btn)return;
  btn.classList.toggle('done',activ);
  var ic=document.getElementById('conclIco'); if(!ic)return;
  ic.className=activ?'ti ti-circle-check-filled':'ti ti-circle-dashed';
  ic.removeAttribute('data-ico'); renderIcons(ic.parentNode);
}
function marcheazaOmis(done){
  var btn=document.getElementById('btnOmite'); if(!btn)return;
  if(btn.classList.contains('activ-omis')===done)return;
  _sincronizeazaBtnOmite(done);
  var rec=LISTA.find(function(d){return d.id===DET_ID;});
  if(rec){
    rec.omis=done;
    // Omis nu are rost sa mai intre in deploy - anuleaza automat "rezolvat"
    if(done && rec.rezolvat){ rec.rezolvat=false; _sincronizeazaBtnConcl(false); }
  }
  _arataSalvat();
}
function toggleOmis(){
  var btn=document.getElementById('btnOmite'); if(!btn)return;
  marcheazaOmis(!btn.classList.contains('activ-omis'));
}

// ── SURSA (stanga, read-only, cu highlight linii sim/uniq + buton de mutare) ──
var _selBloc=null;   // indicele blocului selectat din stanga (click pe el)
var _blocuriAduse={};   // text integral bloc adus din sursa -> true (marcat verde)
function _statusLiniiBloc(srcText, targetText){
  var A=srcText.split('\n');
  if(!targetText){ return A.map(function(){return 'uniq';}); }
  var B=targetText.split('\n');
  var na=A.map(_normLinie), nb=B.map(_normLinie);
  var ops=new SequenceMatcher(na, nb).get_opcodes();
  var status=new Array(A.length).fill('uniq');
  ops.forEach(function(o){
    var tag=o[0],i1=o[1],i2=o[2],j1=o[3],j2=o[4];
    if(tag==='equal'){
      for(var i=i1;i<i2;i++) status[i]='sim';
    } else if(tag==='replace'){
      var lenA=i2-i1, lenB=j2-j1, comun=Math.min(lenA,lenB);
      for(var k=0;k<comun;k++){
        var sc=scor_similaritate(A[i1+k], B[j1+k]);
        status[i1+k]=(sc>=40)?'sim':'uniq';
      }
      for(var k=comun;k<lenA;k++) status[i1+k]='uniq';
    } else if(tag==='delete'){
      for(var i=i1;i<i2;i++) status[i]='uniq'; // fara pereche in target
    }
    // 'insert' nu afecteaza A, se ignora
  });
  return status;
}
// ── SURSA (stanga, read-only, cu buton de mutare in tinta) ──
function renderSursa(){
  _sugCache={};
  var elSrc=document.getElementById('scSrc');
  var prevScroll=elSrc?elSrc.scrollTop:0;
  var h='', lnum=0;
  var blocuriDr=null;
  var _difOrd={}, _difTot=0;
  DIF_EL.forEach(function(r){ if(r.st!=null && (r.tip==='sters'||r.tip==='modificat')){ _difTot++; _difOrd[r.i]=_difTot; } });
  DIF_EL.forEach(function(r){
    if(r.st==null){ h+='<div class="blk gol"></div>';return; }
    var cls=r.tip==='sters'?'del':r.tip==='modificat'?'mods':'eq';
    var difera=(r.tip==='sters'||r.tip==='modificat');
    var activ=(_selBloc===r.i) || (_plc && _plc.srcI===r.i);
    var status=null;
    if(activ && difera){
      if(!blocuriDr) blocuriDr=_split_statements(document.getElementById('editTA').value);
      var m=_celMaiApropiat(r.st, blocuriDr);
      var targetText=(m.idx>=0 && m.scor>=55)?blocuriDr[m.idx]:null;
      status=_statusLiniiBloc(r.st, targetText);
    }
    var wr=(modDif && r.tip==='modificat' && r.ss)?_dsLineRanges(r.ss):null;   // diff intra-linie pe sursa
    var liniiBloc=r.st.split('\n'), corp='';
    liniiBloc.forEach(function(ln, k){
      lnum++;
      var c=(wr && wr[k] && wr[k].length)?_hlCuWord(ln, wr[k], 'del'):hlSQL(ln);
      if(c==='')c='&#8203;';
      var stCls=status?(' ln-'+status[k]):'';
      corp+='<span class="bl-row'+stCls+'"><span class="bl-ln">'+lnum+'</span>'+c+'</span>';
    });
    var drag=difera?' draggable="true" ondragstart="dragStart(event,'+r.i+')" ondragend="dragEnd()"':'';
    var clickH=difera?' onclick="selectBloc('+r.i+')"':'';
    var acts='';
    if(difera){
      acts='<span class="blk-poz-perm">'+_difOrd[r.i]+'/'+_difTot+'</span>'
        +'<span class="blk-act">'
        +'<button class="ba-btn ba-eye" onclick="event.stopPropagation();deschidePv('+r.i+')" title="Previzualizare marita a diferentei">&#128065;</button>'
        +'<button class="ba-btn ba-ok" onclick="event.stopPropagation();aplicaBlocDirect('+r.i+')" title="Aplica acest bloc pe pozitia sugerata">&#10003;</button>'
        +'</span>';
    }
    var extraCls=(_ghid && _ghid.cur===r.i?' ghid-cur':'');
    var tagHtml='';
    if(difera){
      var sgB=_sugereazaCached(r.i), tagTxt=null, tagCls=null;
      if(r.tip==='sters'){
        if(sgB && sgB.pick && sgB.pick.tip==='blk' && sgB.pick.scor>=50){
          tagTxt=Math.round(sgB.pick.scor)+'% SIMILAR'; tagCls='tag-similar';
        } else { tagTxt='DOAR SURSA'; tagCls='tag-sters'; extraCls+=' no-match'; }
      } else if(r.tip==='modificat'){
        tagTxt='MODIFICAT'; tagCls='tag-mod';
      }
      if(tagTxt) tagHtml='<span class="blk-tag '+tagCls+'">'+tagTxt+'</span>';
    }

    if(activ){
      h+='<div class="sel-dest-wrap" data-i="'+r.i+'"'+drag+clickH+' style="cursor:pointer">'
        +'<div class="sel-dest-label" style="display:flex;align-items:center;justify-content:center;gap:8px;position:relative">'
          +'<span>&rarr; BLOC SELECTAT &larr;</span>'
          +'<button onclick="event.stopPropagation();iesireFocusStanga()" '
          +'style="position:absolute;right:10px;top:50%;transform:translateY(-50%);background:transparent;border:1px solid #fff;color:#fff;'
          +'border-radius:4px;width:18px;height:18px;display:inline-flex;align-items:center;justify-content:center;cursor:pointer;font-size:11px;line-height:1">✕</button>'
        +'</div>'
        +'<div class="blk sel-dest-bloc" style="position:relative">'+corp+'</div></div>';
    } else {
      h+='<div class="blk '+cls+(difera?' trag':'')+extraCls+'" data-i="'+r.i+'"'+drag+clickH+'>'+tagHtml+acts+corp+'</div>';
    }
  });
  h+='<div class="eof-marker eof-src"><i class="ti ti-circle-check-filled" aria-hidden="true"></i>Sfarsit procedura</div>';
  document.getElementById('dSrc').innerHTML=h;
  renderIcons(document.getElementById('dSrc'));
  if(typeof _vedere!=='undefined' && _vedere==='inline')renderInline();
  if(elSrc){
    _syncLock=true;
    elSrc.scrollTop=prevScroll;
    requestAnimationFrame(function(){_syncLock=false;desenLegaturiGutter();});
  } else { desenLegaturiGutter(); }
}

// click pe bloc = selectie + preview destinatie sugerata (read-only)
// in mod ghidat: click = muta blocul activ (bara de merge se muta sub el)
function selectBloc(i){
  if(_ghid){
    if(_ghid.cur!==i){ _ghid.cur=i; ghidArata(); }
    return;
  }
  if(typeof _vedere!=='undefined' && _vedere==='inline'){ deschidePv(i); return; }
  mutaBloc(i);
}
function _scrollLaBlocSursa(i){
  var elBlk=document.querySelector('#dSrc [data-i="'+i+'"]');
  var cont=document.getElementById('scSrc');
  if(!elBlk||!cont)return;
  _syncLock=true;
  var contRect=cont.getBoundingClientRect(), elRect=elBlk.getBoundingClientRect();
  var delta=elRect.top-contRect.top;
  var tinta=Math.max(0, cont.scrollTop+delta-10);
  if(typeof cont.scrollTo==='function'){ cont.scrollTo({top:tinta,behavior:'smooth'}); }
  else { cont.scrollTop=tinta; }
  setTimeout(function(){_syncLock=false;},400);
}
function scrollLaPickDreapta(){
  var list=document.querySelector('#plcTargets .plc-dest-list');
  if(!list)return;
  var pick=list.querySelector('.pick');
  if(!pick)return;
  var listRect=list.getBoundingClientRect(), pickRect=pick.getBoundingClientRect();
  var tinta=Math.max(0, list.scrollTop+(pickRect.top-listRect.top)-10);
  if(typeof list.scrollTo==='function'){ list.scrollTo({top:tinta,behavior:'smooth'}); }
  else { list.scrollTop=tinta; }
}

function iesireFocusStanga(){
  if(_plc){ anuleazaPlasare(); return; }   // anuleazaPlasare acum face renderSursa()
  if(_selBloc!==null){ _selBloc=null; _ascundePreviewDr(); renderSursa(); }
}
function _ascundePreviewDr(){
  var prev=document.getElementById('plcPreview');
  if(prev){ prev.classList.remove('on'); prev.innerHTML=''; }
  if(!_plc){ var tb=document.getElementById('dtbBar'); if(tb)tb.style.display=''; }
}

function _liniiPlain(txt){
  return txt.split('\n').map(function(l){
    var c=hlSQL(l);
    if(c==='')c='&#8203;';
    return '<span class="pt-ln" style="display:block; text-align:left; white-space:pre-wrap;">'+c+'</span>';
  }).join('');
}function _arataPrevDr(i){
  var tb=document.getElementById('dtbBar'); if(tb)tb.style.display='none';
  var el=DIF_EL.find(function(e){return e.i===i;});
  if(!el||el.st==null)return;
  var grup=_grupPlasare(i);
  var textGrup=grup?grup.text:el.st;
  var ta=document.getElementById('editTA');
  var blocuriDr=_split_statements(ta.value);
  var idxPref=_gapSugerat(el,blocuriDr);
  var m=_celMaiApropiat(el.st, blocuriDr, idxPref);
  var sugg=(m.idx>=0 && m.scor>=55)?{tip:'blk',idx:m.idx}:{tip:'gap',idx:idxPref};
  var textPlasare=(sugg.tip==='gap')?textGrup:el.st;

  var ctxH='';
  if(blocuriDr.length===0){
    ctxH='<div class="prev-ctx-top">&mdash; baza tinta goala &mdash;</div>';
  } else if(sugg.tip==='gap'){
    var inainte=sugg.idx>0?blocuriDr[sugg.idx-1]:null;
    var dupa=sugg.idx<blocuriDr.length?blocuriDr[sugg.idx]:null;
    if(inainte) ctxH+='<div class="prev-ctx-bloc">'+_liniiPlain(inainte)+'</div>';
    else ctxH+='<div class="prev-ctx-top">&mdash; inceput fisier &mdash;</div>';
    ctxH+='<div class="prev-dest-wrap" id="prevTarget"><div class="prev-dest-label">&darr; VA FI INSERAT AICI &darr;</div>'
      +'<div class="prev-dest-bloc">'+_liniiPlain(textPlasare)+'</div></div>';
    if(dupa) ctxH+='<div class="prev-ctx-bloc">'+_liniiPlain(dupa)+'</div>';
    else ctxH+='<div class="prev-ctx-top">&mdash; sfarsit fisier &mdash;</div>';
  } else {
    var blkTinta=blocuriDr[sugg.idx];
    var inainte2=sugg.idx>0?blocuriDr[sugg.idx-1]:null;
    var dupa2=sugg.idx<blocuriDr.length-1?blocuriDr[sugg.idx+1]:null;
    if(inainte2) ctxH+='<div class="prev-ctx-bloc">'+_liniiPlain(inainte2)+'</div>';
    ctxH+='<div class="prev-dest-wrap" id="prevTarget"><div class="prev-dest-label">&darr; VA FI INLOCUIT &darr;</div>'
      +'<div class="prev-dest-bloc prev-dest-replace">'+_blocDiff(blkTinta, textPlasare)+'</div></div>';
    if(dupa2) ctxH+='<div class="prev-ctx-bloc">'+_liniiPlain(dupa2)+'</div>';
  }

  var panouDr='<div class="plc-dest-panel" style="width:100%;flex:1;display:flex;flex-direction:column;height:100%;overflow:hidden">'
    +'<div class="plc-dest-hd">Destinatie sugerata &middot; apasa &rarr; pe bloc pentru a plasa</div>'
    +'<div class="plc-dest-list" style="overflow-y:auto;flex:1;padding:8px 16px;text-align:left">'+ctxH+'</div>'
    +'</div>';

  var prev=document.getElementById('plcPreview');
  prev.style.flexDirection='row';
  prev.innerHTML='<div style="display:flex;width:100%;height:100%;min-height:0;overflow:hidden">'+panouDr+'</div>';
  prev.classList.add('on');
  renderIcons(prev);

  requestAnimationFrame(function(){
    var t=document.getElementById('prevTarget');
    if(!t)return;
    var cont=t.closest('.plc-dest-list');
    if(!cont){ return; }
    var contRect=cont.getBoundingClientRect(), tRect=t.getBoundingClientRect();
    cont.scrollTop=Math.max(0, cont.scrollTop+(tRect.top-contRect.top)-10);
  });
}

// ── MOD PLASARE (buton → pe bloc) ──
var _plc=null, _plcLastI=null, _prevTimer=null;
// grupeaza pt PLASARE (nu afecteaza split-ul/diff-ul): daca blocul e un IF/WHILE...BEGIN
// neinchis, il leaga de urmatoarele blocuri pana se echilibreaza BEGIN/END - ca blocul
// mutat sa fie intotdeauna o secventa valida din punct de vedere sintactic.
var _GRUP_PLASARE_MAX_BLOCURI=6;   // plafon de siguranta: peste atatea blocuri, renuntam la grupare
function _grupPlasare(startI){
  var idx=DIF_EL.findIndex(function(e){return e.i===startI;});
  if(idx===-1)return null;
  if(idx===0)return {indices:[startI], text:DIF_EL[idx].st};   // headerul (semnatura procedurii) nu se grupeaza niciodata
  var el0=DIF_EL[idx];
  if(el0.st==null)return {indices:[startI], text:el0.st};
  var depth=_adancimeBeginEnd(el0.st);
  if(depth<=0)return {indices:[startI], text:el0.st};
  var indices=[startI], texte=[el0.st], k=idx;
  while(depth>0 && k+1<DIF_EL.length && indices.length<_GRUP_PLASARE_MAX_BLOCURI){
    k++;
    var e=DIF_EL[k];
    if(e.st==null)continue;   // intrari doar-adaugate nu fac parte din secventa sursa
    depth+=_adancimeBeginEnd(e.st);
    indices.push(e.i); texte.push(e.st);
  }
  if(depth>0){
    // nu s-a echilibrat in limita de siguranta (probabil BEGIN apartine unui
    // construct mult mai larg, ex. procedura intreaga) - renuntam la grupare,
    // mutam blocul original singur, neechilibrat, mai bine decat sa inghitim tot obiectul
    return {indices:[startI], text:el0.st};
  }
  return {indices:indices, text:texte.join('\n')};
}
var _sugCache={};
function _sugereazaCached(i){
  if(Object.prototype.hasOwnProperty.call(_sugCache,i)) return _sugCache[i];
  var s=_sugereaza(i);
  _sugCache[i]=s;
  return s;
}
function _culoareBloc(el){
  return el.tip==='sters' ? 'var(--coral)' : 'var(--amber)';
}
// Diagrama de conectori din gutter-ul dintre panouri (doar in Side-by-side):
// o curba subtire leaga mijlocul blocului din stanga de linia corespunzatoare
// din dreapta (calculata cu aceeasi pozitie REALA folosita si la centrarea din
// mod ghidat - vezi _liniiStartBlocuri). Fara umplere, doar contur, ca sa nu
// se "umfle" vizual cand cele doua coloane sunt decalate de scroll independent.
function _curataMarkereInsertie(){
  document.querySelectorAll('#editHLInner .hl-line.ins-marker').forEach(function(el){ el.classList.remove('ins-marker'); el.removeAttribute('title'); });
}
function desenLegaturiGutter(){
  var svg=document.getElementById('dGutSvg'), gut=document.getElementById('dGut');
  if(!svg||!gut) return;
  if(typeof _vedere!=='undefined' && _vedere==='inline'){ svg.innerHTML=''; _curataMarkereInsertie(); return; }
  if(!DIF_EL||!DIF_EL.length){ svg.innerHTML=''; _curataMarkereInsertie(); return; }
  if(typeof _plc!=='undefined' && _plc){ svg.innerHTML=''; _curataMarkereInsertie(); return; }
  var gutR=gut.getBoundingClientRect();
  var w=gutR.width, hgt=gutR.height;
  if(w<=0||hgt<=0){ svg.innerHTML=''; _curataMarkereInsertie(); return; }
  var ta=document.getElementById('editTA'), wrapEl=document.getElementById('editWrap');
  if(!ta||!wrapEl||wrapEl.classList.contains('plc-hide')){ svg.innerHTML=''; _curataMarkereInsertie(); return; }
  var wrapR=wrapEl.getBoundingClientRect();
  var lineH=parseFloat(getComputedStyle(ta).lineHeight)||19;
  var taVal=ta.value;
  var parts=[];
  var lstarturiInsertie=[];
  DIF_EL.forEach(function(el){
   try{
    if(el.st==null) return;
    if(!(el.tip==='sters'||el.tip==='modificat')) return;
    var blk=document.querySelector('#dSrc .blk[data-i="'+el.i+'"]');
    if(!blk) return;
    var br=blk.getBoundingClientRect();
    var ly=br.top+br.height/2-gutR.top;
    if(ly<-60||ly>hgt+60) return;
    var sg=_sugereazaCached(el.i);
    var ry=ly, areCorespondent=false, scor=0;
    var grpEls=(el.tip==='modificat')?document.querySelectorAll('#editHLInner .hl-line[data-i="'+el.i+'"]'):null;
    if(grpEls && grpEls.length){
      var gTop=Infinity, gBot=-Infinity;
      grpEls.forEach(function(ge){ var r=ge.getBoundingClientRect(); if(r.top<gTop)gTop=r.top; if(r.bottom>gBot)gBot=r.bottom; });
      ry=(gTop+gBot)/2-gutR.top;
      areCorespondent=true; scor=100;
    } else if(sg){
      var b=sg.blocuriDr, p=sg.pick;
      areCorespondent=(p.tip==='blk');
      var idxT=areCorespondent?p.idx:Math.min(p.idx,b.length);
      var starts=_liniiStartBlocuri(taVal,b);
      var lstart=(idxT<starts.length)?starts[idxT]:(taVal?taVal.split('\n').length:0);
      var span=(areCorespondent&&b[idxT])?b[idxT].split('\n').length:1;
      ry=areCorespondent
        ? (wrapR.top+(lstart+span/2)*lineH-gutR.top)
        : (wrapR.top+lstart*lineH-gutR.top);
      scor=p.scor||0;
      if(!areCorespondent) lstarturiInsertie.push(lstart);
    }
    if(ry<-60||ry>hgt+60) return;
    var midX=w/2, color=_culoareBloc(el);
    var titlu=areCorespondent
      ? ('Corespondent gasit in tinta ('+Math.round(scor)+'% similar)')
      : 'Fara corespondent in tinta - va fi inserat ca bloc nou';
    if(areCorespondent){
      parts.push('<path d="M0,'+ly.toFixed(1)+' C'+midX.toFixed(1)+','+ly.toFixed(1)+' '+midX.toFixed(1)+','+ry.toFixed(1)+' '+w.toFixed(1)+','+ry.toFixed(1)+
        '" stroke="'+color+'" stroke-width="1.5" fill="none" opacity="0.8"><title>'+titlu+'</title></path>');
      parts.push('<circle cx="0" cy="'+ly.toFixed(1)+'" r="3" fill="'+color+'"/>');
      parts.push('<circle cx="'+w.toFixed(1)+'" cy="'+ry.toFixed(1)+'" r="3" fill="'+color+'"/>');
    } else {
      parts.push('<path d="M0,'+ly.toFixed(1)+' C'+midX.toFixed(1)+','+ly.toFixed(1)+' '+midX.toFixed(1)+','+ry.toFixed(1)+' '+w.toFixed(1)+','+ry.toFixed(1)+
        '" stroke="'+color+'" stroke-width="1.5" fill="none" opacity="0.4" stroke-dasharray="3,3"><title>'+titlu+'</title></path>');
      parts.push('<circle cx="0" cy="'+ly.toFixed(1)+'" r="3" fill="'+color+'"/>');
      parts.push('<circle cx="'+w.toFixed(1)+'" cy="'+ry.toFixed(1)+'" r="3" fill="none" stroke="'+color+'" stroke-width="1.5"/>');
    }
   } catch(eBlk){ /* un bloc problematic nu trebuie sa strice restul diagramei */ }
  });
  svg.innerHTML=parts.join('');
  _curataMarkereInsertie();
  var liniiHL=document.querySelectorAll('#editHLInner .hl-line');
  lstarturiInsertie.forEach(function(lst){
    var tinta=lst-1;   // linia goala reala e chiar inaintea inceputului blocului urmator
    if(tinta>=0 && tinta<liniiHL.length){
      liniiHL[tinta].classList.add('ins-marker');
      liniiHL[tinta].title='Aici ar veni blocul fara corespondent din sursa (dupa aplicare)';
    }
  });
}
var _gutRafPending=false;
function _cereRedrawGutter(){
  if(_gutRafPending) return;
  _gutRafPending=true;
  requestAnimationFrame(function(){ _gutRafPending=false; desenLegaturiGutter(); });
}

// ── SCROLL SINCRONIZAT PE ANCORE (ca in Araxis Merge) ─────────────────────────
// Nu copiaza scrollTop 1:1 (coloanele au lungimi diferite) - construieste o
// lista de "ancore" din blocurile cu corespondent cunoscut in ambele panouri
// (aceleasi data-i folosite si de conectorii curbati), apoi interpoleaza
// liniar intre cele mai apropiate doua ancore de mijlocul ecranului sursa,
// ca sa gaseasca pozitia echivalenta in panoul celalalt.
var _panSyncLock=false, _syncRafPending=false, _syncDirCer=null;
function _cereSincScroll(dinStanga){
  if(_panSyncLock) return;
  _syncDirCer=dinStanga;
  if(_syncRafPending) return;
  _syncRafPending=true;
  requestAnimationFrame(function(){ _syncRafPending=false; _sincronizeazaScroll(_syncDirCer); });
}
function _construiesteAncoreScroll(scSrc,scWork){
  var anc=[], harta={};
  var srcBaseY=scSrc.getBoundingClientRect().top, dstBaseY=scWork.getBoundingClientRect().top;
  var srcTop=scSrc.scrollTop, dstTop=scWork.scrollTop;
  document.querySelectorAll('#dSrc .blk[data-i]').forEach(function(b){
    var id=b.getAttribute('data-i');
    if(harta[id]) return;
    var r=b.getBoundingClientRect();
    harta[id]={ly:srcTop+(r.top-srcBaseY)+r.height/2};
  });
  document.querySelectorAll('#editHLInner .hl-line[data-i]').forEach(function(b){
    var id=b.getAttribute('data-i');
    if(!harta[id]) return;
    var r=b.getBoundingClientRect();
    var yTop=dstTop+(r.top-dstBaseY), yBot=dstTop+(r.bottom-dstBaseY);
    if(harta[id].ryTop==null||yTop<harta[id].ryTop) harta[id].ryTop=yTop;
    if(harta[id].ryBot==null||yBot>harta[id].ryBot) harta[id].ryBot=yBot;
  });
  for(var id in harta){
    if(harta[id].ryTop==null) continue; // fara corespondent in dreapta (ex: sters)
    anc.push({ly:harta[id].ly, ry:(harta[id].ryTop+harta[id].ryBot)/2});
  }
  anc.sort(function(a,b){return a.ly-b.ly;});
  return anc;
}
function _sincronizeazaScroll(dinStanga){
  if(typeof _vedere!=='undefined' && _vedere==='inline') return;
  if(typeof _plc!=='undefined' && _plc) return;
  var scSrc=document.getElementById('scSrc'), scWork=document.getElementById('scWork');
  if(!scSrc||!scWork) return;
  var editWrap=document.getElementById('editWrap');
  if(!editWrap||editWrap.classList.contains('plc-hide')) return;
  var anc=_construiesteAncoreScroll(scSrc,scWork);
  if(!anc.length) return;
  var srcPanel=dinStanga?scSrc:scWork, dstPanel=dinStanga?scWork:scSrc;
  var centruSrc=srcPanel.scrollTop+srcPanel.clientHeight/2;
  var lo=null,hi=null;
  for(var k=0;k<anc.length;k++){
    var av=dinStanga?anc[k].ly:anc[k].ry;
    if(av<=centruSrc) lo=anc[k];
    if(av>=centruSrc){ hi=anc[k]; break; }
  }
  var centruDst;
  if(lo&&hi&&lo!==hi){
    var avLo=dinStanga?lo.ly:lo.ry, avHi=dinStanga?hi.ly:hi.ry;
    var dvLo=dinStanga?lo.ry:lo.ly, dvHi=dinStanga?hi.ry:hi.ly;
    var t=(avHi>avLo)?(centruSrc-avLo)/(avHi-avLo):0;
    centruDst=dvLo+t*(dvHi-dvLo);
  } else {
    var a=lo||hi, av=dinStanga?a.ly:a.ry, dv=dinStanga?a.ry:a.ly;
    centruDst=dv+(centruSrc-av);   // dincolo de ultima ancora - pastreaza decalajul
  }
  var tinta=Math.max(0, centruDst-dstPanel.clientHeight/2);
  if(Math.abs(tinta-dstPanel.scrollTop)<1) return;
  _panSyncLock=true;
  dstPanel.scrollTop=tinta;
  _cereRedrawGutter();
  requestAnimationFrame(function(){ requestAnimationFrame(function(){ _panSyncLock=false; }); });
}

function _sugereaza(i){
  var el=DIF_EL.find(function(e){return e.i===i;});
  if(!el||el.st==null)return null;
  var grup=_grupPlasare(i);
  var textGrup=grup?grup.text:el.st;
  var ta=document.getElementById('editTA');
  var blocuriDr=_split_statements(ta.value);
  var idxPref=_gapSugerat(el,blocuriDr);
  // pozitia se cauta cu fragmentul MIC, original (precis) - nu cu textul grupat,
  // care ar fi comparat nedrept cu blocuri tinta mici si ar da scoruri false
  var idxInDif=DIF_EL.findIndex(function(e){return e.i===i;});
  var esteAntet=(idxInDif===0);
  var sugg;
  if(esteAntet && blocuriDr.length){
    // antetul (CREATE PROCEDURE/FUNCTION ... AS) e mereu unic si mereu primul
    // bloc, in sursa si in tinta - il potrivim direct, fara scor de similaritate.
    // Altfel, daca s-au adaugat/schimbat multi parametri, scorul token-cu-token
    // poate scadea sub prag si sistemul renunta, sugerand gresit "insereaza la
    // inceput" desi antetul corespunzator exista deja in tinta.
    sugg={tip:'blk',idx:0,scor:100};
  } else {
    var m=_celMaiApropiat(el.st, blocuriDr, idxPref);
    sugg=(m.idx>=0 && m.scor>=55)?{tip:'blk',idx:m.idx,scor:m.scor}:{tip:'gap',idx:idxPref,scor:(m?m.scor:0)};
  }
  // la INLOCUIRE (blk) folosim fragmentul mic original (swap 1:1, sigur - nu
  // duplica restul grupului, care ar ramane neatins in blocurile tinta vecine).
  // la INSERARE IN GOL (gap) folosim textul grupat (aducem constructul complet,
  // presupunand ca lipseste in intregime din tinta).
  var textPlasare=(sugg.tip==='gap')?textGrup:el.st;
  var indicesPlasare=(sugg.tip==='gap' && grup)?grup.indices:[i];
  return {srcI:i, srcIndices:indicesPlasare, srcText:textPlasare, blocuriDr:blocuriDr, pick:sugg};
}
function mutaBloc(i){
  _selBloc=null; _ascundePreviewDr();
  var s=_sugereaza(i);
  if(!s)return;
  _plc=s;
  document.getElementById('plcNume').textContent=_scurt(s.srcText);
  document.getElementById('plcBar').classList.add('on');
  document.getElementById('editWrap').classList.add('plc-hide');
  document.getElementById('plcTargets').classList.add('on');
  { var tb=document.getElementById('dtbBar'); if(tb)tb.style.display='none'; }
  renderTargets(true);
  renderSursa();
  _scrollLaBlocSursa(i);
}
// aplica direct un bloc pe pozitia sugerata, fara a deschide UI-ul de plasare
// (butonul ✓ de pe bloc si modul ghidat)
function aplicaBlocDirect(i,textOverride){
  if(_ghid){ _ghid.aplicate++; _ghid.skipStreak=0; }
  var s=_sugereaza(i);
  if(!s)return;
  if(textOverride!=null && textOverride!==s.srcText) s.srcText=textOverride;
  _plc=s;
  confirmaPlasare();
  if(_ghid){
    var q=_ghidQueue();
    if(!q.length){ arataGhidDone(); return; }
    // i e acum stabil (ancorat pe pozitia in sursa) - gasim primul bloc
    // ramas nerezolvat dupa cel tocmai aplicat, la fel ca ghidUrm(); daca
    // era ultimul, revenim la inceputul listei ramase.
    var nxt=-1;
    for(var k=0;k<q.length;k++){ if(q[k]>i){nxt=q[k];break;} }
    _ghid.cur=(nxt!==-1)?nxt:q[0];
    ghidArata();
  }
}
function _tokensPotrivire(s){
  // ignoram sigiliul de tabel (#, ## sau @) la scor - #X, ##X si @X trebuie
  // sa se potriveasca ca acelasi identificator, altfel exact refactorul pe care
  // tool-ul e facut sa-l urmareasca (tabel temporar -> variabila tabel) ii scade
  // artificial scorul de potrivire si il impinge sub prag
  return normalizeaza(s).replace(/(^|[\s(,])[#@]{1,2}(\w)/g, '$1$2')
    .replace(/([(),;'])/g,' $1 ').replace(/\s+/g,' ').trim().split(' ');
}
function _celMaiApropiat(txt, blocuri, idxPreferat){
  var ta=_tokensPotrivire(txt), rez=[];
  for(var k=0;k<blocuri.length;k++){
    var s=_round_py(new SequenceMatcher(ta, _tokensPotrivire(blocuri[k])).ratio()*100,2);
    rez.push({idx:k, scor:s});
  }
  rez.sort(function(a,b){return b.scor-a.scor;});
  if(!rez.length) return {idx:-1,scor:-1};
  var best=rez[0], second=rez[1]?rez[1].scor:-1;
  if(best.scor - second < 8){
    // scoruri foarte apropiate = ambiguu, indiferent cat de mare e scorul absolut
    // (statement-uri aproape identice structural - UPDATE ... WHERE ... - pot
    // avea scoruri mari intre ele, gen 82.76 vs 82.35, dpearece difera doar
    // printr-un singur cuvant cheie). Preferam pozitia, nu doar scorul brut.
    if(idxPreferat!=null){
      var apropiat=rez.filter(function(r){ return Math.abs(r.idx-idxPreferat)<=1 && r.scor>=40; });
      if(apropiat.length) return apropiat[0];
    }
    if(best.scor<70) return {idx:-1, scor:-1};
  }
  return best;
}
function _gapSugerat(el, blocuriDr){
  var inainte=0;
  for(var k=0;k<DIF_EL.length;k++){ var r=DIF_EL[k]; if(r.i===el.i)break; if(r.dr!=null)inainte++; }
  return Math.min(inainte, blocuriDr.length);
}
function renderTargets(){
  var c=document.getElementById('plcTargets');
  var oldList=c.querySelector('.plc-dest-list');
  var oldScroll=oldList?oldList.scrollTop:null;

  var b=_plc.blocuriDr, p=_plc.pick;
  var lblSvg='<span class="pt-lbl"><svg viewBox="0 0 24 24"><path d="M6 9l6 6l6 -6"/></svg></span>';   // <-- AICI

  var lista='';
  lista+=_gapHTML(0, p);
  for(var k=0;k<b.length;k++){
    var pk=(p.tip==='blk'&&p.idx===k);
    // NOU
    lista+='<div class="pt-blk'+(pk?' pick':'')+'" onclick="pickTarget(\'blk\','+k+')" ondblclick="pickTarget(\'blk\','+k+');confirmaPlasare()" '
  +'ondragover="dzOver(event)" ondragleave="dzLeave(event)" ondrop="dzDrop(event,\'blk\','+k+')" title="Inlocuieste acest bloc">'
  +lblSvg+(pk?_blocDiff(b[k], _plc.srcText):_blocPlain(b[k]))+'</div>';
    lista+=_gapHTML(k+1, p);
  }
  if(b.length===0){ lista='<div style="padding:14px;color:var(--muted);font-size:11px">(baza tinta e goala)</div>'+_gapHTML(0,p); }
  var destPanel='<div class="plc-dest-panel">'
    +'<div class="plc-dest-hd">Alege pozitia — gol=insereaza · bloc=inlocuieste</div>'
    +'<div class="plc-dest-list">'+lista+'</div></div>';
  c.innerHTML=destPanel;
  renderIcons(c);

  var newList=c.querySelector('.plc-dest-list');
  if(newList){
    if(oldScroll!==null){ newList.scrollTop=oldScroll; }
    else {
      var pick=newList.querySelector('.pick');
      if(pick){
        var listRect=newList.getBoundingClientRect(), pickRect=pick.getBoundingClientRect();
        newList.scrollTop=Math.max(0, newList.scrollTop+(pickRect.top-listRect.top)-10);
      }
    }
  }
}

function _calcRezPreview(p, b, src){
  if(!p)return '';
  if(p.tip==='blk'){
    if(p.idx<0||p.idx>=b.length)return '';
    return _blocDiff(b[p.idx], src);
  } else {
    var rez='';
    var inainte=p.idx>0?b[p.idx-1]:null, dupa=p.idx<b.length?b[p.idx]:null;
    if(inainte){ rez+='<span style="opacity:.4">'+inainte.split('\n').map(function(l){return '<span class="pt-ln">'+hlSQL(l)+'</span>';}).join('')+'</span><span style="display:block;height:4px"></span>'; }
    rez+=src.split('\n').map(function(l){var cx=hlSQL(l);if(cx==='')cx='&#8203;';return '<span class="pt-ln has-mk"><span class="pt-mk m-add"></span>'+cx+'</span>';}).join('');
    if(dupa){ rez+='<span style="display:block;height:4px"></span><span style="opacity:.4">'+dupa.split('\n').map(function(l){return '<span class="pt-ln">'+hlSQL(l)+'</span>';}).join('')+'</span>'; }
    return rez;
  }
}
function _blocPlain(txt){
  return txt.split('\n').map(function(l){var c=hlSQL(l);if(c==='')c='&#8203;';return '<span class="pt-ln">'+c+'</span>';}).join('');
}
function _blocDiff(txt, src){
  var A=src.split('\n'), B=txt.split('\n');
  var na=A.map(_normLinie), nb=B.map(_normLinie);
  var ops=new SequenceMatcher(na, nb).get_opcodes();
  var tipLinie={}, wordLinie={};
  ops.forEach(function(o){
    var tag=o[0],i1=o[1],i2=o[2],j1=o[3],j2=o[4];
    if(tag==='replace'){ for(var j=j1;j<j2;j++){ tipLinie[j]='mod'; var si=i1+(j-j1); wordLinie[j]=_wordRanges(si<i2?A[si]:'', B[j]); } }
    else if(tag==='insert'){ for(var j2i=j1;j2i<j2;j2i++){ tipLinie[j2i]='add'; wordLinie[j2i]=[[0,B[j2i].length]]; } }
  });
  var out='';
  for(var i=0;i<B.length;i++){
    var tip=tipLinie[i], cont;
    if(tip && wordLinie[i] && wordLinie[i].length){ cont=_hlCuWord(B[i], wordLinie[i], tip==='add'?'adus':'mod'); }
    else { cont=hlSQL(B[i]); }
    if(cont==='')cont='&#8203;';
    var mk=tip?'<span class="pt-mk '+(tip==='add'?'m-add':'m-mod')+'"></span>':'';
    out+='<span class="pt-ln'+(tip?' has-mk':'')+'">'+mk+cont+'</span>';
  }
  return out;
}
function _gapHTML(idx, p){
  var pk=(p.tip==='gap'&&p.idx===idx);
  return '<div class="pt-gap'+(pk?' pick':'')+'" onclick="pickTarget(\'gap\','+idx+')" ondblclick="pickTarget(\'gap\','+idx+');confirmaPlasare()" '
    +'ondragover="dzOver(event)" ondragleave="dzLeave(event)" ondrop="dzDrop(event,\'gap\','+idx+')">'
    +'<span class="pt-gap-lbl">+ insereaza aici</span></div>';
}
function pickTarget(tip, idx){ if(!_plc)return; _plc.pick={tip:tip, idx:idx}; renderTargets(); }
function dragStart(ev, i){ try{ev.dataTransfer.setData('text/plain',String(i));}catch(x){} ev.dataTransfer.effectAllowed='copy'; mutaBloc(i); document.body.classList.add('dragging'); }
function dragEnd(){ document.body.classList.remove('dragging'); }
function dzOver(ev){ ev.preventDefault(); ev.dataTransfer.dropEffect='copy'; ev.currentTarget.classList.add('dz-hot'); }
function dzLeave(ev){ ev.currentTarget.classList.remove('dz-hot'); }
function dzDrop(ev, tip, idx){ ev.preventDefault(); ev.currentTarget.classList.remove('dz-hot'); document.body.classList.remove('dragging'); if(!_plc)return; _plc.pick={tip:tip,idx:idx}; confirmaPlasare(); }
function anuleazaPlasare(){ _incheiePlasare(); renderSursa(); }

var _istoricPlasari=[];
function confirmaPlasare(){
  if(!_plc)return;
  var ta=document.getElementById('editTA');
  var stareAnterioara=ta.value;
  try{
    var b=_plc.blocuriDr, txt=_plc.srcText, p=_plc.pick, indiciRef=_plc.srcIndices||[_plc.srcI];
    // IMPORTANT: nu reconstruim tot textul din b.join('\n') - asta ar sterge toate
    // liniile goale dintre TOATE instructiunile din fisier (nu doar in jurul
    // blocului atins), generand diferente noi, fantoma, la fiecare aplicare.
    // In loc, inlocuim/inseram DOAR liniile reale ocupate de blocul vizat,
    // folosind pozitia reala calculata cu _liniiStartBlocuri (aceeasi functie
    // folosita si la centrarea din mod ghidat) - tot restul textului ramane
    // exact cum era, caracter cu caracter.
    var linii=stareAnterioara.split('\n');
    var starts=_liniiStartBlocuri(stareAnterioara,b);
    var novi=txt.split('\n');
    var linieTinta;
    if(p.tip==='blk'){
      var s=starts[p.idx];
      var oldLen=b[p.idx]?b[p.idx].split('\n').length:0;
      Array.prototype.splice.apply(linii,[s,oldLen].concat(novi));
      linieTinta=s;
    } else {
      var s=(p.idx<starts.length)?starts[p.idx]:linii.length;
      Array.prototype.splice.apply(linii,[s,0].concat(novi));
      linieTinta=s;
    }
    ta.value=linii.join('\n');
    _istoricPlasari.push(stareAnterioara);
    _blocuriAduse[_norm(txt)]=true;
    _incheiePlasare();
    recalcDiff();renderSursa();_arataSalvat();autoResize();
    indiciRef.forEach(function(idxRef){
      var blk=document.querySelector('#dSrc .blk[data-i="'+idxRef+'"]');
      if(blk)blk.classList.add('mutat');
    });
    requestAnimationFrame(function(){
      var lineH=parseFloat(getComputedStyle(ta).lineHeight)||19;
      var sc=document.getElementById('scWork');
      sc.scrollTop=Math.max(0, linieTinta*lineH - sc.clientHeight/3);
    });
    var btnU=document.getElementById('btnUndoPlasare');
    btnU.style.display='';
    btnU.title='Anuleaza ultima plasare de bloc - Ctrl+Z ('+_istoricPlasari.length+' in istoric)';
  } catch(e){
    _incheiePlasare();
    console.error('Eroare la plasare:', e);
  }
}
function anuleazaUltimaPlasare(){
  if(!_istoricPlasari.length)return;
  document.getElementById('editTA').value=_istoricPlasari.pop();
  var btnU=document.getElementById('btnUndoPlasare');
  if(_istoricPlasari.length){
    btnU.title='Anuleaza ultima plasare de bloc - Ctrl+Z ('+_istoricPlasari.length+' in istoric)';
  } else {
    btnU.style.display='none';
  }
  resyncDiff();autoResize();
}

function _incheiePlasare(){
  if(_plc)_plcLastI=_plc.srcI;
  _plc=null; _selBloc=null; _ascundePreviewDr();
  document.getElementById('plcBar').classList.remove('on');
  document.getElementById('editWrap').classList.remove('plc-hide');
  document.getElementById('plcTargets').classList.remove('on');
  document.getElementById('plcTargets').innerHTML='';
}

// ── CONTOR REZOLVARE (blocurile aplicate dispar din diff dupa recalc) ─────────
function actualizeazaRezCnt(){
  var tot=0;
  DIF_EL.forEach(function(r){
    if(r.st!=null && (r.tip==='sters'||r.tip==='modificat')) tot++;
  });
  var el=document.getElementById('rezCnt');
  if(el){
    el.style.display=tot?'':'none';
    el.innerHTML='<span class="tcnt">'+tot+'</span> nerezolvate';
  }
  return tot;
}

// ── MOD GHIDAT (bara inline sub blocul activ; Enter = aplica, Esc = sari) ─────
// Semantica identica cu editorul de referinta: "Sari peste" e doar navigatie
// (nu marcheaza nimic permanent); daca faci un ocol complet numai cu sarituri,
// fara sa aplici nimic, modul se inchide singur.
var _ghid=null;   // {cur:indice DIF_EL, aplicate:n, sarite:n, skipStreak:n}
function _ghidQueue(){
  var q=[];
  DIF_EL.forEach(function(r){
    if(r.st!=null && (r.tip==='sters'||r.tip==='modificat')) q.push(r.i);
  });
  return q;
}
function _blurActiv(){ var a=document.activeElement; if(a&&a.blur)a.blur(); }
function _ghidBarInner(i, ord, tot){
  var infoT='';
  var sg=_sugereaza(i);
  if(sg){
    if(sg.pick.tip==='blk'){
      var sc2=scor_similaritate(DIF_EL.find(function(e){return e.i===i;}).st, sg.blocuriDr[sg.pick.idx]);
      infoT='&rarr; inlocuieste blocul evidentiat in dreapta ('+sc2+'% similar)';
    } else {
      infoT='&rarr; nu exista in tinta &middot; se insereaza la pozitia '+(sg.pick.idx+1)+' (marcata in dreapta)';
    }
  }
  return '<span class="gmb-pos">Diferenta '+(ord||'?')+' din '+tot+' ramase</span>'
    +'<span class="gmb-info">'+infoT+'</span>'
    +'<button class="gmb-ok" onclick="event.stopPropagation();ghidAplica()" title="Enter">&#10003; Aplica Sursa &rarr;</button>'
    +'<button class="gmb-sec" onclick="event.stopPropagation();ghidSari()" title="Esc">&#10005; Sari peste</button>'
    +'<button class="gmb-sec" onclick="event.stopPropagation();deschidePv('+i+')" title="Previzualizare">'
    +'<svg viewBox="0 0 24 24" width="13" height="13" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round" style="margin-right:4px;vertical-align:-2px">'
    +'<path d="M3 10a7 7 0 1 0 14 0a7 7 0 1 0 -14 0" /><path d="M21 21l-6 -6" /></svg>Preview</button>';
}
function _ghidBarHTML(i, ord, tot){
  return '<div class="gmb" data-gmb="'+i+'">'+_ghidBarInner(i,ord,tot)+'</div>';
}
function _ghidBarRowHTML(i, ord, tot, colspan){
  return '<tr class="gmb-row" data-gmb="'+i+'"><td colspan="'+colspan+'"><div class="gmb">'+_ghidBarInner(i,ord,tot)+'</div></td></tr>';
}
function toggleGhidat(){
  if(_ghid){ iesiGhidat(); return; }
  if(_plc)anuleazaPlasare();
  if(typeof _vedere!=='undefined' && _vedere==='inline') setVedere('side');
  var q=_ghidQueue();
  if(!q.length){ _ghid={cur:null,aplicate:0,sarite:0,skipStreak:0}; arataGhidDone(); return; }
  _ghid={cur:q[0], aplicate:0, sarite:0, skipStreak:0};
  document.getElementById('btnGhid').classList.add('grn');
  document.body.classList.add('guided-mode');
  document.getElementById('ghidTxt').textContent='Iesi din ghidat';
  _blurActiv();   // Enter/Esc trebuie sa ajunga la ghidaj, nu in textarea
  ghidArata();
}
function iesiGhidat(){
  _ghid=null;
  if(_tintaFocus){ _tintaFocus=null; pinteazaHL(); }
  document.getElementById('btnGhid').classList.remove('grn');
  document.body.classList.remove('guided-mode');
  document.getElementById('ghidTxt').textContent='Mod ghidat';
  inchidePv();
  renderSursa();
  _pozitioneazaGmbOverlay();
}
// Bara de control ca overlay peste AMBELE panouri (nu doar coloana sursa) -
// pozitionata dinamic, chiar sub blocul activ din stanga, in loc de fixa sus.
// Recalculata dupa fiecare render/scroll, ca sa "urmareasca" blocul.
function _pozitioneazaGmbOverlay(){
  var el=document.getElementById('gmbOverlay');
  if(!el)return;
  if(!_ghid || _ghid.cur==null || (typeof _vedere!=='undefined' && _vedere==='inline')){
    el.classList.remove('on'); el.innerHTML=''; return;
  }
  var dcols=document.querySelector('#ed .dcols');
  var blk=document.querySelector('#dSrc .blk[data-i="'+_ghid.cur+'"]');
  if(!dcols||!blk){ el.classList.remove('on'); return; }
  var dr=dcols.getBoundingClientRect(), br=blk.getBoundingClientRect();
  // bara acopera ambele coloane, deci trebuie sa stea sub oricare bloc e mai jos -
  // altfel, cand destinatia are mai multe linii decat sursa, bara ramane sus si
  // acopera evidentierea tinta din coloana dreapta (t-focus "pierdut in fundal")
  var bottomMax=br.bottom;
  var focusEl=document.querySelector('#editHLInner .t-focus.tf-last')||document.querySelector('#editHLInner .t-focus');
  if(focusEl){ bottomMax=Math.max(bottomMax, focusEl.getBoundingClientRect().bottom); }
  var q=_ghidQueue();
  var ord=q.indexOf(_ghid.cur)+1;
  el.innerHTML='<div class="gmb">'+_ghidBarInner(_ghid.cur, ord, q.length)+'</div>';
  el.classList.add('on');
  // pozitionam DUPA ce am pus continutul (ii stim inaltimea reala pt clamping)
  var barH=el.offsetHeight||70;
  var top=bottomMax-dr.top+8;
  top=Math.max(8, Math.min(top, dr.height-barH-8));
  el.style.top=top+'px';
}
function ghidArata(){
  if(!_ghid)return;
  var q=_ghidQueue();
  if(!q.length){ arataGhidDone(); return; }
  if(q.indexOf(_ghid.cur)===-1)_ghid.cur=q[0];
  renderSursa();
  _scrollAmbeleLaBloc(_ghid.cur);
  _pozitioneazaGmbOverlay();
}
function ghidAplica(textOverride){
  if(!_ghid||_ghid.cur==null)return;
  _blurActiv();
  _ghid.skipStreak=0;
  aplicaBlocDirect(_ghid.cur,textOverride);
}
function ghidSari(){
  if(!_ghid||_ghid.cur==null)return;
  _blurActiv();
  _ghid.sarite++;
  _ghid.skipStreak++;
  var q=_ghidQueue();
  if(_ghid.skipStreak>=q.length){
    // ocol complet doar cu sarituri - nimic nou de rezolvat, iesim
    iesiGhidat();
    return;
  }
  ghidUrm(1);
}
function ghidUrm(d){
  if(!_ghid)return;
  var q=_ghidQueue();
  if(!q.length){ arataGhidDone(); return; }
  var ix=q.indexOf(_ghid.cur);
  if(ix===-1){
    var old=_ghid.cur, gasit=-1;
    for(var k=0;k<q.length;k++){ if(q[k]>old){gasit=k;break;} }
    ix=(gasit===-1)?0:gasit;
  } else {
    ix=(ix+d+q.length)%q.length;
  }
  _ghid.cur=q[ix];
  ghidArata();
}
function arataGhidDone(){
  var a=_ghid?_ghid.aplicate:0, s=_ghid?_ghid.sarite:0;
  document.getElementById('gdBody').innerHTML=
    'Nu mai exista diferente nerezolvate in sursa.<br>'
    +'<b style="color:var(--neon)">'+a+'</b> aplicate &middot; <b style="color:var(--amber)">'+s+'</b> sarite in aceasta sesiune.';
  document.getElementById('ghidDone').classList.add('on');
  if(_ghid)iesiGhidat();
}
function inchideGhidDone(){ document.getElementById('ghidDone').classList.remove('on'); }

// ── MODAL PREVIZUALIZARE MARITA ───────────────────────────────────────────────
var _pvI=null, _pvSrcRaw='', _pvDstRaw=null;
function _blocSursaHL(el){
  var wr=(el.tip==='modificat' && el.ss)?_dsLineRanges(el.ss):null;
  return el.st.split('\n').map(function(ln,k){
    var c=(wr && wr[k] && wr[k].length)?_hlCuWord(ln, wr[k], 'del'):hlSQL(ln);
    if(c==='')c='&#8203;';
    return '<span class="pt-ln">'+c+'</span>';
  }).join('');
}
function deschidePv(i){
  var el=DIF_EL.find(function(e){return e.i===i;});
  if(!el||el.st==null)return;
  _pvI=i;
  var ord=0, tot=0;
  DIF_EL.forEach(function(r){ if(r.st!=null&&(r.tip==='sters'||r.tip==='modificat')){tot++; if(r.i===i)ord=tot;} });
  // in mod ghidat, modalul devine vederea principala a blocului activ:
  // titlul arata progresul, iar "Sari peste" e disponibil direct din modal
  if(_ghid){
    document.getElementById('pvTitlu').textContent='Diferenta '+ord+' din '+tot+' ramase';
    document.getElementById('pvSari').style.display='';
  } else {
    document.getElementById('pvTitlu').textContent='Previzualizare diferenta '+ord+'/'+tot+' ('+(el.tip==='sters'?'lipseste in tinta':'modificat')+')';
    document.getElementById('pvSari').style.display='none';
  }
  document.getElementById('pvSrc').innerHTML=_blocSursaHL(el);
  var blocuriDr=_split_statements(document.getElementById('editTA').value);
  var idxPref=_gapSugerat(el,blocuriDr);
  var m=_celMaiApropiat(el.st, blocuriDr, idxPref);
  var dst;
  if(m.idx>=0 && m.scor>=55){
    dst='<div class="prev-dest-label">&darr; VA FI INLOCUIT ('+m.scor.toFixed(0)+'% similar) &darr;</div>'+_blocDiff(blocuriDr[m.idx], el.st);
  } else {
    dst='<div class="prev-dest-label">&darr; VA FI INSERAT (pozitia '+(idxPref+1)+') &darr;</div>'
      +el.st.split('\n').map(function(l){var cx=hlSQL(l);if(cx==='')cx='&#8203;';return '<span class="pt-ln has-mk"><span class="pt-mk m-add"></span>'+cx+'</span>';}).join('');
  }
  document.getElementById('pvDst').innerHTML=dst;
  // textele brute disponibile pt butoanele "Din sursa"/"Din tinta" din caseta de rezultat final
  _pvSrcRaw=el.st;
  _pvDstRaw=(m.idx>=0 && m.scor>=55)?blocuriDr[m.idx]:null;
  var badd=document.getElementById('pvAddDst');
  if(badd) badd.style.display=_pvDstRaw?'':'none';
  // textul care CHIAR se va aplica (identic cu ce foloseste aplicaBlocDirect - vezi _sugereaza),
  // afisat editabil ca ultima sansa de corectie inainte de aplicare
  var sPrev=_sugereaza(i);
  var fta=document.getElementById('pvFinal');
  fta.value=sPrev?sPrev.srcText:el.st;
  fta.readOnly=true;
  _pvIstoric=[];
  _pvSetButoaneFolosite(false);
  _pvRenderHL();
  var beb=document.getElementById('pvEditBtn'); if(beb)beb.classList.remove('on');
  var betxt=document.getElementById('pvEditBtnTxt'); if(betxt)betxt.textContent='Editeaza';
  document.getElementById('pvAplica').onclick=function(){
    var ii=_pvI, textFinal=document.getElementById('pvFinal').value; inchidePv();
    if(_ghid){ _ghid.cur=ii; ghidAplica(textFinal); } else { aplicaBlocDirect(ii,textFinal); }
  };
  document.getElementById('pvSari').onclick=function(){
    var ii=_pvI; inchidePv();
    if(_ghid){ _ghid.cur=ii; ghidSari(); }
  };
  document.getElementById('pvOverlay').classList.add('on');
}
var _pvIstoric=[];
function _toggleEditPv(){
  var fta=document.getElementById('pvFinal'), btn=document.getElementById('pvEditBtn'), txt=document.getElementById('pvEditBtnTxt');
  fta.readOnly=!fta.readOnly;
  btn.classList.toggle('on',!fta.readOnly);
  txt.textContent=fta.readOnly?'Editeaza':'Se editeaza';
  if(!fta.readOnly)fta.focus();
}
function _pvSetButoaneFolosite(folosite){
  var s=document.getElementById('pvAddSrc'), d=document.getElementById('pvAddDst'), u=document.getElementById('pvUndoBtn');
  if(s)s.disabled=folosite;
  if(d)d.disabled=folosite;
  if(u)u.disabled=!_pvIstoric.length;
}
function _pvInsereaza(care){
  var text=(care==='src')?_pvSrcRaw:_pvDstRaw;
  if(text==null)return;
  var fta=document.getElementById('pvFinal');
  _pvIstoric.push(fta.value);   // pt Anuleaza / Ctrl+Z - .value= sters de mai jos rupe undo-ul nativ al browserului
  if(fta.readOnly)_toggleEditPv();   // activeaza editarea automat, ca schimbarea sa fie vizibil editabila
  fta.value=text;   // rescrie TOT continutul dintr-o data - nu se aduna la infinit din click-uri repetate
  _pvRenderHL();
  _pvSetButoaneFolosite(true);   // folosibile o singura data, pana la Anuleaza
}
function _pvUndo(){
  if(!_pvIstoric.length)return;
  var fta=document.getElementById('pvFinal');
  fta.value=_pvIstoric.pop();
  _pvRenderHL();
  _pvSetButoaneFolosite(false);
}
var _pvHLPending=false;
function _pvRenderHL(){
  // grupam randarile pe cadru: la scris rapid, mai multe evenimente input se transforma intr-o
  // singura recalculare pe frame, in loc sa refacem tot HTML-ul (scump) la fiecare litera tastata
  if(_pvHLPending)return;
  _pvHLPending=true;
  requestAnimationFrame(function(){
    _pvHLPending=false;
    var fta=document.getElementById('pvFinal'), hl=document.getElementById('pvFinalHL');
    if(!fta||!hl)return;
    hl.innerHTML=fta.value.split('\n').map(function(l){var c=hlSQL(l);return c===''?'&#8203;':c;}).join('\n');
    _pvSyncHL();
  });
}
function _pvSyncHL(){
  var fta=document.getElementById('pvFinal'), hl=document.getElementById('pvFinalHL');
  if(!fta||!hl)return;
  hl.style.transform='translate(-'+fta.scrollLeft+'px,-'+fta.scrollTop+'px)';
}
function inchidePv(){ document.getElementById('pvOverlay').classList.remove('on'); _pvI=null; }

// ── MINIMAP SURSA (benzi clicabile, sar la bloc + la tinta corespunzatoare) ───
function _scrollTintaLaBloc(i){ _scrollAmbeleLaBloc(i); }
// scroll simultan pe ambele panouri, sub un singur lacat de sincronizare -
// altfel scroll-sync-ul 1:1 suprascrie pozitia tintei imediat dupa setare
var _tintaFocus=null;   // {s,e} - liniile blocului tinta evidentiat in ghidat
function _scrollAmbeleLaBloc(i){
  var el=DIF_EL.find(function(e){return e.i===i;});
  if(_vedere==='inline'){
    var wrap=document.getElementById('inlineView');
    var blk=document.querySelector('#ilBody [data-i="'+i+'"]');
    if(wrap&&blk){
      var d=blk.getBoundingClientRect().top-wrap.getBoundingClientRect().top;
      wrap.scrollTop=Math.max(0,wrap.scrollTop+d-80);
    }
    return;
  }
  _syncLock=true;
  var cont=document.getElementById('scSrc');
  var blkS=document.querySelector('#dSrc [data-i="'+i+'"]');
  var ancoraSus=Math.max(80, cont?cont.clientHeight/3:80);
  if(blkS&&cont){
    var brS=blkS.getBoundingClientRect();
    var centruS=brS.top+brS.height/2-cont.getBoundingClientRect().top;
    cont.scrollTop=Math.max(0,cont.scrollTop+centruS-ancoraSus);
  }
  var focusNou=null, linie=0, span=1, ta=document.getElementById('editTA');
  var sg=(el&&el.st!=null)?_sugereaza(i):null;
  if(sg){
    var b=sg.blocuriDr, p=sg.pick;
    var taVal=ta.value;
    var starts=_liniiStartBlocuri(taVal, b);
    var areTinta=(p.tip==='blk');
    var idxT=areTinta?p.idx:Math.min(p.idx,b.length);
    linie=(idxT<starts.length)?starts[idxT]:(taVal?taVal.split('\n').length:0);
    span=(areTinta&&b[idxT])?b[idxT].split('\n').length:1;
    if(_ghid)focusNou={s:linie, e:linie+span};
  }
  var focusSchimbat=(!_tintaFocus)!==(!focusNou) ||
    (_tintaFocus && focusNou && (_tintaFocus.s!==focusNou.s || _tintaFocus.e!==focusNou.e));
  if(focusSchimbat){ _tintaFocus=focusNou; pinteazaHL(); }
  if(sg){
    // scroll dupa pozitia REALA din DOM a evidentierii (.t-focus), la fel ca la sursa -
    // formula veche (linie*lineH) presupunea inaltime uniforma pe linie si se desincroniza
    // fata de sursa cand cele doua coloane au numar diferit de blocuri/linii (cazul normal la diff)
    var sc=document.getElementById('scWork');
    var ancoraSusD=Math.max(80, sc.clientHeight/3);
    var focusEl=document.querySelector('#editHLInner .t-focus.tf-first')||document.querySelector('#editHLInner .t-focus');
    if(focusEl){
      var brD=focusEl.getBoundingClientRect();
      var centruD=brD.top+brD.height/2-sc.getBoundingClientRect().top;
      sc.scrollTop=Math.max(0, sc.scrollTop+centruD-ancoraSusD);
    } else {
      var lineH=parseFloat(getComputedStyle(ta).lineHeight)||19;
      sc.scrollTop=Math.max(0, (linie+span/2)*lineH - ancoraSusD);
    }
  }
  setTimeout(function(){_syncLock=false;},80);
}

// popup la hover pe sageata: exact efectul aplicarii (ce dispare / ce vine)
function ilPrevShow(ev, i){
  var el=DIF_EL.find(function(e){return e.i===i;});
  if(!el||el.st==null)return;
  var sg=_sugereaza(i);
  if(!sg)return;
  var h='';
  if(sg.pick.tip==='blk'){
    h+='<div class="mp-sec"><div class="mp-t mp-rem">&minus; Sters din tinta (bloc inlocuit)</div>'+_blocPlain(sg.blocuriDr[sg.pick.idx])+'</div>';
    h+='<div class="mp-sec"><div class="mp-t mp-add">+ Adaugat din sursa</div>'+_blocPlain(el.st)+'</div>';
  } else {
    h+='<div class="mp-sec"><div class="mp-t mp-add">+ Adaugat din sursa (inserare la pozitia '+(sg.pick.idx+1)+', nimic nu se sterge)</div>'+_blocPlain(sg.srcText)+'</div>';
  }
  var p=document.getElementById('ilPrev');
  p.innerHTML=h;
  p.classList.add('on');
  var r=ev.currentTarget.getBoundingClientRect();
  var top=r.bottom+6;
  if(top+p.offsetHeight>window.innerHeight-10) top=Math.max(10, r.top-p.offsetHeight-6);
  p.style.top=top+'px';
  p.style.left=Math.max(10, Math.min(window.innerWidth-p.offsetWidth-10, r.left+18))+'px';
}
function ilPrevHide(){ var p=document.getElementById('ilPrev'); if(p)p.classList.remove('on'); }

// ── VEDERE SIDE-BY-SIDE / INLINE ──────────────────────────────────────────────
var _vedere='side';
function setVedere(v){
  if(_vedere===v)return;
  if(typeof _plc!=='undefined' && _plc && typeof _incheiePlasare==='function') _incheiePlasare();
  if(v==='inline' && typeof _ghid!=='undefined' && _ghid && typeof iesiGhidat==='function') iesiGhidat();
  _vedere=v;
  document.getElementById('btnVSide').classList.toggle('grn',v==='side');
  document.getElementById('btnVInline').classList.toggle('grn',v==='inline');
  var dc=document.querySelector('#ed .dcols'), iv=document.getElementById('inlineView');
  if(v==='inline'){ dc.style.display='none'; iv.classList.add('on'); renderInline(); document.getElementById('dGutSvg').innerHTML=''; }
  else { dc.style.display=''; iv.classList.remove('on'); renderSursa(); pinteazaHL(); autoResize(); _cereRedrawGutter(); }
  if(_ghid)ghidArata();
}
function _ilLinii(txt, lnStart, ranges){
  // randuri pt o parte a unui bloc: {ln, content}[] + linia finala
  var ln=lnStart, out=[];
  (txt||'').split('\n').forEach(function(l,k){
    ln++;
    var c=(ranges&&ranges[k]&&ranges[k].length)?_hlCuWord(l,ranges[k],'mod'):hlSQL(l);
    if(c==='')c='&#8203;';
    out.push({ln:ln, content:c});
  });
  return {rows:out, ln:ln};
}
var INLINE_COLSPAN=5;
function renderInline(){
  var body=document.getElementById('ilBody');
  if(!body)return;
  ilPrevHide();
  var wrap=document.getElementById('inlineView');
  var prevScroll=wrap?wrap.scrollTop:0;
  var _difOrd={}, _difTot=0;
  DIF_EL.forEach(function(r){ if(r.st!=null && (r.tip==='sters'||r.tip==='modificat')){ _difTot++; _difOrd[r.i]=_difTot; } });
  var h='', lnA=0, lnB=0;
  DIF_EL.forEach(function(r){
    var difera=(r.tip==='sters'||r.tip==='modificat');
    var linii=[];   // {cls, lnLeft, lnRight, mk, content}
    if(r.tip==='egal'){
      var rr=_ilLinii(r.dr!=null?r.dr:r.st, lnB, null);
      lnA+=(r.st!=null?r.st.split('\n').length:0);
      lnB=rr.ln;
      rr.rows.forEach(function(rw){ linii.push({cls:'il-same', lnLeft:rw.ln, lnRight:rw.ln, mk:' ', content:rw.content}); });
    } else if(r.tip==='sters'){
      var rs=_ilLinii(r.st, lnA, null); lnA=rs.ln;
      rs.rows.forEach(function(rw){ linii.push({cls:'il-rem', lnLeft:rw.ln, lnRight:null, mk:'-', content:rw.content}); });
    } else if(r.tip==='adaugat'){
      var ra=_ilLinii(r.dr, lnB, null); lnB=ra.ln;
      ra.rows.forEach(function(rw){ linii.push({cls:'il-add', lnLeft:null, lnRight:rw.ln, mk:'+', content:rw.content}); });
    } else { // modificat: randul sursa (-) apoi randul tinta (+), cu word-diff
      var wa=r.ss?_dsLineRanges(r.ss):null, wb=r.ds?_dsLineRanges(r.ds):null;
      var rm=_ilLinii(r.st, lnA, wa); lnA=rm.ln;
      var rp=_ilLinii(r.dr, lnB, wb); lnB=rp.ln;
      rm.rows.forEach(function(rw){ linii.push({cls:'il-mod-a', lnLeft:rw.ln, lnRight:null, mk:'-', content:rw.content}); });
      rp.rows.forEach(function(rw){ linii.push({cls:'il-mod-b', lnLeft:null, lnRight:rw.ln, mk:'+', content:rw.content}); });
    }
    if(!linii.length)return;

    var ghidCur=(_ghid && _ghid.cur===r.i);
    var clickH=difera?' onclick="selectBloc('+r.i+')"':'';
    var gutterHtml='';
    if(difera){
      gutterHtml='<div class="il-gut-wrap"><span class="il-poz">'+_difOrd[r.i]+'/'+_difTot+'</span>'
        +'<span class="il-gut-btns">'
        +'<button class="il-arrow" onclick="event.stopPropagation();ilPrevHide();aplicaBlocDirect('+r.i+')" '
        +'onmouseenter="ilPrevShow(event,'+r.i+')" onmouseleave="ilPrevHide()" '
        +'title="Aplica sursa peste tinta (vezi preview la hover)">&rarr;</button>'
        +'<button class="il-eye" onclick="event.stopPropagation();ilPrevHide();deschidePv('+r.i+')" '
        +'title="Previzualizare marita, side-by-side, cu evidentiere pe cuvinte">&#128065;</button>'
        +'</span></div>';
    }
    linii.forEach(function(rw, k){
      var firstCls=(k===0)?' il-blk-first':'';
      var gutterCell=(k===0)?('<td class="il-gut" rowspan="'+linii.length+'">'+gutterHtml+'</td>') : '';
      h+='<tr class="'+rw.cls+(ghidCur?' il-ghid-cur':'')+firstCls+'" data-i="'+r.i+'" data-dif="'+(difera?1:0)+'"'+clickH+'>'
        +'<td class="il-ln">'+(rw.lnLeft!=null?rw.lnLeft:'')+'</td>'
        +'<td class="il-ln">'+(rw.lnRight!=null?rw.lnRight:'')+'</td>'
        +gutterCell
        +'<td class="il-div"></td>'
        +'<td class="il-code"><span class="il-mk">'+rw.mk+'</span>'+rw.content+'</td>'
        +'</tr>';
    });
    if(ghidCur && difera){
      h+=_ghidBarRowHTML(r.i, _difOrd[r.i], _difTot, INLINE_COLSPAN);
    }
  });
  body.innerHTML=(h||'<tr><td colspan="'+INLINE_COLSPAN+'" style="padding:14px;color:var(--muted)">Nicio diferenta.</td></tr>')
    +'<tr><td colspan="'+INLINE_COLSPAN+'" style="padding:0;border-top:1px dashed var(--border2)"><span class="eof-marker eof-tinta" style="margin:14px 14px;border-top:0;padding-top:0">'+
      '<i class="ti ti-circle-check-filled" aria-hidden="true"></i>Sfarsit procedura</span></td></tr>';
  renderIcons(body);
  if(wrap)wrap.scrollTop=prevScroll;
}


function _scurt(t){ var s=(t||'').replace(/\s+/g,' ').trim(); return s.length>60?s.slice(0,60)+'…':s; }
function insereazaLaCursor(ta,text){
  var start=ta.selectionStart, end=ta.selectionEnd, val=ta.value;
  var pre=val.slice(0,start), sep=(pre==='' || pre.endsWith('\n'))?'':'\n', ins=sep+text;
  ta.value=pre+ins+val.slice(end);
  ta.selectionStart=ta.selectionEnd=start+ins.length; ta.focus();
}


// ── EDITARE LIVE (dreapta) ────────────────────────────────────────────────────
// Principiu anti-decalaj: stratul de highlight randeaza EXACT textul din textarea,
// linie cu linie, caracter cu caracter. Nu reconstruim din blocuri (aia cauza
// decalajul). In modul "curat" = doar colorare SQL. In modul "diferente" = in plus
// fundal pe liniile schimbate fata de sursa (diff pe linii, aliniat perfect).
var modDif=true;   // true = arata diferentele (default), false = curat
var _liniiDif={};   // {nrLinie: 'mod'|'adus'|'user'}
var _liniiWord={};  // {nrLinie: [[start,end,tip?],...]} - intervale de caractere diferite
var _liniiGrp={};   // {nrLinie: r.i} - id-ul stabil de bloc, pt granite intre blocuri adiacente de acelasi tip
var _grpRank={};    // {r.i: rang 0-based} - ordinea blocului printre toate grupurile afisate
var _grpTotal=0;    // numarul total de grupuri afisate
var _grpTip={};     // {r.i: {txt, cls}} - eticheta de status (MODIFICAT/ADUS/DOAR TINTA) pt panoul tinta

function _arataSalvat(){
  var el=document.getElementById('savedTs');
  if(!el)return;
  var d=new Date();
  var hh=(''+d.getHours()).padStart(2,'0'), mm=(''+d.getMinutes()).padStart(2,'0'), ss=(''+d.getSeconds()).padStart(2,'0');
  el.textContent='Salvat la '+hh+':'+mm+':'+ss;
}
function onEdit(){
  pinteazaHL(); autoResize();
  if(_debId)clearTimeout(_debId);
  // recalcDiff() apeleaza pinteazaHL() la final - nu-l mai dublam aici
  _debId=setTimeout(function(){
    _debId=null;recalcDiff();
    if(_ghid){ ghidArata(); } else { renderSursa(); }
    _arataSalvat();
  },250);
}
function resyncDiff(){
  if(_debId){clearTimeout(_debId);_debId=null;}
  recalcDiff();
  if(_ghid){ ghidArata(); } else { renderSursa(); }
}
function toggleModDif(){
  modDif=!modDif;
  document.getElementById('modIco').className=modDif?'ti ti-eye-off':'ti ti-eye'; renderIcons();
  document.getElementById('modTxt').textContent=modDif?'Ascunde diferentele':'Arata diferentele';
  document.getElementById('btnModDif').classList.toggle('grn',modDif);
  if(modDif)recalcDiff();
  pinteazaHL();
  renderSursa();
}
function pinteazaHL(){
  var ta=document.getElementById('editTA');
  var linii=ta.value.split('\n'), parti=[];
  for(var i=0;i<linii.length;i++){
    var linie=linii[i], tip=_liniiDif[i];
    var ln='<span class="ln">'+(i+1)+'</span>';
    var mk=tip?'<span class="mk"></span>':'';
    var grpFirst=tip&&_liniiGrp[i-1]!==_liniiGrp[i], grpLast=tip&&_liniiGrp[i+1]!==_liniiGrp[i];
    var esteFocus=(_tintaFocus&&i>=_tintaFocus.s&&i<_tintaFocus.e);
    var focusFirst=esteFocus&&i===_tintaFocus.s, focusLast=esteFocus&&i===_tintaFocus.e-1;
    var rank=(_liniiGrp[i]!=null)?_grpRank[_liniiGrp[i]]:null;
    var altCls=(rank!=null && rank%2===1)?' grp-alt':'';
    var gtip=(_liniiGrp[i]!=null)?_grpTip[_liniiGrp[i]]:null;
    var hasTag=(grpFirst && gtip);
    var cls='hl-line'+(tip?(' d-'+tip):'')+(grpFirst?' blk-first':'')+(grpLast?' blk-last':'')
      +altCls+(hasTag?' has-tag':'')+(esteFocus?' t-focus':'')+(focusFirst?' tf-first':'')+(focusLast?' tf-last':'');
    var badge=(grpFirst && rank!=null)?('<span class="grp-badge">'+(rank+1)+'/'+_grpTotal+'</span>'):'';
    var statusTag=hasTag?('<span class="grp-tag '+gtip.cls+altCls+'">'+gtip.txt+'</span>'):'';
    var cont;
    if(tip==='adus'){
      cont=hlSQL(linie);
    } else if(modDif && tip && _liniiWord[i] && _liniiWord[i].length){
      cont=_hlCuWord(linie, _liniiWord[i], tip);
    } else { cont=hlSQL(linie); }
    if(cont==='')cont='&#8203;';
    var grpAttr=(_liniiGrp[i]!=null)?(' data-i="'+_liniiGrp[i]+'"'):'';
    parti.push('<span class="'+cls+'"'+grpAttr+'>'+ln+mk+badge+cont+statusTag+'</span>');
  }
  document.getElementById('editHLInner').innerHTML=parti.join('');
  actualizaDiffMap(linii.length); syncHL();
}
function _hlCuWord(linie, ranges, tip){
  ranges=ranges.slice().sort(function(a,b){return a[0]-b[0];});
  var out='', pos=0;
  for(var k=0;k<ranges.length;k++){
    var s=Math.max(pos,ranges[k][0]), e=ranges[k][1];
    if(e<=s)continue;
    var t=ranges[k][2]||tip;
    var wcls='w-'+(t==='del'?'del':t==='add'?'add':t==='mod'?'mod':t==='adus'?'adus':'user');
    if(s>pos)out+=hlSQL(linie.slice(pos,s));
    out+='<span class="wdiff '+wcls+'">'+hlSQL(linie.slice(s,e))+'</span>';
    pos=e;
  }
  if(pos<linie.length)out+=hlSQL(linie.slice(pos));
  return out;
}
function actualizaDiffMap(nrLinii){
  var map=document.getElementById('diffMap'); if(!map)return;
  var chei=Object.keys(_liniiDif);
  if(!chei.length||!nrLinii){map.innerHTML='';return;}
  var h='';
  chei.forEach(function(k){
    var pct=(Number(k)/nrLinii)*100;
    h+='<i class="'+_liniiDif[k]+'" style="top:'+pct.toFixed(2)+'%"></i>';
  });
  map.innerHTML=h;
}
function autoResize(){
  var ta=document.getElementById('editTA'), hl=document.getElementById('editHL');
  ta.style.height='auto'; hl.style.height='auto';
  var h=Math.max(ta.scrollHeight, hl.scrollHeight);
  ta.style.height=h+'px'; hl.style.height=h+'px';
}
function syncHL(){
  var ta=document.getElementById('editTA'), inner=document.getElementById('editHLInner');
  inner.style.transform='translate(-'+ta.scrollLeft+'px,-'+ta.scrollTop+'px)';
}
function _dsLineRanges(ds){
  var lines=[[]], col=0, li=0;
  ds.forEach(function(t){
    var text=t[0], type=t[1], i=0;
    while(i<text.length){
      var nl=text.indexOf('\n', i);
      var segEnd = nl===-1?text.length:nl;
      var seg=text.slice(i,segEnd);
      if(type && seg.length) lines[li].push([col,col+seg.length,type]);
      col+=seg.length;
      if(nl!==-1){ li++; lines[li]=lines[li]||[]; col=0; i=nl+1; } else { i=segEnd; }
    }
  });
  return lines;
}

function _esteAdus(dr){
  if(!dr)return false;
  var n=_norm(dr);
  for (var k in _blocuriAduse){ if (n.indexOf(k)>=0) return true; }
  return false;
}
// Cand un bloc "DOAR SURSA" ar trebui inserat exact la granita dintre doua
// blocuri deja existente in tinta (zero linii goale intre ele), nu avem cum
// sa aratam clar slotul de inserare fara sa se suprapuna peste text real -
// asa ca, in loc de un artificiu vizual, adaugam un rand gol REAL in text,
// o singura data (idempotent: verificam intai daca spatiul exista deja).
// Pastreaza pozitia cursorului daca utilizatorul scrie chiar atunci.
function _asigurareSpatiiPentruInserari(){
  var ta=document.getElementById('editTA');
  var cur=ta.value;
  var dif=diff_aliniat(_srcText, cur);
  var blocuriDr=_split_statements(cur);
  var starts=_liniiStartBlocuri(cur, blocuriDr);
  var linii=cur.split('\n');
  var candidati=[];
  dif.forEach(function(el){
    if(el.tip!=='sters') return;
    var inainte=0;
    for(var k=0;k<dif.length;k++){ if(dif[k].i===el.i) break; if(dif[k].dr!=null) inainte++; }
    var idxT=Math.min(inainte, blocuriDr.length);
    var lstart=(idxT<starts.length)?starts[idxT]:linii.length;
    if(lstart<=0 || lstart>=linii.length) return;   // capat de document - nimic de despartit
    if(linii[lstart-1].trim()==='') return;          // deja exista spatiu - idempotent, nu mai facem nimic
    candidati.push(lstart);
  });
  if(!candidati.length) return false;
  var prefixLen=[0];
  for(var i=0;i<linii.length;i++) prefixLen.push(prefixLen[i]+linii[i].length+1);
  var selS=ta.selectionStart, selE=ta.selectionEnd;
  var uniceOrdonate=Array.from(new Set(candidati)).sort(function(a,b){return b-a;});
  var deltaS=0, deltaE=0;
  uniceOrdonate.forEach(function(lstart){
    var off=prefixLen[lstart];
    if(off<=selS) deltaS++;
    if(off<=selE) deltaE++;
    linii.splice(lstart,0,'');
  });
  ta.value=linii.join('\n');
  try{ ta.selectionStart=selS+deltaS; ta.selectionEnd=selE+deltaE; }catch(e){}
  return true;
}
function recalcDiff(){
  _asigurareSpatiiPentruInserari();
  var cur=document.getElementById('editTA').value;
  DIF_EL=diff_aliniat(_srcText, cur);
  DIF=[]; DIF_EL.forEach(function(e){ if(e.tip!=='egal')DIF.push(e.i); });
  var mod=DIF_EL.filter(function(e){return e.tip==='modificat';}).length;
  var add=DIF_EL.filter(function(e){return e.tip==='adaugat';}).length;
  var st =DIF_EL.filter(function(e){return e.tip==='sters';}).length;
  var scor=scor_similaritate(_srcText,cur);
  document.getElementById('infMsg').textContent=
    scor+'% similar · '+mod+' mod, '+add+' add, '+st+' sterse';
  document.getElementById('aplCnt').textContent=(mod+add);
  document.getElementById('totCnt').textContent=(mod+add+st);
  actualizaNav();
  actualizeazaRezCnt();

  _liniiDif={}; _liniiWord={}; _liniiGrp={}; _grpRank={}; _grpTotal=0; _grpTip={};
var pointer=0, lineIdx=0;
DIF_EL.forEach(function(r){
  if(r.dr==null) return;
  var idx=cur.indexOf(r.dr, pointer);
  if(idx<0) idx=pointer;
  for(var p=pointer;p<idx;p++){ if(cur[p]==='\n') lineIdx++; }
  pointer=idx;
  var blocLinii=r.dr.split('\n');
  var esteGrup=false;

  if(_esteAdus(r.dr)){
    esteGrup=true;
    if(_grpTip[r.i]===undefined) _grpTip[r.i]={txt:'ADUS DIN SURSA',cls:'tag-adus'};
    blocLinii.forEach(function(ln,k){
      var gl=lineIdx+k;
      _liniiDif[gl]='adus'; _liniiGrp[gl]=r.i;
      _liniiWord[gl]=[[0, ln.length,'adus']];
    });
  } else if(r.tip==='modificat'){
    esteGrup=true;
    if(_grpTip[r.i]===undefined) _grpTip[r.i]={txt:'MODIFICAT',cls:'tag-mod'};
    var ranges=r.ds?_dsLineRanges(r.ds):[];
    blocLinii.forEach(function(ln,k){
      var gl=lineIdx+k;
      var lr=ranges[k]||[];
      _liniiDif[gl]='mod'; _liniiGrp[gl]=r.i;
      if(lr.length)_liniiWord[gl]=lr;
    });
  } else if(r.tip==='adaugat'){
    esteGrup=blocLinii.some(function(ln){return _normLinie(ln)!=='';});
    if(esteGrup && _grpTip[r.i]===undefined) _grpTip[r.i]={txt:'DOAR TINTA',cls:'tag-user'};
    blocLinii.forEach(function(ln,k){
      var gl=lineIdx+k;
      if(_normLinie(ln)!=='') { _liniiDif[gl]='user'; _liniiGrp[gl]=r.i; }
      _liniiWord[gl]=[[0, ln.length]];
    });
  } else if(r.tip==='egal'){
    // fara evidentiere - doar ancora (data-i) pt sincronizarea de scroll,
    // ca sa existe repere dese pe tot fisierul, nu doar langa diferente
    blocLinii.forEach(function(ln,k){ _liniiGrp[lineIdx+k]=r.i; });
  }
  if(esteGrup && _grpRank[r.i]===undefined){ _grpRank[r.i]=_grpTotal++; }

  pointer=idx+r.dr.length;
  lineIdx += blocLinii.length-1;
});
pinteazaHL();
}
function _tokenize(s){
  var re=/\s+|[A-Za-z_@#]\w*|[^\sA-Za-z_@#]/g, t=[], m;
  while((m=re.exec(s))!==null){ t.push(m[0]); if(re.lastIndex===m.index)re.lastIndex++; }
  return t;
}
function _wordRanges(srcLn, curLn){
  var A=_tokenize(srcLn), B=_tokenize(curLn);
  var na=A.map(function(x){return x.toLowerCase();});
  var nb=B.map(function(x){return x.toLowerCase();});
  var ops=new SequenceMatcher(na, nb).get_opcodes();
  var ranges=[], start=[], acc=0;
  for(var k=0;k<B.length;k++){ start.push(acc); acc+=B[k].length; }
  ops.forEach(function(o){
    var tag=o[0],j1=o[3],j2=o[4];
    if(tag!=='equal' && j2>j1){
      var s=start[j1], e=start[j2-1]+B[j2-1].length;
      ranges.push([s,e]);
    }
  });
  return ranges;
}
function _normLinie(l){ return l.replace(/\s+/g,' ').trim().toLowerCase(); }
var DIF_EL=[];

// construieste `adaugat` pt deployer din starea curenta a editorului.
// Model: comparam textul editat cu sursa; deployer-ul va folosi textul final direct.
function construiesteAdaugat(){
  // Trimitem textul final integral sub cheia speciala '__FINAL__' - deployer-ul
  // il foloseste ca atare (vezi app.py). Compatibil cu vechea logica pe blocuri.
  var cur=document.getElementById('editTA').value;
  return {'__FINAL__': cur};
}

// ── HIGHLIGHT SQL (fara librarii externe) ─────────────────────────────────────
var _KW_RE=/^(?:SELECT|INSERT|UPDATE|DELETE|FROM|WHERE|JOIN|LEFT|RIGHT|INNER|OUTER|FULL|CROSS|APPLY|ON|AND|OR|NOT|IN|IS|NULL|AS|BY|ORDER|GROUP|HAVING|UNION|ALL|DISTINCT|TOP|DECLARE|SET|IF|ELSE|BEGIN|END|RETURN|WHILE|BREAK|CONTINUE|EXEC|EXECUTE|CREATE|ALTER|DROP|TABLE|PROCEDURE|FUNCTION|VIEW|TRIGGER|WITH|NOLOCK|INTO|VALUES|EXISTS|CASE|WHEN|THEN|ELSE|CONVERT|CAST|ISNULL|COALESCE|VARCHAR|NVARCHAR|INT|BIGINT|SMALLINT|TINYINT|DATETIME|DATE|BIT|CHAR|NCHAR|DECIMAL|NUMERIC|FLOAT|MONEY|XML|MERGE|OVER|PARTITION|GO|PRINT|OUT|OUTPUT|OPENXML|MAX)$/i;
function _esc1(c){ return c==='&'?'&amp;':c==='<'?'&lt;':c==='>'?'&gt;':c; }
function _escTok(t){ return t.replace(/[&<>]/g,_esc1); }
// tokenizare intr-o SINGURA trecere: nu reprocesam niciodata HTML deja emis,
// deci nu mai exista scurgeri de stiluri (bug-ul cu "008000;font-style...").
function hlSQL(cod){
  if(cod==null)return'';
  var isLight=(temaCur==='github');
  var cCom=isLight?'#3f8f3f':'#5a7a9a';
  var cStr=isLight?'#a31515':'#ce9178';
  var cNum=isLight?'#098658':'#b5cea8';
  // regex global cu alternative in ordine de prioritate
  var re=/(--[^\n]*)|(\/\*[\s\S]*?\*\/)|('(?:''|[^'])*')|(@@?\w+)|(\b\d+(?:\.\d+)?\b)|([A-Za-z_]\w*)|(\s+)|([\s\S])/g;
  var out='', m;
  while((m=re.exec(cod))!==null){
    if(m[1]!=null||m[2]!=null){ out+='<span style="color:'+cCom+';font-style:italic">'+_escTok(m[1]!=null?m[1]:m[2])+'</span>'; }
    else if(m[3]!=null){ out+='<span style="color:'+cStr+'">'+_escTok(m[3])+'</span>'; }
    else if(m[4]!=null){ out+='<span class="v">'+_escTok(m[4])+'</span>'; }
    else if(m[5]!=null){ out+='<span style="color:'+cNum+'">'+m[5]+'</span>'; }
    else if(m[6]!=null){ out+= _KW_RE.test(m[6]) ? '<span class="kw">'+m[6]+'</span>' : _escTok(m[6]); }
    else if(m[7]!=null){ out+=m[7]; }
    else { out+=_escTok(m[8]); }
    if(re.lastIndex===m.index)re.lastIndex++;   // garda anti-bucla
  }
  return out;
}
function esc(s){return (s==null?'':String(s)).replace(/&/g,'&amp;').replace(/</g,'&lt;').replace(/>/g,'&gt;');}

// ── TOOLBAR ACTIUNI ───────────────────────────────────────────────────────────
function _clearIstoricPlasari(){
  _istoricPlasari=[];
  var btnU=document.getElementById('btnUndoPlasare');
  if(btnU)btnU.style.display='none';
}
function adaugaTot(){
  // aduce toate blocurile din sursa in textul editat (rezultatul = sursa completa)
  document.getElementById('editTA').value=_srcText;
  _clearIstoricPlasari();
  if(_ghid){ _ghid.aplicate+=_ghidQueue().length; arataGhidDone(); }
  resyncDiff();autoResize();
  marcheazaRezolvat(true);
}
function resetTot(){
  // revine la textul original al bazei tinta
  var textTinta=EL.map(function(e){return e.dr;}).filter(function(x){return x!=null;}).join('\n');
  document.getElementById('editTA').value=textTinta;
  _clearIstoricPlasari();
  if(_ghid)iesiGhidat();
  resyncDiff();autoResize();
}
var _liniiNav=[];  // liniile schimbate, in ordine, pt navigare
function actualizaNav(){
  _liniiNav=Object.keys(_liniiDif).map(Number).sort(function(a,b){return a-b;});
  document.getElementById('navI').textContent=_liniiNav.length?(Math.min(nav+1,_liniiNav.length))+'/'+_liniiNav.length:'0/0';
}
function scrollLaDif(k){
  if(!_liniiNav.length)return;
  nav=Math.max(0,Math.min(_liniiNav.length-1,k));
  actualizaNav();
  var linie=_liniiNav[nav];
  var ta=document.getElementById('editTA');
  var lineH=parseFloat(getComputedStyle(ta).lineHeight)||19;
  var sc=document.getElementById('scWork');
  sc.scrollTop=Math.max(0,linie*lineH-sc.clientHeight/2);
  // daca nu suntem in mod diferente, aprindem temporar sa se vada unde
  if(!modDif){toggleModDif();}
}
function navDif(d){ if(!_liniiNav.length)return; scrollLaDif((nav||0)+d); }
function toggleEgale(){ /* pastrat pt compat */ }

// ── RESIZE COLOANE (trage bara dintre SURSA si BAZA) ──
// ── COPIERE ───────────────────────────────────────────────────────────────────
function _copie(text,eticheta){
  function _feedback(){
    var n=text.trim()?text.trim().split('\n').length:0;
    var el=document.getElementById('infMsg');var vechi=el.textContent;
    el.textContent='✓ '+eticheta+' copiat ('+n+' linii)';
    setTimeout(function(){el.textContent=vechi;},1500);
  }
  function _legacy(){
    var ta=document.createElement('textarea');ta.value=text;
    document.body.appendChild(ta);ta.select();
    try{document.execCommand('copy');}catch(x){}
    document.body.removeChild(ta);
    _feedback();
  }
  if(navigator.clipboard&&navigator.clipboard.writeText){
    navigator.clipboard.writeText(text).then(_feedback).catch(_legacy);
  } else { _legacy(); }
}
function copiazaSursa(){ _copie(_srcText,'Sursa'); }
function copiazaRezultat(){ _copie(document.getElementById('editTA').value,'Rezultat'); }

// ── DEPLOYER + EXPORT ─────────────────────────────────────────────────────────
function genereazaDeployer(){
  var bif=LISTA.filter(function(d){return d.rezolvat;});
  if(!bif.length){alert('Niciun obiect marcat ca rezolvat.');return;}
  document.getElementById('depOverlay').classList.add('on');
  actualizeazaDepInfo();
}
function _depSelectie(){
  // selectia REALA care ajunge in deployer: rezolvate, fara "extra"
  // (backend-ul le sare oricum), filtrate dupa optiunea "deja deployate"
  var incl=document.getElementById('depInclDeployate').checked;
  return LISTA.filter(function(d){return d.rezolvat&&d.cat!=='extra'&&(incl||!d.deployat);});
}
function actualizeazaDepInfo(){
  var el=document.getElementById('depSumar'); if(!el)return;
  var toate=LISTA.filter(function(d){return d.rezolvat;});
  var extra=toate.filter(function(d){return d.cat==='extra';}).length;
  var sel=_depSelectie();
  var dejaDep=toate.filter(function(d){return d.deployat&&d.cat!=='extra';}).length;
  var incl=document.getElementById('depInclDeployate').checked;
  var perBaza={}; sel.forEach(function(d){perBaza[d.baza]=(perBaza[d.baza]||0)+1;});
  var nrBaze=Object.keys(perBaza).length;
  var mod=(document.querySelector('input[name=depmod]:checked')||{}).value||'obiect';
  var n=Math.max(1,parseInt(document.getElementById('depN').value,10)||1);
  var fisiere=0;
  if(mod==='obiect')fisiere=sel.length;
  else if(mod==='baza')fisiere=nrBaze;
  else if(mod==='unul')fisiere=sel.length?1:0;
  else Object.keys(perBaza).forEach(function(b){fisiere+=Math.ceil(perBaza[b]/n);});
  var h='Se genereaza: <b>'+sel.length+'</b> obiecte, <b>'+nrBaze+'</b> baze &rarr; <b>'+fisiere+'</b> fisier(e) .sql';
  var note=[];
  if(dejaDep) note.push(incl?(dejaDep+' fusesera deja deployate (incluse din nou)'):(dejaDep+' deja deployate, excluse'));
  if(extra) note.push(extra+' obiecte "extra" sarite (nu au sursa in referinta)');
  if(!sel.length) note.push('nimic de generat cu setarile curente');
  if(note.length)h+='<br><span style="color:var(--muted)">'+note.join(' &middot; ')+'</span>';
  el.innerHTML=h;
}
function inchideDeployModal(){ document.getElementById('depOverlay').classList.remove('on'); }
function confirmaDeploy(){
  var mod=(document.querySelector('input[name=depmod]:checked')||{}).value||'obiect';
  var n=parseInt(document.getElementById('depN').value,10)||10;
  var bif=_depSelectie().map(function(d){return{id:d.id,adaugat:d.adaugat||{}};});
  if(!bif.length){alert('Nimic de generat cu setarile curente.\nBifeaza "Include si ce a fost deja trimis la deploy" sau marcheaza obiecte ca rezolvate.');return;}
  inchideDeployModal();
  pywebview.api.genereaza_deployer(bif,mod,n).then(function(r){
    if(r.ok){
      var acum=new Date().toISOString();
      var idsDeployate={};
      bif.forEach(function(b){idsDeployate[b.id]=true;});
      LISTA.forEach(function(d){ if(idsDeployate[d.id]) d.deployat=acum; });
      renderLista();
      salveazaSesiune();
      var msg='Deployer salvat!\n'+r.cale+'\n\n'+r.nr+' obiecte in '+r.fisiere+' fisier(e).';
      if(r.sarite)msg+='\n('+r.sarite+' obiecte "extra" sarite - nu au corespondent in referinta)';
      alert(msg);
    }
    else if(r.anulat){/* Cancel */}
    else{alert('Eroare: '+r.mesaj);}
  }).catch(function(e){alert('Eroare la generarea deployer-ului: '+String(e));});
}
function exportaCSV(){
  var rows=_rows.map(function(d){return{obiect:d.obiect,baza:d.baza,tip:TIP_NUME[d.tip]||d.tip,
    cat:d.cat,scor:d.scor,rezumat:d.rezumat,rezolvat:!!d.rezolvat};});
  if(!rows.length){alert('Lista filtrata e goala - nimic de exportat.');return;}
  pywebview.api.exporta_csv(rows).then(function(r){
    if(r.ok){/* salvat cu succes */}
    else if(r.anulat){/* Cancel - fara mesaj */}
    else{alert('Eroare export: '+r.mesaj);}
  }).catch(function(e){alert('Eroare export: '+String(e));});
}

// ── SCURTATURI TASTATURA ─────────────────────────────────────────────────────
document.addEventListener('keydown',function(e){
  // modal previzualizare deschis: in mod ghidat accepta acelasi Enter/Esc ca bara ghidata
  // (doar daca nu se scrie chiar acum in caseta de rezultat final editabila)
  var pvOv=document.getElementById('pvOverlay');
  if(pvOv && pvOv.classList.contains('on')){
    if((e.key==='z'||e.key==='Z')&&(e.ctrlKey||e.metaKey)&&_pvIstoric.length){
      e.preventDefault(); _pvUndo(); return;
    }
    var seScrieInFinal=(document.activeElement===document.getElementById('pvFinal') && !document.getElementById('pvFinal').readOnly);
    if(!seScrieInFinal && _ghid){
      if(e.key==='Enter'){e.preventDefault();document.getElementById('pvAplica').click();return;}
      if(e.key==='Escape'){
        e.preventDefault();
        var ps=document.getElementById('pvSari');
        if(ps && ps.style.display!=='none'){ ps.click(); } else { inchidePv(); }
        return;
      }
    }
    if(e.key==='Escape'){e.preventDefault();inchidePv();}
    return;
  }
  var gdOv=document.getElementById('ghidDone');
  if(gdOv && gdOv.classList.contains('on')){
    if(e.key==='Escape'||e.key==='Enter'){e.preventDefault();inchideGhidDone();}
    return;
  }
  var offOv=document.getElementById('offOverlay');
  if(offOv && offOv.classList.contains('on')){
    if(e.key==='Escape'){e.preventDefault();inchideOfflineModal();}
    return;
  }
  // mod ghidat: Enter = aplica, Esc = sari peste, Sus/Jos = navigare
  if(typeof _ghid!=='undefined' && _ghid && document.getElementById('ed').classList.contains('active')
     && document.activeElement!==document.getElementById('editTA')){
    if(e.key==='Enter'){e.preventDefault();ghidAplica();return;}
    if(e.key==='Escape'){e.preventDefault();ghidSari();return;}
    if((e.key==='ArrowDown'||e.key==='ArrowRight')&&!e.altKey){e.preventDefault();ghidUrm(1);return;}
    if((e.key==='ArrowUp'||e.key==='ArrowLeft')&&!e.altKey){e.preventDefault();ghidUrm(-1);return;}
  }
  // Esc: inchide dropdown tema, apoi detaliu -> lista
  if(e.key==='Escape'){
    if(_plc){anuleazaPlasare();return;}
    if(_selBloc!==null){_selBloc=null;_ascundePreviewDr();renderSursa();return;}
    var dropDeschis=document.querySelector('.tema-dropdown.open');
    if(dropDeschis){inchideTemele();return;}
    if(document.getElementById('ed').classList.contains('active')){inapoiLista();return;}
    return;
  }
  // Ctrl+Z: anuleaza ultima plasare de bloc (doar daca exista istoric - altfel
  // lasam undo-ul nativ al textarei sa functioneze normal pentru text tastat)
  if((e.ctrlKey||e.metaKey)&&!e.shiftKey&&e.key.toLowerCase()==='z'&&document.getElementById('ed').classList.contains('active')&&typeof _istoricPlasari!=='undefined'&&_istoricPlasari.length){
    e.preventDefault();anuleazaUltimaPlasare();return;
  }
  // Ctrl+F: focus pe cautare cand lista e activa
  if((e.ctrlKey||e.metaKey)&&e.key.toLowerCase()==='f'&&document.getElementById('el').classList.contains('active')){
    e.preventDefault();document.getElementById('fNume').focus();document.getElementById('fNume').select();return;
  }
  // Alt+Sus/Jos: navigare intre diferente in ecranul de detaliu
  if(e.altKey&&document.getElementById('ed').classList.contains('active')){
    if(e.key==='ArrowDown'){e.preventDefault();navDif(1);}
    else if(e.key==='ArrowUp'){e.preventDefault();navDif(-1);}
    else if(e.key==='ArrowRight'){e.preventDefault();navObiect(1);}
    else if(e.key==='ArrowLeft'){e.preventDefault();navObiect(-1);}
  }
  // PageUp/PageDown: navigare intre obiecte in ecranul de detaliu
  if(document.getElementById('ed').classList.contains('active')){
    if(e.key==='PageDown'){e.preventDefault();navObiect(1);}
    else if(e.key==='PageUp'){e.preventDefault();navObiect(-1);}
  }
});

// ── SCROLL SYNC DETALIU ───────────────────────────────────────────────────────
var _syncBound=false,_syncLock=false;
function syncScrollDet(){
  if(_syncBound)return;            // ataseaza O SINGURA DATA (nu stivui listeneri)
  _syncBound=true;
  var a=document.getElementById('scSrc'),b=document.getElementById('scWork');
  if(!a||!b)return;
  function leaga(sursa,tinta){
    sursa.addEventListener('scroll',function(){
      if(_syncLock)return;         // suntem in mijlocul unei sincronizari -> ignora
      _syncLock=true;
      tinta.scrollTop=sursa.scrollTop;
      // elibereaza lacatul dupa ce evenimentul de scroll reflectat s-a consumat
      requestAnimationFrame(function(){_syncLock=false;});
    },{passive:true});
  }
  leaga(a,b); leaga(b,a);
}


// ── TERMINAL BACKEND ────────────────────────────────────────────────────────
var _termLogIdx=0;
var _termAutoScroll=true;

function termInit(){
  _termLogIdx=0;_termAutoScroll=true;
  var body=document.getElementById('termBody');
  body.innerHTML='';
  body.onscroll=function(){_termAutoScroll=(body.scrollTop+body.clientHeight>=body.scrollHeight-4);};
}
function termPollLog(){
  if(typeof pywebview==='undefined'||!pywebview.api.log_lines)return;
  pywebview.api.log_lines(_termLogIdx).then(function(r){
    if(!r||!r.linii||!r.linii.length)return;
    r.linii.forEach(function(linie){termAppend(linie,_termCls(linie));});
    _termLogIdx=r.total;
  }).catch(function(){});
}
function _termCls(linie){
  var l=linie.toLowerCase();
  if(l.indexOf('eroare')>=0||l.indexOf('error')>=0)return 't-err';
  if(l.indexOf('anulat')>=0||l.indexOf('warn')>=0)return 't-warn';
  if(l.indexOf('gata')>=0||l.indexOf('succes')>=0||l.indexOf('conectat')>=0)return 't-ok';
  return 't-info';
}
var _termQueue=[];
var _termTyping=false;
function _splitTs(text){
  var m=/^(\[\d{1,2}:\d{2}:\d{2}\])(\s*)([\s\S]*)$/.exec(text);
  if(m) return {ts:m[1],sep:m[2],rest:m[3]};
  return {ts:'',sep:'',rest:text};
}
function _termClearEmpty(body){
  var gol=body.querySelector('.term-empty');if(gol)gol.remove();
}
function termAppend(linie,cls){
  var body=document.getElementById('termBody');if(!body)return;
  _termQueue.push({linie:linie,cls:cls||'t-info'});
  // daca se aduna prea multe linii in coada, le afisam instant (fara animatie)
  // ca sa nu ramanem in urma cu logul in timpul rularilor rapide
  while(_termQueue.length>6){ _termFlushInstant(_termQueue.shift()); }
  if(!_termTyping) _termTypeNext();
}
function _termFlushInstant(item){
  var body=document.getElementById('termBody');if(!body)return;
  _termClearEmpty(body);
  var old=body.querySelector('.term-cursor');if(old)old.remove();
  var span=document.createElement('span');
  span.className='term-line '+item.cls;
  var prefix=document.createElement('span');
  prefix.className='term-prompt';prefix.textContent='> ';
  span.appendChild(prefix);
  var parts=_splitTs(item.linie);
  if(parts.ts){
    var tsSpan=document.createElement('span');
    tsSpan.className='term-ts';tsSpan.textContent=parts.ts+parts.sep;
    span.appendChild(tsSpan);
  }
  span.appendChild(document.createTextNode(parts.rest));
  body.appendChild(span);body.appendChild(document.createTextNode('\n'));
  if(_termAutoScroll)body.scrollTop=body.scrollHeight;
}
function _termTypeNext(){
  var body=document.getElementById('termBody');
  if(!body){_termTyping=false;return;}
  if(!_termQueue.length){
    _termTyping=false;
    var old=body.querySelector('.term-cursor');if(old)old.remove();
    var cur=document.createElement('span');
    cur.className='term-cursor';cur.textContent='_';body.appendChild(cur);
    return;
  }
  _termTyping=true;
  _termClearEmpty(body);
  var old=body.querySelector('.term-cursor');if(old)old.remove();
  var item=_termQueue.shift();
  var span=document.createElement('span');
  span.className='term-line '+item.cls;
  var prefix=document.createElement('span');
  prefix.className='term-prompt';prefix.textContent='> ';
  span.appendChild(prefix);
  var parts=_splitTs(item.linie);
  if(parts.ts){
    var tsSpan=document.createElement('span');
    tsSpan.className='term-ts';tsSpan.textContent=parts.ts+parts.sep;
    span.appendChild(tsSpan);
  }
  body.appendChild(span);
  var i=0,txt=parts.rest;
  var iv=setInterval(function(){
    span.appendChild(document.createTextNode(txt.charAt(i)));
    i++;
    if(_termAutoScroll)body.scrollTop=body.scrollHeight;
    if(i>=txt.length){
      clearInterval(iv);
      body.appendChild(document.createTextNode('\n'));
      _termTypeNext();
    }
  },9);
}
function _termLegendaHTML(){
  return '<div class="term-empty term-legend">'
    +'<div class="tl-corner tl-c1"></div><div class="tl-corner tl-c2"></div><div class="tl-corner tl-c3"></div><div class="tl-corner tl-c4"></div>'
    +'<div class="tl-sec"><div class="tl-rail"><span class="tl-dot"></span><span class="tl-line"></span></div><div class="tl-body">'
      +'<div class="tl-tag">§ 01</div>'
      +'<div class="tl-title">Ce face aplicatia</div>'
      +'<div class="tl-desc">Compara structurile SQL (tabele, view-uri, proceduri, functii, triggere) dintre baza de <b>REFERINTA</b> si una sau mai multe baze <b>TINTA</b>, evidentiaza diferentele si genereaza un script de deploy pentru obiectele rezolvate.</div>'
    +'</div></div>'
    +'<div class="tl-sec"><div class="tl-rail"><span class="tl-dot"></span><span class="tl-line"></span></div><div class="tl-body">'
      +'<div class="tl-tag">§ 02</div>'
      +'<div class="tl-title">Cum completezi ecranul de conectare</div>'
      +'<div class="tl-desc">'
        +'<b>SQL Auth / Windows</b> — alege modul de autentificare: utilizator+parola sau contul Windows curent.<br>'
        +'<b>SERVER SQL</b> — scrie numele serverului (ex: <b>SERVER\\INSTANTA</b>) sau apasa <i class="ti ti-search" aria-hidden="true"></i> ca sa cauti automat serverele SQL din reteaua locala.<br>'
        +'<b>Utilizator / Parola</b> — necesare doar la SQL Auth.<br>'
        +'<b>Testeaza conexiunea</b> — verifica acreditarile si aduce lista bazelor de date disponibile pe server.<br>'
        +'<b>Alege o baza de REFERINTA</b> din lista aparuta — restul bazelor vor fi comparate fata de aceasta.<br>'
        +'<b>Porneste verificarea</b> — devine activ dupa ce ai ales referinta; incepe scanarea si compararea.'
      +'</div>'
    +'</div></div>'
    +'<div class="tl-sec"><div class="tl-rail"><span class="tl-dot"></span><span class="tl-line"></span></div><div class="tl-body">'
      +'<div class="tl-tag">§ 03</div>'
      +'<div class="tl-title">Cum functioneaza sesiunile</div>'
      +'<div class="tl-desc">'
        +'Progresul (ce e rezolvat, ce a fost trimis la deploy) se salveaza automat, <b>separat pentru fiecare baza de REFERINTA</b> — nu se suprascriu intre ele.<br>'
        +'Tab-ul <b>Sesiuni</b> arata toate sesiunile salvate; <b>Reia</b> o marcheaza ca activa si completeaza automat serverul si autentificarea; <b>Ignora</b> o sterge definitiv.<br>'
        +'Sesiunea activa apare mov, in tab-ul <b>Conectare</b>, ca sa stii clar in ce te conectezi — se poate anula oricand din acelasi banner.<br>'
        +'La <b>Deconectare</b>, progresul curent se salveaza automat si te intorci la lista de sesiuni.'
      +'</div>'
    +'</div></div>'
    +'<div class="tl-sec"><div class="tl-rail"><span class="tl-dot"></span><span class="tl-line"></span></div><div class="tl-body">'
      +'<div class="tl-tag">§ 04</div>'
      +'<div class="tl-title">Simbolurile din ecranul Lista</div>'
      +'<div class="tl-tokens">'
        +'<span class="tl-tok"><span class="d" style="background:var(--neon);box-shadow:0 0 6px var(--neon)"></span>Rezolvat</span>'
        +'<span class="tl-tok"><span class="d" style="background:transparent;border:2px solid var(--muted)"></span>Nerezolvat</span>'
        +'<span class="tl-tok"><span class="d" style="background:var(--muted);box-shadow:0 0 6px rgba(89,99,110,.4)"></span>Omis (analizat, exclus din deploy)</span>'
        +'<span class="tl-tok"><span class="d" style="background:var(--teal)"></span>Trimis la deploy</span>'
        +'<span class="tl-tok"><span class="d" style="background:var(--blue)"></span>Pregatit pentru deploy</span>'
        +'<span class="tl-tok"><span class="d" style="background:var(--amber)"></span>Versiune noua / modificat de atunci</span>'
        +'<span class="tl-tok"><span class="d" style="background:var(--muted)"></span>Neintrodus in deploy</span>'
      +'</div>'
    +'</div></div>'
    +'<div class="tl-sec"><div class="tl-rail"><span class="tl-dot"></span><span class="tl-line"></span></div><div class="tl-body">'
      +'<div class="tl-tag">§ 05</div>'
      +'<div class="tl-title">Simbolurile din Editorul de diferente</div>'
      +'<div class="tl-tokens">'
        +'<span class="tl-tok"><span class="d" style="background:var(--amber)"></span>Modificat</span>'
        +'<span class="tl-tok"><span class="d" style="background:var(--neon)"></span>Adus din sursa</span>'
        +'<span class="tl-tok"><span class="d" style="background:var(--kw)"></span>Scris de tine</span>'
      +'</div>'
    +'</div></div>'
    +'<div class="tl-sec"><div class="tl-rail"><span class="tl-dot" style="background:var(--muted);box-shadow:none"></span></div><div class="tl-body">'
      +'<div class="tl-hint">Apasa <b>Porneste verificarea</b><span class="tl-cursor"></span> — jurnalul de mai sus va fi inlocuit de activitatea reala.</div>'
    +'</div></div>'
    +'</div>';
}
function termClear(){
  var body=document.getElementById('termBody');
  if(body){body.innerHTML=_termLegendaHTML();renderIcons(body);}
  _termLogIdx=0;
  _termQueue=[];
  _termTyping=false;
}
setAuth('sql');
renderIcons();  // converteste iconitele statice din HTML la pornire
// orice DOM adaugat dinamic (randuri lista, blocuri diff) primeste automat iconite
(function(){
  var obs=new MutationObserver(function(muts){
    for(var i=0;i<muts.length;i++){
      var m=muts[i];
      for(var j=0;j<m.addedNodes.length;j++){
        var n=m.addedNodes[j];
        if(n.nodeType===1){
          if(n.matches&&n.matches('i.ti:not([data-ico])'))renderIcons(n.parentNode||document);
          else if(n.querySelector&&n.querySelector('i.ti:not([data-ico])'))renderIcons(n);
        }
      }
    }
  });
  obs.observe(document.body,{childList:true,subtree:true});
})();
window.addEventListener('resize', _cereRedrawGutter);
window.addEventListener('pywebviewready', verificaSesiuneSalvata);

</script></body></html>
"""