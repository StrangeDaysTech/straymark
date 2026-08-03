# StrayMark — Baton: enrutamiento de trabajo declarado (Experimental)

**Dile a StrayMark qué *tipo* de trabajo es un cambio — y recibe consejo de enrutamiento honesto y consciente del costo.**

> ⚠️ **EXPERIMENTAL — Baton 0.x (Track C: validación prospectiva con adoptantes).** Baton es un experimento opt-in e inestable. Su superficie de CLI, formato de reportes, y su existencia misma pueden cambiar o desaparecer sin ciclo de deprecación hasta que gradúe. **No** es parte del contrato soportado de Framework ni CLI — no construyas automatización contra él todavía. Es **read-only**: Baton nunca escribe en tu proyecto.

---

## Qué es Baton

Baton es el **Coherence Bridge** experimental de StrayMark: reconcilia la intención (charters, specs) contra la gobernanza y el código, y clasifica unidades de trabajo en tiers de enrutamiento conscientes del costo:

- **Frontier** — diseño genuino / territorio desconocido: enruta al modelo más fuerte.
- **Economic** — trabajo conocido con contratos claros: enruta a un modelo medio.
- **Local/operator** — trabajo mecánico o definido upstream: bastan modelos baratos o locales.

Desde la decisión **#332**, la clasificación es **por declaración**: el autor declara la clase de trabajo en frontmatter, y Baton nunca adivina desde títulos o texto. Una unidad no declarada es un *estado honesto* — Baton la enruta al tier frontier y te sugiere declarar, en vez de fabricar una clasificación.

## Qué te pide el Track C

La validación prospectiva (gate #3 para graduar Baton) necesita tráfico real de gobernanza. Tu parte es pequeña:

1. **Declara la clase de trabajo** en unidades nuevas, exactamente como documenta la [Referencia de CLI](./CLI-REFERENCE.md#straymark-validate):
   - Frontmatter del Charter: `work_verb: design | implement | audit | operate` y, solo cuando importa para `implement`, `design_provenance: new | upstream`.
   - Entradas del backlog de follow-ups: los mismos campos opcionales.
2. **Trabaja como siempre.** Nada más de tu cadencia cambia; los campos son advisory y la ausencia es silenciosa.
3. **Tras 2–4 semanas**, corre el protocolo de calibración simplificado del [kit del adoptante](https://github.com/StrangeDaysTech/straymark/blob/main/experiment-baton/07-track-c-adopter-kit.md) y reporta (ver [Adopter Feedback](./ADOPTER-FEEDBACK.md)).

El kit completo — vocabulario, reglas de determinación, protocolo de calibración, preguntas de fricción — vive en [`experiment-baton/07-track-c-adopter-kit.md`](https://github.com/StrangeDaysTech/straymark/blob/main/experiment-baton/07-track-c-adopter-kit.md).

## Cómo obtener el binario

Baton se distribuye **solo como assets de GitHub Releases** (sin crates.io), igual que Loom:

1. Abre el [release `baton-*`](https://github.com/StrangeDaysTech/straymark/releases) — solo se conserva el más reciente.
2. Descarga el asset para tu plataforma:

   | Plataforma | Asset |
   |---|---|
   | Linux x86_64 | `straymark-baton-v{version}-x86_64-unknown-linux-gnu.tar.gz` |
   | macOS Intel | `straymark-baton-v{version}-x86_64-apple-darwin.tar.gz` |
   | macOS Apple Silicon | `straymark-baton-v{version}-aarch64-apple-darwin.tar.gz` |
   | Windows x86_64 | `straymark-baton-v{version}-x86_64-pc-windows-msvc.zip` |

3. Extráelo y pon `straymark-baton` en tu `PATH`.

Alternativa: compílalo desde el repositorio — `cargo build --release --manifest-path experiment-baton/Cargo.toml`.

Comprobación rápida (read-only, no muta nada):

```bash
straymark-baton --version
straymark-baton classify .          # clases declaradas de tus unidades de trabajo registradas
straymark-baton route . --dry-run   # consejo de enrutamiento por tier; nunca ejecuta nada
```

## Garantías read-only

- `classify` y `route` solo **leen** tu árbol `.straymark/`; `route` exige `--dry-run` y no existe ruta de ejecución.
- Sin llamadas de red a proveedores de modelos — Baton clasifica desde declaraciones, no corre modelos.
- Nada en tus documentos de gobernanza ni en tu código es modificado. El CLI (`validate`, `status`) sigue siendo el gate.

## Limitaciones honestas

- **Baton es N=1+ (dogfood de Sentinel).** Espera asperezas; repórtalas por el canal de Adopter Feedback.
- El consejo de enrutamiento es **consejo**: nunca bloquea, muta, ni decide por ti.
- Durante el Track C, los releases se reemplazan entre sí (solo sobrevive el `baton-*` más reciente) — re-descarga cuando aparezca uno nuevo.

---

## Ver también

- [Kit del adoptante — Track C](https://github.com/StrangeDaysTech/straymark/blob/main/experiment-baton/07-track-c-adopter-kit.md) — colocación de la declaración, reglas de determinación, calibración E1 simplificada.
- [Adopter Feedback](./ADOPTER-FEEDBACK.md) — cómo fluyen los hallazgos y la telemetría upstream.
- [Referencia de CLI](./CLI-REFERENCE.md) — los checks advisory de vocabulario `work_verb` en `straymark validate`.

---

*StrayMark — Because every change tells a story.*

[Strange Days Tech](https://strangedays.tech)
