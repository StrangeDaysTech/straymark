#!/usr/bin/env python3
"""E2 — piloto work_verb (tercera vía declarada vs. title-scan determinista).

Simula el clasificador verb-aware de la propuesta sobre las 32 unidades del sample
E1, y lo compara contra (a) la predicción real de Baton (title-scan) y (b) el ground
truth. Read-only. NO patchea Baton; modela el cambio mínimo propuesto:

    class = verb→tier; si verb∈{implement,design} y provenance=upstream ⇒ operator.
    confidence = high (el autor lo declaró).

Uso: python3 .straymark/baton-calibration/verb_pilot.py
"""
import json
import os
import re

HERE = os.path.dirname(os.path.abspath(__file__))
VERBS = os.path.join(HERE, "verbs.yml")
KEY = os.path.join(HERE, ".predictions-key.json")
LABELS = os.path.join(HERE, "labels-blind.yml")

VERB_TIER = {"design": "planner", "implement": "implementer",
             "audit": "auditor", "operate": "operator"}
RANK = {"planner": 4, "implementer": 3, "auditor": 2, "operator": 1}


def parse_verbs(path):
    out, cur = {}, None
    for raw in open(path, encoding="utf-8"):
        if raw.strip().startswith("#") or not raw.strip():
            continue
        l = raw.split(" #", 1)[0].rstrip()  # comentario inline va tras espacio; ids llevan #batch
        m = re.match(r"- id:\s*(.+)$", l)
        if m:
            cur = m.group(1).strip()
            out[cur] = {}
        elif cur and ":" in l:
            k, _, v = l.strip().partition(":")
            out[cur][k.strip()] = v.strip()
    return out


def parse_truth(path):
    out, cur = {}, None
    for raw in open(path, encoding="utf-8"):
        l = raw.rstrip("\n")
        if l.lstrip().startswith("#"):
            continue
        m = re.match(r"- id:\s*(.+)$", l)
        if m:
            cur = m.group(1).strip()
        elif cur and l.startswith("  true_class:"):
            out[cur] = l.split(":", 1)[1].split("#")[0].strip()
    return out


def verb_classify(v):
    tier = VERB_TIER[v["work_verb"]]
    if v.get("design_provenance") == "upstream" and tier in ("implementer", "planner"):
        tier = "operator"  # instrumenta diseño previo ⇒ mecánico
    return tier


def main():
    verbs = parse_verbs(VERBS)
    key = json.load(open(KEY, encoding="utf-8"))
    truth = parse_truth(LABELS)

    rows = []
    for uid, v in verbs.items():
        rows.append({
            "id": uid,
            "truth": truth[uid],
            "baton": key[uid]["predicted_class"],
            "baton_conf": key[uid]["confidence"],
            "verb": v["work_verb"],
            "prov": v.get("design_provenance", "new"),
            "verb_pred": verb_classify(v),
        })

    n = len(rows)
    print(f"Piloto work_verb — {n} unidades\n")

    # --- Exactitud: title-scan (Baton) vs verb-declared ---
    baton_ok = sum(r["truth"] == r["baton"] for r in rows)
    verb_ok = sum(r["truth"] == r["verb_pred"] for r in rows)
    print("EXACTITUD GLOBAL")
    print(f"  Baton (title-scan):   {baton_ok}/{n} = {baton_ok/n:.2f}")
    print(f"  Verb declarado:       {verb_ok}/{n} = {verb_ok/n:.2f}")

    # --- high+medium decisiva (Baton) vs verb (todo high por declaración) ---
    hm = [r for r in rows if r["baton_conf"] in ("high", "medium")]
    hm_ok = sum(r["truth"] == r["baton"] for r in hm)
    print("\nPRECISIÓN high+medium (la cifra decisiva)")
    print(f"  Baton:  {hm_ok}/{len(hm)} = {hm_ok/len(hm):.2f}")
    print(f"  Verb:   {verb_ok}/{n} = {verb_ok/n:.2f}  (todas high por declaración)")

    # --- Confianza: el lever de #328 ---
    from collections import Counter
    bc = Counter(r["baton_conf"] for r in rows)
    print("\nCONFIANZA (lever #328)")
    print(f"  Baton (inferida):  high {bc['high']} · medium {bc['medium']} · low {bc['low']}")
    print(f"  Verb (declarada):  high {n} · medium 0 · low 0")

    # --- Errores hacia abajo (peligrosos) ---
    def down(pred_key):
        return [r for r in rows if r["truth"] != r[pred_key]
                and RANK[r[pred_key]] < RANK[r["truth"]]]
    bd, vd = down("baton"), down("verb_pred")
    print("\nERRORES HACIA ABAJO (frontier→barato, el inaceptable del §2)")
    print(f"  Baton: {len(bd)}   |   Verb: {len(vd)}")
    for r in bd:
        fixed = "✓ corregido" if r["truth"] == r["verb_pred"] else "✗ persiste"
        print(f"    {r['id'].split('#')[-1] if '#' in r['id'] else r['id'][:34]:34s} "
              f"Baton={r['baton']}<-{r['truth']}  [verb→{r['verb_pred']}] {fixed}")

    # --- Casos donde provenance fue NECESARIA (verb solo no bastaba) ---
    print("\nCASOS QUE EXIGIERON design_provenance (verb-solo habría fallado)")
    any_prov = False
    for r in rows:
        if r["prov"] == "upstream":
            naive = VERB_TIER[r["verb"]]  # verb sin la regla de provenance
            print(f"    {r['id'].split('#')[-1] if '#' in r['id'] else r['id'][:30]:30s} "
                  f"verb={r['verb']}→{naive}, pero prov=upstream→{r['verb_pred']} "
                  f"(gt {r['truth']}) {'✓' if r['verb_pred']==r['truth'] else '✗'}")
            any_prov = True
    if not any_prov:
        print("    (ninguno)")

    # --- residual del verb (honestidad) ---
    vmiss = [r for r in rows if r["truth"] != r["verb_pred"]]
    print(f"\nResidual del verb-classifier: {len(vmiss)} error(es)")
    for r in vmiss:
        print(f"    {r['id']}: verb→{r['verb_pred']} vs gt {r['truth']}")


if __name__ == "__main__":
    main()
