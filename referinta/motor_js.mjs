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
function _tokenize(s){
  var re=/\s+|[A-Za-z_@#]\w*|[^\sA-Za-z_@#]/g, t=[], m;
  while((m=re.exec(s))!==null){ t.push(m[0]); if(re.lastIndex===m.index)re.lastIndex++; }
  return t;
}
function _normLinie(l){ return l.replace(/\s+/g,' ').trim().toLowerCase(); }
export { _norm, normalizeaza, pregateste_afisare, scor_similaritate, _split_statements, _cheie_bloc, rezumat_rapid, diff_aliniat, diff_to_elemente, SequenceMatcher };
