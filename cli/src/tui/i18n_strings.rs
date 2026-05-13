//! Translation table for static UI strings rendered by the TUI shell:
//! navigation tree group labels, status-bar key hints, help-popup sections
//! and action descriptions.
//!
//! The single entry point is [`t`]: callers pass the canonical English
//! literal, and the function returns either a translation for `lang` or
//! the English original (silent fallback) when no entry matches. This
//! mirrors the file-level fallback behavior in `utils::resolve_localized_path`.
//!
//! Translated document content (governance docs, templates, adopter
//! markdown) is *not* handled here — those go through the file-resolution
//! path in `utils.rs` and `tui::index`.

/// Translate a canonical English UI string. Returns the original `en`
/// when `lang == "en"`, when the language is unknown, or when the
/// specific string has not been translated. Callers MUST pass the exact
/// English literal that appears as a key in the match arms below.
pub fn t<'a>(en: &'a str, lang: &str) -> &'a str {
    if lang == "en" {
        return en;
    }
    match (en, lang) {
        // ── Navigation tree group labels ──────────────────────────────
        ("Governance", "es") => "Gobernanza",
        ("Governance", "zh-CN") => "治理",
        ("Requirements", "es") => "Requisitos",
        ("Requirements", "zh-CN") => "需求",
        ("Design", "es") => "Diseño",
        ("Design", "zh-CN") => "设计",
        ("Implementation", "es") => "Implementación",
        ("Implementation", "zh-CN") => "实施",
        ("Testing", "es") => "Pruebas",
        ("Testing", "zh-CN") => "测试",
        ("Operations", "es") => "Operaciones",
        ("Operations", "zh-CN") => "运营",
        ("Evolution", "es") => "Evolución",
        ("Evolution", "zh-CN") => "演进",
        ("AI Audit", "es") => "Auditoría IA",
        ("AI Audit", "zh-CN") => "AI 审计",
        ("Security", "es") => "Seguridad",
        ("Security", "zh-CN") => "安全",
        ("AI Models", "es") => "Modelos IA",
        ("AI Models", "zh-CN") => "AI 模型",
        // Charter-related labels: "Charters" stays as a loanword in ES to match
        // the rest of the bilingual technical vocabulary (Plan→Charter rename).
        ("Charters", "es") => "Charters",
        ("Charters", "zh-CN") => "章程",

        // ── Subgroup labels ───────────────────────────────────────────
        ("Exceptions", "es") => "Excepciones",
        ("Exceptions", "zh-CN") => "例外",
        ("Decisions", "es") => "Decisiones",
        ("Decisions", "zh-CN") => "决策",
        ("Incidents", "es") => "Incidentes",
        ("Incidents", "zh-CN") => "事件",
        ("Runbooks", "es") => "Runbooks",
        ("Runbooks", "zh-CN") => "操作手册",
        ("Technical debt", "es") => "Deuda técnica",
        ("Technical debt", "zh-CN") => "技术债务",
        ("Agent logs", "es") => "Bitácoras de agentes",
        ("Agent logs", "zh-CN") => "代理日志",
        ("Ethical reviews", "es") => "Revisiones éticas",
        ("Ethical reviews", "zh-CN") => "伦理审查",

        // ── Nav tree title and sort hints ─────────────────────────────
        ("Navigation", "es") => "Navegación",
        ("Navigation", "zh-CN") => "导航",
        ("[s:sort ↓name]", "es") => "[s:orden ↓nombre]",
        ("[s:sort ↓name]", "zh-CN") => "[s:排序 ↓名称]",
        ("[s:sort ↓date]", "es") => "[s:orden ↓fecha]",
        ("[s:sort ↓date]", "zh-CN") => "[s:排序 ↓日期]",

        // ── Status bar key hints ──────────────────────────────────────
        ("quit", "es") => "salir",
        ("quit", "zh-CN") => "退出",
        ("search", "es") => "buscar",
        ("search", "zh-CN") => "搜索",
        ("panel", "es") => "panel",
        ("panel", "zh-CN") => "面板",
        ("open", "es") => "abrir",
        ("open", "zh-CN") => "打开",
        ("fullscreen", "es") => "pantalla completa",
        ("fullscreen", "zh-CN") => "全屏",
        ("back", "es") => "volver",
        ("back", "zh-CN") => "返回",
        ("help", "es") => "ayuda",
        ("help", "zh-CN") => "帮助",
        ("apply", "es") => "aplicar",
        ("apply", "zh-CN") => "应用",
        ("cancel", "es") => "cancelar",
        ("cancel", "zh-CN") => "取消",
        ("(press any key to dismiss)", "es") => "(presiona una tecla para descartar)",
        ("(press any key to dismiss)", "zh-CN") => "（按任意键关闭）",
        ("docs", "es") => "docs",
        ("docs", "zh-CN") => "文档",

        // ── Help popup: title, section headers, footer ────────────────
        ("Keyboard Shortcuts", "es") => "Atajos de teclado",
        ("Keyboard Shortcuts", "zh-CN") => "键盘快捷键",
        ("Navigation panel", "es") => "Panel de navegación",
        ("Navigation panel", "zh-CN") => "导航面板",
        ("Metadata panel", "es") => "Panel de metadatos",
        ("Metadata panel", "zh-CN") => "元数据面板",
        ("Document panel", "es") => "Panel del documento",
        ("Document panel", "zh-CN") => "文档面板",
        ("General", "es") => "General",
        ("General", "zh-CN") => "通用",
        ("Press any key to close", "es") => "Presiona cualquier tecla para cerrar",
        ("Press any key to close", "zh-CN") => "按任意键关闭",

        // ── Help popup: action descriptions ───────────────────────────
        ("Move selection down", "es") => "Mover selección hacia abajo",
        ("Move selection down", "zh-CN") => "向下移动选择",
        ("Move selection up", "es") => "Mover selección hacia arriba",
        ("Move selection up", "zh-CN") => "向上移动选择",
        ("Expand group / Open document", "es") => "Expandir grupo / Abrir documento",
        ("Expand group / Open document", "zh-CN") => "展开分组 / 打开文档",
        ("Collapse group / Clear search", "es") => "Colapsar grupo / Limpiar búsqueda",
        ("Collapse group / Clear search", "zh-CN") => "折叠分组 / 清除搜索",
        ("Jump to group by number", "es") => "Saltar al grupo por número",
        ("Jump to group by number", "zh-CN") => "按编号跳到分组",
        ("Move between related links", "es") => "Mover entre enlaces relacionados",
        ("Move between related links", "zh-CN") => "在关联链接间移动",
        ("Follow selected related link", "es") => "Seguir enlace relacionado seleccionado",
        ("Follow selected related link", "zh-CN") => "跳转到选中的关联链接",
        ("Back to Navigation", "es") => "Volver a navegación",
        ("Back to Navigation", "zh-CN") => "返回导航",
        ("Scroll down", "es") => "Desplazarse hacia abajo",
        ("Scroll down", "zh-CN") => "向下滚动",
        ("Scroll up", "es") => "Desplazarse hacia arriba",
        ("Scroll up", "zh-CN") => "向上滚动",
        ("Top / Bottom of document", "es") => "Inicio / Fin del documento",
        ("Top / Bottom of document", "zh-CN") => "文档开头 / 末尾",
        ("Half page down / up", "es") => "Media página abajo / arriba",
        ("Half page down / up", "zh-CN") => "向下 / 向上半页",
        ("Page down / up", "es") => "Página abajo / arriba",
        ("Page down / up", "zh-CN") => "向下 / 向上一页",
        ("Next / Previous document", "es") => "Documento siguiente / anterior",
        ("Next / Previous document", "zh-CN") => "下一个 / 上一个文档",
        ("Toggle fullscreen", "es") => "Alternar pantalla completa",
        ("Toggle fullscreen", "zh-CN") => "切换全屏",
        ("Exit fullscreen / Back to Nav", "es") => "Salir de pantalla completa / Volver",
        ("Exit fullscreen / Back to Nav", "zh-CN") => "退出全屏 / 返回导航",
        ("Next panel: Nav → Meta → Doc", "es") => "Panel siguiente: Nav → Meta → Doc",
        ("Next panel: Nav → Meta → Doc", "zh-CN") => "下一面板：导航 → 元数据 → 文档",
        ("Prev panel: Doc → Meta → Nav", "es") => "Panel anterior: Doc → Meta → Nav",
        ("Prev panel: Doc → Meta → Nav", "zh-CN") => "上一面板：文档 → 元数据 → 导航",
        ("Search by name, title, tags, date", "es") => {
            "Buscar por nombre, título, etiquetas, fecha"
        }
        ("Search by name, title, tags, date", "zh-CN") => "按名称、标题、标签、日期搜索",
        ("Cycle sort order", "es") => "Cambiar orden de clasificación",
        ("Cycle sort order", "zh-CN") => "切换排序方式",
        ("Refresh document index", "es") => "Refrescar índice de documentos",
        ("Refresh document index", "zh-CN") => "刷新文档索引",
        ("Cycle display language", "es") => "Cambiar idioma de la interfaz",
        ("Cycle display language", "zh-CN") => "切换显示语言",
        ("Language", "es") => "Idioma",
        ("Language", "zh-CN") => "语言",

        // ── Metadata panel ────────────────────────────────────────────
        ("Metadata", "es") => "Metadatos",
        ("Metadata", "zh-CN") => "元数据",
        ("No document selected", "es") => "Sin documento seleccionado",
        ("No document selected", "zh-CN") => "未选择文档",
        ("File:", "es") => "Archivo:",
        ("File:", "zh-CN") => "文件:",
        ("No frontmatter", "es") => "Sin frontmatter",
        ("No frontmatter", "zh-CN") => "无 frontmatter",
        ("Status:", "es") => "Estado:",
        ("Status:", "zh-CN") => "状态:",
        ("Created:", "es") => "Creado:",
        ("Created:", "zh-CN") => "创建:",
        ("Agent:", "es") => "Agente:",
        ("Agent:", "zh-CN") => "代理:",
        ("Confidence:", "es") => "Confianza:",
        ("Confidence:", "zh-CN") => "可信度:",
        ("Risk:", "es") => "Riesgo:",
        ("Risk:", "zh-CN") => "风险:",
        ("Review:", "es") => "Revisión:",
        ("Review:", "zh-CN") => "审查:",
        ("⚠ REQUIRED", "es") => "⚠ REQUERIDA",
        ("⚠ REQUIRED", "zh-CN") => "⚠ 需要",
        ("Tags:", "es") => "Etiquetas:",
        ("Tags:", "zh-CN") => "标签:",
        ("Related:", "es") => "Relacionados:",
        ("Related:", "zh-CN") => "关联:",
        (" (Enter: search)", "es") => " (Enter: buscar)",
        (" (Enter: search)", "zh-CN") => "（Enter: 搜索）",
        (" (Enter: follow)", "es") => " (Enter: seguir)",
        (" (Enter: follow)", "zh-CN") => "（Enter: 跳转）",
        ("Charter ID:", "es") => "Charter ID:",
        ("Charter ID:", "zh-CN") => "Charter ID:",
        ("Effort:", "es") => "Esfuerzo:",
        ("Effort:", "zh-CN") => "工作量:",
        ("Trigger:", "es") => "Disparador:",
        ("Trigger:", "zh-CN") => "触发条件:",
        ("Origin:", "es") => "Origen:",
        ("Origin:", "zh-CN") => "来源:",

        // ── Document panel ────────────────────────────────────────────
        ("Document", "es") => "Documento",
        ("Document", "zh-CN") => "文档",

        // ── Welcome screen (empty doc state) ──────────────────────────
        ("Documentation Governance for AI-Assisted Development", "es") => {
            "Gobernanza de documentación para desarrollo asistido por IA"
        }
        ("Documentation Governance for AI-Assisted Development", "zh-CN") => {
            "AI 辅助开发的文档治理"
        }
        ("Using repo root: ", "es") => "Usando raíz del repo: ",
        ("Using repo root: ", "zh-CN") => "使用仓库根目录：",
        ("Total documents: ", "es") => "Total de documentos: ",
        ("Total documents: ", "zh-CN") => "文档总数：",
        ("Quick start", "es") => "Inicio rápido",
        ("Quick start", "zh-CN") => "快速开始",
        ("Navigate groups in the left panel", "es") => {
            "Navega grupos en el panel izquierdo"
        }
        ("Navigate groups in the left panel", "zh-CN") => "在左侧面板中导航分组",
        ("Expand a group and open a document", "es") => {
            "Expande un grupo y abre un documento"
        }
        ("Expand a group and open a document", "zh-CN") => "展开分组并打开文档",
        ("Next panel / ", "es") => "Panel siguiente / ",
        ("Next panel / ", "zh-CN") => "下一面板 / ",
        ("Previous panel", "es") => "Panel anterior",
        ("Previous panel", "zh-CN") => "上一面板",
        ("Search by filename, title, tags, or date", "es") => {
            "Buscar por archivo, título, etiquetas o fecha"
        }
        ("Search by filename, title, tags, or date", "zh-CN") => {
            "按文件名、标题、标签或日期搜索"
        }
        ("Toggle document fullscreen", "es") => "Alternar pantalla completa",
        ("Toggle document fullscreen", "zh-CN") => "切换文档全屏",
        ("Show all keyboard shortcuts", "es") => "Mostrar todos los atajos",
        ("Show all keyboard shortcuts", "zh-CN") => "显示所有快捷键",
        ("Developed by ", "es") => "Desarrollado por ",
        ("Developed by ", "zh-CN") => "由 ",
        ("Quit", "es") => "Salir",
        ("Quit", "zh-CN") => "退出",
        ("Force quit", "es") => "Forzar salida",
        ("Force quit", "zh-CN") => "强制退出",

        _ => en,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn english_passes_through() {
        assert_eq!(t("Governance", "en"), "Governance");
        assert_eq!(t("anything not in the table", "en"), "anything not in the table");
    }

    #[test]
    fn unknown_lang_falls_back_to_english() {
        assert_eq!(t("Governance", "fr"), "Governance");
        assert_eq!(t("Governance", ""), "Governance");
    }

    #[test]
    fn missing_translation_falls_back_silently() {
        // A string the table has never heard of must not panic.
        assert_eq!(t("Some unknown label", "zh-CN"), "Some unknown label");
    }

    #[test]
    fn group_labels_translate() {
        assert_eq!(t("Governance", "es"), "Gobernanza");
        assert_eq!(t("Governance", "zh-CN"), "治理");
        assert_eq!(t("AI Audit", "es"), "Auditoría IA");
        assert_eq!(t("AI Audit", "zh-CN"), "AI 审计");
    }

    #[test]
    fn subgroup_labels_translate() {
        assert_eq!(t("Technical debt", "es"), "Deuda técnica");
        assert_eq!(t("Technical debt", "zh-CN"), "技术债务");
    }

    #[test]
    fn status_bar_keys_translate() {
        assert_eq!(t("quit", "es"), "salir");
        assert_eq!(t("quit", "zh-CN"), "退出");
        assert_eq!(t("search", "zh-CN"), "搜索");
    }

    #[test]
    fn help_popup_strings_translate() {
        assert_eq!(t("Keyboard Shortcuts", "es"), "Atajos de teclado");
        assert_eq!(t("Keyboard Shortcuts", "zh-CN"), "键盘快捷键");
        assert_eq!(t("Force quit", "zh-CN"), "强制退出");
    }
}
