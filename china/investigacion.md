# Normativas y Estándares Chinos Relevantes para StrayMark

> Investigación realizada el 2026-04-04.
> Contexto: StrayMark ya soporta EU AI Act, ISO 42001, NIST AI RMF/600-1, y GDPR. Esta investigación evalúa el ecosistema regulatorio chino de IA para determinar qué normativas son relevantes para integrar en el framework.

---

## Panorama General

China tiene un ecosistema regulatorio de IA muy activo y en rápida evolución. A diferencia de la UE (un solo reglamento horizontal) o NIST (frameworks voluntarios), China usa un enfoque **vertical y fragmentado**: múltiples regulaciones específicas por tecnología, estándares nacionales obligatorios (GB) y recomendados (GB/T), y un sistema de registro de algoritmos.

---

## 1. Marco de Gobernanza de Seguridad de IA (TC260)

**Equivalente más cercano a**: NIST AI RMF + EU AI Act (conceptualmente)

El [AI Safety Governance Framework v2.0](https://www.geopolitechs.org/p/china-releases-upgraded-ai-safety) fue publicado en septiembre 2025 por el TC260 (Comité Técnico Nacional de Estandarización de Seguridad de Redes). Se organiza en **4 pilares**:

| Pilar | Contenido |
|-------|-----------|
| **Principios de gobernanza** | Centrado en las personas, IA para el bien, seguro y controlable |
| **Taxonomía de riesgos** | 3 categorías: riesgos endógenos técnicos, riesgos de aplicación técnica, riesgos derivados de aplicación |
| **Contramedidas técnicas** | Medidas específicas por tipo de riesgo |
| **Medidas de gobernanza** | Implementación organizacional |

### Clasificación de riesgos

**5 niveles** basados en tres criterios: escenario de aplicación, nivel de inteligencia, escala de aplicación — desde "bajo" hasta "extremadamente grave".

### Evolución

- **v1.0** (septiembre 2024): Declaración de principios — por qué y en qué dirección gobernar la IA.
- **v2.0** (septiembre 2025): Manual de implementación — cómo gobernar, en qué etapas, cómo responder ante problemas. Introduce cláusulas sobre gobernanza de IA de código abierto y fortalece la atención a riesgos de pérdida de control y riesgos catastróficos.

### Relevancia para StrayMark

Comparable al risk classification del EU AI Act y las risk categories de NIST AI 600-1. Podría mapearse en los templates ETH y AILOG.

---

## 2. Estándares Nacionales de IA Generativa (GB/T y GB)

En abril 2025 se publicaron 3 estándares nacionales (efectivos noviembre 2025):

| Estándar | Propósito | Tipo | Equivalente |
|----------|-----------|------|-------------|
| **GB/T 45652-2025** | Seguridad de datos de pre-entrenamiento y fine-tuning para IA generativa | Recomendado | Parcialmente NIST AI 600-1 (data privacy) |
| **GB 45438-2025** | Métodos de etiquetado de contenido generado por IA | **Obligatorio** | EU AI Act Art. 50 (transparencia) |
| Estándar de anotación de datos | Seguridad en anotación de datos para IA generativa | — | Sin equivalente directo |

### GB 45438-2025: Etiquetado de Contenido IA

Cualquier servicio de IA generativa en China **debe** etiquetar contenido generado (texto, audio, imagen, video, escenas virtuales). Los productos deben indicar el nombre del modelo y número de registro en un lugar visible.

Servicios cubiertos:
- Generación/edición de chat y texto
- Síntesis/clonación de voz
- Generación/intercambio/edición de imágenes faciales y video
- Manipulación de poses
- Escenas inmersivas/hiperrealistas
- Texto a imagen, música/audio, texto a video/imagen a video

### Relevancia para StrayMark

GB 45438 es obligatorio — afecta directamente los templates MCARD (documentación de modelos que generan contenido) y AILOG (acciones de IA generativa).

---

## 3. PIPL — Ley de Protección de Información Personal

**Equivalente a**: GDPR

La [PIPL](https://securiti.ai/personal-information-protection-impact-assessment-pipia-under-china-pipl/) (efectiva desde noviembre 2021) requiere un **PIPIA** (Personal Information Protection Impact Assessment) — análogo al DPIA del GDPR:

### Escenarios que disparan PIPIA

- Procesamiento de datos personales sensibles
- Uso de datos personales para toma de decisiones automatizada
- Encargo de procesamiento a terceros
- Provisión de datos a otros manejadores de información personal
- Divulgación de información personal
- Transferencias transfronterizas de datos

### Requisitos clave

- Los reportes PIPIA deben conservarse **mínimo 3 años**
- Requiere evaluación de riesgos algorítmicos para sistemas públicos de IA
- Transferencias transfronterizas requieren: PIPIA + revisión de seguridad con autoridades + certificación de protección de información personal

### Relevancia para StrayMark

El template DPIA podría extenderse o referenciarse con PIPIA. La estructura es muy similar al DPIA del GDPR.

---

## 4. Registro de Algoritmos (CAC)

El [sistema de registro de algoritmos](https://www.lexology.com/library/detail.aspx?g=3c7273cf-8f85-4702-af70-6edf394ff1c3) administrado por la Cyberspace Administration of China (CAC) requiere:

- **Filing obligatorio** para servicios de IA con "atributos de opinión pública o capacidad de movilización social"
- +5,000 algoritmos registrados (noviembre 2025)
- 748 servicios de IA generativa con filing completado (diciembre 2025)
- Crecimiento de 250-300 entradas mensuales
- Los productos deben mostrar el nombre del modelo y número de registro

### Relevancia para StrayMark

Podría reflejarse en MCARD (modelo registrado) y SBOM (componentes con registro CAC).

---

## 5. Enmiendas a la Ley de Ciberseguridad (CSL) — enero 2026

Las [enmiendas de octubre 2025](https://www.globalpolicywatch.com/2025/10/china-amends-cybersecurity-law-and-incident-reporting-regime-to-address-ai-and-infrastructure-risks/) traen IA a la ley nacional por primera vez:

- Soporte a I+D de algoritmos
- Construcción de infraestructura de datos de entrenamiento y computación
- Normativa acelerada para ética de IA
- Evaluación de riesgos y gobernanza de seguridad de IA

---

## 6. ISO 42001 en China

China **no tiene un equivalente nacional GB/T de ISO 42001** todavía, pero [organismos de certificación operan en China](https://www.sgsgroup.com.cn/en-cn/services/iso-iec-42001-certification-artificial-intelligence-ai-management-system) ofreciendo certificación ISO/IEC 42001 directamente. Es probable que el TC260 publique una adopción nacional en el futuro (como hace con otros estándares ISO).

---

## 7. Medidas Provisionales para la Administración de Servicios de IA Generativa

Publicadas en julio 2023 por la CAC, establecen:

- Requisitos de legalidad de datos de entrenamiento
- Obligación de etiquetado de contenido generado
- Filing ante la CAC para servicios con atributos de opinión pública
- Prohibición de generar contenido que subvierta el poder del estado, incite al separatismo, socave la unidad nacional, promueva terrorismo o extremismo

---

## Comparativa con Estándares que StrayMark ya Soporta

| Aspecto | EU AI Act | NIST AI RMF | China |
|---------|-----------|-------------|-------|
| **Enfoque** | Horizontal, basado en riesgo | Voluntario, basado en frameworks | Vertical, por tecnología |
| **Clasificación de riesgo** | 4 niveles (inaceptable → mínimo) | Sin clasificación formal | 5 niveles (TC260 v2.0) |
| **Registro obligatorio** | Base de datos EU para high-risk | No | Sí (CAC: algoritmos + GenAI) |
| **Etiquetado de contenido IA** | Art. 50 (transparencia) | Voluntario | GB 45438 (obligatorio) |
| **Protección de datos** | GDPR (DPIA) | Sin equivalente federal | PIPL (PIPIA) |
| **Reporte de incidentes** | Art. 73 (10-15 días) | Voluntario | CSL enmiendas 2026 |
| **IA generativa específica** | Parcial (GPAI chapter) | NIST AI 600-1 | Medidas Provisionales GenAI + GB/T 45652 |
| **Gestión de IA (sistema)** | Referencia ISO 42001 | Framework voluntario | Sin equivalente nacional aún (usa ISO 42001 directamente) |

---

## Oportunidades de Integración en StrayMark

| Prioridad | Normativa | Template afectado | Cambio propuesto |
|-----------|-----------|-------------------|-----------------|
| **Alta** | TC260 Risk Classification (5 niveles) | ETH, AILOG | Nuevo campo `tc260_risk_level` o sección de evaluación TC260 (como ya hacemos con EU AI Act risk y NIST GenAI risks) |
| **Alta** | PIPL / PIPIA | DPIA | Referencia cruzada desde DPIA — la estructura PIPIA es muy similar al DPIA del GDPR |
| **Media** | GB 45438 Content Marking | MCARD | Nueva sección para servicios que generan contenido — requisitos de etiquetado |
| **Media** | CAC Algorithm Filing | MCARD, SBOM | Nuevo campo para número de registro CAC (`cac_filing_number`) |
| **Baja** | GB/T 45652 Data Security | AILOG, MCARD | Referencia en sección de datos de entrenamiento |
| **Baja** | CSL Amendments 2026 | INC | Considerar requisitos de reporte de incidentes bajo CSL |

---

## Fuentes

- [China Releases Upgraded AI Safety Governance Framework (TC260 v2.0)](https://www.geopolitechs.org/p/china-releases-upgraded-ai-safety)
- [China AI Governance Framework: What Global Businesses Need to Know in 2026](https://gaicc.org/blog/china-ai-governance-framework/)
- [China's TC260 Releases AI Safety Governance Framework | OneTrust](https://www.onetrust.com/blog/chinas-tc260-releases-ai-safety-governance-framework/)
- [China's Key Developments in AI Governance in 2025 | ICLG](https://iclg.com/practice-areas/telecoms-media-and-internet-laws-and-regulations/03-china-s-key-developments-in-artificial-intelligence-governance-in-2025)
- [AI Laws and Regulations in China | CMS](https://cms.law/en/int/expert-guides/ai-regulation-scanner/china)
- [Key Differences Between EU, Chinese AI Regulations | IAPP](https://iapp.org/news/a/preparing-for-compliance-key-differences-between-eu-chinese-ai-regulations)
- [China's PIPIA Under PIPL | Securiti](https://securiti.ai/personal-information-protection-impact-assessment-pipia-under-china-pipl/)
- [China's Algorithm Filing Regime | Lexology](https://www.lexology.com/library/detail.aspx?g=3c7273cf-8f85-4702-af70-6edf394ff1c3)
- [GB 45438-2025 Content Labeling Standard](https://www.codeofchina.com/standard/GB45438-2025.html)
- [China Amends Cybersecurity Law for AI | Global Policy Watch](https://www.globalpolicywatch.com/2025/10/china-amends-cybersecurity-law-and-incident-reporting-regime-to-address-ai-and-infrastructure-risks/)
- [ISO/IEC 42001 Certification in China | SGS](https://www.sgsgroup.com.cn/en-cn/services/iso-iec-42001-certification-artificial-intelligence-ai-management-system)
- [Notes from Asia-Pacific: Strong Start to 2026 for China's AI Governance | IAPP](https://iapp.org/news/a/notes-from-the-asia-pacific-region-strong-start-to-2026-for-china-s-data-ai-governance-landscape)
- [An Agile Approach: Understanding China's AI Governance Framework | Lexology](https://www.lexology.com/library/detail.aspx?g=ef8efea8-05b1-4bf2-8fd5-08ac531bfd7d)
- [Global AI Governance Law and Policy: China | IAPP](https://iapp.org/resources/article/global-ai-governance-china)
