#!/usr/bin/env python3
"""Baton E1 scoring — une la hoja CIEGA rellenada contra la clave oculta.

Read-only. No muta el repo. Computa lo que pide el §2 del plan de adoptante:
precisión por clase, precisión del subconjunto high+medium (la cifra decisiva),
matriz de confusión y, sobre todo, la DIRECCIÓN del error (hacia abajo = peligroso).

Uso:
    python3 .straymark/baton-calibration/score.py
"""
import json
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
LABELS = os.path.join(HERE, "labels-blind.yml")
KEY = os.path.join(HERE, ".predictions-key.json")

CLASSES = ["planner", "implementer", "auditor", "operator"]
# Rango de "route-up" (idéntico a classify.rs::rank): mayor = tier más capaz.
RANK = {"planner": 4, "implementer": 3, "auditor": 2, "operator": 1}


def parse_labels(path):
    """Mini-parser del subconjunto YAML que produce la hoja (sin dependencias)."""
    entries = []
    cur = None
    with open(path, encoding="utf-8") as fh:
        for raw in fh:
            line = raw.rstrip("\n")
            if line.lstrip().startswith("#") or not line.strip():
                continue
            if line.startswith("- id:"):
                if cur:
                    entries.append(cur)
                cur = {"id": line.split("id:", 1)[1].strip()}
            elif cur is not None and ":" in line:
                k, _, v = line.strip().partition(":")
                k = k.strip()
                # quita comentario en línea solo en true_class (el resto puede traer ':')
                if k == "true_class":
                    v = v.split("#", 1)[0]
                cur[k] = v.strip().strip('"')
        if cur:
            entries.append(cur)
    return entries


def main():
    if not os.path.exists(KEY):
        sys.exit("falta .predictions-key.json — regenera la hoja primero")
    key = json.load(open(KEY, encoding="utf-8"))
    labels = parse_labels(LABELS)

    rows, blank, bad = [], [], []
    for e in labels:
        tc = (e.get("true_class") or "").strip().lower()
        if not tc:
            blank.append(e["id"])
            continue
        if tc not in CLASSES:
            bad.append((e["id"], tc))
            continue
        k = key.get(e["id"])
        if not k:
            bad.append((e["id"], "id no está en la clave"))
            continue
        rows.append({"id": e["id"], "true": tc, "pred": k["predicted_class"],
                     "conf": k["confidence"]})

    total = len(labels)
    print(f"Muestra: {total} unidades | etiquetadas: {len(rows)} | "
          f"en blanco: {len(blank)} | inválidas: {len(bad)}")
    if bad:
        for i, why in bad:
            print(f"  ! {i}: {why}")
    if not rows:
        sys.exit("\nNada etiquetado todavía. Llena `true_class` en labels-blind.yml y re-corre.")
    if blank:
        print(f"  (sin etiquetar aún: {', '.join(blank)})")

    correct = sum(r["true"] == r["pred"] for r in rows)
    print(f"\nExactitud global: {correct}/{len(rows)} = {correct/len(rows):.2f}")

    # --- precisión por clase predicha ---
    print("\nPrecisión por clase PREDICHA (aciertos / veces que Baton predijo esa clase):")
    for c in CLASSES:
        sub = [r for r in rows if r["pred"] == c]
        if sub:
            ok = sum(r["true"] == c for r in sub)
            print(f"  {c:12s} {ok}/{len(sub)} = {ok/len(sub):.2f}")
        else:
            print(f"  {c:12s} (no predicha en la muestra)")

    # --- LA CIFRA DECISIVA: precisión del subconjunto high+medium ---
    hm = [r for r in rows if r["conf"] in ("high", "medium")]
    if hm:
        ok = sum(r["true"] == r["pred"] for r in hm)
        print(f"\n>> Precisión high+medium (DECISIVA): {ok}/{len(hm)} = {ok/len(hm):.2f}"
              f"  (objetivo ≥ 0.80)")
    only_high = [r for r in rows if r["conf"] == "high"]
    if only_high:
        ok = sum(r["true"] == r["pred"] for r in only_high)
        print(f"   solo high: {ok}/{len(only_high)} = {ok/len(only_high):.2f}")
    low = [r for r in rows if r["conf"] == "low"]
    if low:
        ok = sum(r["true"] == r["pred"] for r in low)
        print(f"   solo low : {ok}/{len(low)} = {ok/len(low):.2f}")

    # --- matriz de confusión (filas=verdadero, columnas=predicho) ---
    print("\nMatriz de confusión (fila = verdadero, col = predicho):")
    print("           " + "".join(f"{c[:4]:>6s}" for c in CLASSES))
    for t in CLASSES:
        cells = [sum(1 for r in rows if r["true"] == t and r["pred"] == p) for p in CLASSES]
        print(f"  {t:9s}" + "".join(f"{n:6d}" for n in cells))

    # --- dirección del error: la señal de alarma ---
    down = [r for r in rows if r["true"] != r["pred"] and RANK[r["pred"]] < RANK[r["true"]]]
    up = [r for r in rows if r["true"] != r["pred"] and RANK[r["pred"]] > RANK[r["true"]]]
    print(f"\nDirección del error (de {len(rows)-correct} errores):")
    print(f"  HACIA ABAJO (peligroso — trabajo real ruteado a tier barato): {len(down)}")
    for r in down:
        print(f"    ! {r['id']}: verdadero={r['true']} -> predicho={r['pred']} [{r['conf']}]")
    print(f"  hacia arriba (conservador, seguro): {len(up)}")
    print("\nLectura: el sesgo de Baton es route-up. Si los errores se concentran")
    print("HACIA ABAJO (sobre todo en high/medium), esa es la señal de alarma del §2.")


if __name__ == "__main__":
    main()
