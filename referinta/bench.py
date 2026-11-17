"""Acelasi corpus, prin motor.py. python3 -I referinta/bench.py"""
import json, os, sys, time
R = os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), "mostenire")
sys.path.insert(0, R)
import motor
A = os.path.dirname(os.path.abspath(__file__))
pairs = json.load(open(os.path.join(A, "corpus_bench.json"), encoding="utf-8"))
date = [(motor.pregateste_afisare(p["ref"]), motor.pregateste_afisare(p["cur"])) for p in pairs]

t = time.perf_counter()
for r, c in date:
    motor.scor_similaritate(r, c)
    motor.rezumat_rapid(r, c)
print(f"python scor+rezumat : {(time.perf_counter()-t)*1000:>8.0f} ms pentru {len(date)} perechi")

t = time.perf_counter()
n = sum(len(motor.diff_aliniat(r, c)) for r, c in date)
print(f"python diff_aliniat : {(time.perf_counter()-t)*1000:>8.0f} ms ({n} randuri)")
