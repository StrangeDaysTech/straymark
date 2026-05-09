# Contribuir a StrayMark

¡Gracias por tu interés en contribuir a StrayMark! Este documento proporciona guías e información para contribuidores.

**Idiomas**: [English](../../../CONTRIBUTING.md) | Español | [简体中文](../zh-CN/CONTRIBUTING.md)

## Tabla de Contenidos

- [Código de Conducta](#código-de-conducta)
- [Acuerdo de Licencia de Contribuidor (CLA)](#acuerdo-de-licencia-de-contribuidor-cla)
- [¿Cómo Puedo Contribuir?](#cómo-puedo-contribuir)
- [Configuración de Desarrollo](#configuración-de-desarrollo)
- [Proceso de Pull Request](#proceso-de-pull-request)
- [Guías de Estilo](#guías-de-estilo)
- [Estándares de Documentación](#estándares-de-documentación)

---

## Código de Conducta

Este proyecto se rige por nuestro [Código de Conducta del Pacto del Contribuidor](../../../CODE_OF_CONDUCT.md) ([Español](CODE_OF_CONDUCT.md)). Al participar, se espera que cumplas con este código.

En resumen: sé respetuoso, inclusivo y constructivo en todas las interacciones. El acoso, la discriminación y el trolling no son tolerados. Por favor lee el [Código de Conducta completo](CODE_OF_CONDUCT.md) antes de contribuir.

---

## Acuerdo de Licencia de Contribuidor (CLA)

Este proyecto requiere que todos los contribuidores firmen un **Acuerdo de Licencia de Contribuidor (CLA)** antes de que sus pull requests puedan ser fusionados. Usamos [CLA Assistant](https://cla-assistant.io/) para gestionar este proceso.

### Cómo funciona

1. Cuando abras tu primer pull request, CLA Assistant publicará automáticamente un comentario pidiéndote que firmes el CLA.
2. Haz clic en el enlace del comentario para revisar y firmar el acuerdo.
3. El CLA solo necesita firmarse una vez — cubre todas las contribuciones futuras a este proyecto.
4. Una vez firmado, CLA Assistant actualizará el estado del check del PR y tu contribución podrá proceder a revisión.

Si tienes preguntas sobre el CLA, por favor abre una [Discusión](https://github.com/StrangeDaysTech/straymark/discussions).

---

## ¿Cómo Puedo Contribuir?

### Reportar Bugs

Antes de crear un reporte de bug, por favor revisa los issues existentes para evitar duplicados.

**Al reportar un bug, incluye:**

- Un título claro y descriptivo
- Pasos para reproducir el comportamiento
- Comportamiento esperado
- Comportamiento real
- Capturas de pantalla (si aplica)
- Detalles del entorno (SO, plataforma de IA, etc.)

### Sugerir Características

¡Las sugerencias de características son bienvenidas! Por favor incluye:

- Una descripción clara de la característica
- El problema que resuelve
- Posibles enfoques de implementación
- Alternativas que hayas considerado

### Mejorar Documentación

Las mejoras de documentación son muy valoradas:

- Corregir erratas o redacción poco clara
- Agregar ejemplos
- Mejorar explicaciones
- Traducir a otros idiomas

### Enviar Código

Las contribuciones de código deben:

- Corregir un bug o implementar una característica
- Incluir pruebas apropiadas (si aplica)
- Seguir las guías de estilo del proyecto
- Actualizar documentación según sea necesario

---

## Configuración de Desarrollo

### Prerrequisitos

- **Git**
- **Un editor de texto** (VS Code recomendado)
- **StrayMark CLI** (para validación de documentos — multiplataforma)
- **Rust toolchain** (para desarrollo del CLI — instalar vía [rustup.rs](https://rustup.rs/))
- **Node.js 20+** (opcional, para markdownlint)

### Pasos de Configuración

1. **Fork del repositorio**

   Haz clic en "Fork" en la [página del repositorio de GitHub](https://github.com/StrangeDaysTech/straymark).

2. **Clonar tu fork**
   ```bash
   git clone https://github.com/tu-usuario/straymark.git
   cd straymark
   ```

3. **Instalar el hook de pre-commit**
   ```bash
   echo '#!/bin/sh
   straymark validate --staged' > .git/hooks/pre-commit
   chmod +x .git/hooks/pre-commit
   ```

4. **Instalar herramientas de desarrollo (opcional)**
   ```bash
   # Linting de Markdown
   npm install -g markdownlint-cli
   ```

5. **Crear una rama**
   ```bash
   git checkout -b feature/nombre-de-tu-caracteristica
   # o
   git checkout -b fix/tu-correccion-de-bug
   ```

6. **Hacer tus cambios y validar**
   ```bash
   straymark validate
   ```

---

## Proceso de Pull Request

### Antes de Enviar

- [ ] Ejecutar `straymark validate` exitosamente
- [ ] Actualizar documentación si es necesario
- [ ] Agregarte a CONTRIBUTORS.md (si aplica)
- [ ] Escribir una descripción clara del PR

### Formato de Título del PR

Usa formato de commit convencional:

```
tipo(alcance): descripción

Ejemplos:
feat(templates): agregar plantilla para revisiones de seguridad
fix(validation): corregir regex para nomenclatura de archivos
docs(readme): clarificar pasos de instalación
chore(ci): actualizar flujo de trabajo de GitHub Actions
```

**Tipos:**
- `feat` - Nueva característica
- `fix` - Corrección de bug
- `docs` - Cambios de documentación
- `chore` - Tareas de mantenimiento
- `refactor` - Refactorización de código
- `test` - Adiciones o correcciones de pruebas

### Plantilla de Descripción del PR

```markdown
## Resumen
Breve descripción de los cambios

## Motivación
¿Por qué se necesita este cambio?

## Cambios
- Cambio 1
- Cambio 2

## Pruebas
¿Cómo se probaron estos cambios?

## Lista de Verificación
- [ ] `straymark validate` pasa sin errores
- [ ] Documentación actualizada
- [ ] Sin información sensible incluida
```

### Proceso de Revisión

1. Un mantenedor revisará tu PR
2. Atiende cualquier cambio solicitado
3. Una vez aprobado, un mantenedor hará merge

---

## Guías de Estilo

### Markdown

- Usa encabezados estilo ATX (`#`, `##`, etc.)
- Usa bloques de código cercados con identificadores de lenguaje
- Usa tablas para datos estructurados
- Mantén líneas bajo 120 caracteres cuando sea práctico
- Usa líneas en blanco para separar secciones

### Front-matter YAML

```yaml
---
id: TYPE-YYYY-MM-DD-NNN
title: Título claro y descriptivo
status: draft | accepted | deprecated
created: YYYY-MM-DD
# Campos adicionales según sea necesario
---
```

### Nomenclatura de Archivos

Documentos StrayMark:
```
[TIPO]-[YYYY-MM-DD]-[NNN]-[descripcion].md
```

- Usa minúsculas para la descripción
- Usa guiones para separar palabras
- Mantén las descripciones concisas pero claras

### Código en Scripts

- Usa nombres de variables claros
- Agrega comentarios para lógica compleja
- Sigue las mejores prácticas de shell/PowerShell

---

## Estándares de Documentación

### Agregar un Nuevo Tipo de Documento

Si estás proponiendo un nuevo tipo de documento:

1. **Crear la plantilla**
   - Agregar `TEMPLATE-NUEVOTIPO.md` a `dist/.straymark/templates/`
   - Seguir patrones de plantillas existentes

2. **Actualizar docs de gobernanza**
   - `dist/.straymark/00-governance/DOCUMENTATION-POLICY.md`
   - `dist/.straymark/00-governance/AGENT-RULES.md`
   - `dist/.straymark/QUICK-REFERENCE.md`

3. **Actualizar configs de agente**
   - `dist/dist-templates/directives/` (plantillas de distribución)

4. **Actualizar validación**
   - Reglas de validación en el CLI (`cli/src/commands/validate.rs`)
   - `dist/.github/workflows/docs-validation.yml`

5. **Documentar el cambio**
   - Crear un ADR explicando el nuevo tipo
   - Actualizar el README si es necesario

### Escribir Plantillas

Las plantillas deben incluir:

- Front-matter YAML completo con todos los campos
- Encabezados de sección claros
- Texto placeholder explicando qué va en cada sección
- Ejemplos donde sea útil

### Escribir Documentos de Gobernanza

- Sé claro y sin ambigüedades
- Usa tablas para información de referencia
- Incluye ejemplos
- Mantén las reglas accionables

---

## Desarrollo del CLI

El CLI de StrayMark está escrito en Rust y se encuentra en el directorio `cli/`.

### Compilar

```bash
cd cli
cargo build
```

### Ejecutar Tests

```bash
cd cli
cargo test
```

### Build de Release

```bash
cd cli
cargo build --release
```

El binario de release está optimizado con LTO y stripped para tamaño mínimo.

### Arquitectura

```
cli/src/
├── main.rs              # Punto de entrada + definición CLI con clap
├── commands/
│   ├── mod.rs           # Enrutamiento de subcomandos
│   ├── init.rs          # straymark init [path]
│   ├── update.rs        # straymark update (combinado)
│   ├── update_framework.rs # straymark update-framework
│   ├── update_cli.rs    # straymark update-cli
│   ├── remove.rs        # straymark remove [--full]
│   ├── status.rs        # straymark status [path]
│   └── about.rs         # straymark about
├── config.rs            # Gestión de configuración y checksums
├── download.rs          # API de GitHub Releases (filtrado por prefijo)
├── inject.rs            # Inyección de archivos de directiva (markers)
├── manifest.rs          # Parsing de dist-manifest.yml
├── platform.rs          # Detección de SO/arquitectura para descarga de binarios
├── self_update.rs       # Lógica de auto-actualización del binario CLI
└── utils.rs             # Helpers (hashing, colores, paths)
```

> **Nota**: Framework y CLI usan versionado independiente (tags `fw-*` y `cli-*`). Ver [Referencia CLI](docs/adopters/CLI-REFERENCE.md#versioning) para detalles.

---

## ¿Preguntas?

Si tienes preguntas sobre contribuir:

1. Revisa [Issues](https://github.com/StrangeDaysTech/straymark/issues) existentes
2. Revisa [Discussions](https://github.com/StrangeDaysTech/straymark/discussions)
3. Abre una nueva Discussion para preguntas generales
4. Abre un Issue para bugs o características específicas

---

## Reconocimiento

Los contribuidores son reconocidos en:

- Gráfico de contribuidores de GitHub
- Notas de release para contribuciones significativas
- CONTRIBUTORS.md (para contribuidores recurrentes)

¡Gracias por ayudar a mejorar StrayMark!

---

*StrayMark — Porque cada cambio cuenta una historia.*

[Strange Days Tech](https://strangedays.tech)
