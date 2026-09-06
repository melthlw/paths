use std::collections::HashMap;
use std::sync::RwLock;

/// Information metadata for a supported or regional language
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LanguageInfo {
    pub lang: Language,
    pub code: &'static str,
    pub native_name: &'static str,
    pub localized_name: &'static str,
    pub region: &'static str,
}

/// Supported languages for Paths
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
pub enum Language {
    #[default]
    System,
    PtBr,
    En,
    Es,
    Fr,
    De,
    It,
    Ja,
    ZhCn,
    Ru,
}

impl Language {
    pub fn code(&self) -> &'static str {
        match self {
            Language::System => "system",
            Language::PtBr => "pt_BR",
            Language::En => "en",
            Language::Es => "es",
            Language::Fr => "fr",
            Language::De => "de",
            Language::It => "it",
            Language::Ja => "ja",
            Language::ZhCn => "zh_CN",
            Language::Ru => "ru",
        }
    }

    pub fn from_code(code: &str) -> Self {
        match code.trim().to_lowercase().as_str() {
            "pt" | "pt_br" | "pt-br" | "portuguese" => Language::PtBr,
            "en" | "en_us" | "en-us" | "en_gb" | "english" => Language::En,
            "es" | "es_es" | "es-es" | "spanish" => Language::Es,
            "fr" | "fr_fr" | "fr-fr" | "french" => Language::Fr,
            "de" | "de_de" | "de-de" | "german" => Language::De,
            "it" | "it_it" | "it-it" | "italian" => Language::It,
            "ja" | "ja_jp" | "ja-jp" | "japanese" => Language::Ja,
            "zh" | "zh_cn" | "zh-cn" | "chinese" => Language::ZhCn,
            "ru" | "ru_ru" | "ru-ru" | "russian" => Language::Ru,
            _ => Language::System,
        }
    }

    pub fn all_info() -> &'static [LanguageInfo] {
        &[
            LanguageInfo {
                lang: Language::System,
                code: "system",
                native_name: "System Default",
                localized_name: "System Default",
                region: "Automatic (GNOME)",
            },
            LanguageInfo {
                lang: Language::PtBr,
                code: "pt_BR",
                native_name: "Português (Brasil)",
                localized_name: "Portuguese (Brazil)",
                region: "Brasil",
            },
            LanguageInfo {
                lang: Language::En,
                code: "en",
                native_name: "English (US)",
                localized_name: "English",
                region: "International",
            },
            LanguageInfo {
                lang: Language::Es,
                code: "es",
                native_name: "Español",
                localized_name: "Spanish",
                region: "España / Latinoamérica",
            },
            LanguageInfo {
                lang: Language::Fr,
                code: "fr",
                native_name: "Français",
                localized_name: "French",
                region: "France / Canada",
            },
            LanguageInfo {
                lang: Language::De,
                code: "de",
                native_name: "Deutsch",
                localized_name: "German",
                region: "Deutschland",
            },
            LanguageInfo {
                lang: Language::It,
                code: "it",
                native_name: "Italiano",
                localized_name: "Italian",
                region: "Italia",
            },
            LanguageInfo {
                lang: Language::Ja,
                code: "ja",
                native_name: "日本語",
                localized_name: "Japanese",
                region: "日本",
            },
            LanguageInfo {
                lang: Language::ZhCn,
                code: "zh_CN",
                native_name: "中文 (简体)",
                localized_name: "Chinese (Simplified)",
                region: "中国",
            },
            LanguageInfo {
                lang: Language::Ru,
                code: "ru",
                native_name: "Русский",
                localized_name: "Russian",
                region: "Россия",
            },
        ]
    }

    pub fn info(&self) -> LanguageInfo {
        Self::all_info()
            .iter()
            .find(|i| i.lang == *self)
            .cloned()
            .unwrap_or(Self::all_info()[0].clone())
    }
}

/// Global I18n Manager with embedded catalogs for standalone/cargo execution and GNU gettext compatibility
pub struct I18nManager {
    configured_language: Language,
    effective_language: Language,
    catalogs: HashMap<String, HashMap<String, String>>,
}

static I18N: RwLock<Option<I18nManager>> = RwLock::new(None);

impl I18nManager {
    fn new() -> Self {
        let mut catalogs = HashMap::new();

        // Load embedded PO files compiled directly into binary
        let pt_br_po = include_str!("../../po/pt_BR.po");
        let en_po = include_str!("../../po/en.po");
        let es_po = include_str!("../../po/es.po");

        catalogs.insert("pt_BR".to_string(), parse_po_catalog(pt_br_po));
        catalogs.insert("en".to_string(), parse_po_catalog(en_po));
        catalogs.insert("es".to_string(), parse_po_catalog(es_po));

        // Load any external locale catalogs installed on system or in workspace
        Self::load_external_catalogs(&mut catalogs);

        let configured = Self::load_preference();
        let effective = if configured == Language::System {
            Self::detect_system_language()
        } else {
            configured
        };

        Self {
            configured_language: configured,
            effective_language: effective,
            catalogs,
        }
    }

    fn load_external_catalogs(catalogs: &mut HashMap<String, HashMap<String, String>>) {
        let mut paths_to_check = Vec::new();

        if let Ok(data_dirs) = std::env::var("XDG_DATA_DIRS") {
            for dir in data_dirs.split(':') {
                paths_to_check.push(std::path::PathBuf::from(dir).join("locale"));
            }
        } else {
            paths_to_check.push(std::path::PathBuf::from("/app/share/locale"));
            paths_to_check.push(std::path::PathBuf::from("/usr/share/locale"));
            paths_to_check.push(std::path::PathBuf::from("/usr/local/share/locale"));
        }

        if let Some(data_dir) = dirs::data_dir() {
            paths_to_check.push(data_dir.join("locale"));
        }
        paths_to_check.push(std::path::PathBuf::from("po"));

        for base in paths_to_check {
            if !base.exists() {
                continue;
            }

            for info in Language::all_info() {
                if info.lang == Language::System {
                    continue;
                }
                let code = info.code;
                let candidates = [
                    base.join(format!("{}.po", code)),
                    base.join(code).join("LC_MESSAGES").join("paths.po"),
                    base.join(code).join("LC_MESSAGES").join("gnome-paths.po"),
                ];

                for candidate in &candidates {
                    if candidate.is_file() {
                        if let Ok(content) = std::fs::read_to_string(candidate) {
                            let parsed = parse_po_catalog(&content);
                            if !parsed.is_empty() {
                                catalogs.entry(code.to_string()).or_default().extend(parsed);
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn detect_system_language() -> Language {
        let vars = ["LC_ALL", "LC_MESSAGES", "LANG"];
        for var in &vars {
            if let Ok(val) = std::env::var(var) {
                let low = val.to_lowercase();
                if low.starts_with("pt") || low.contains("pt_br") || low.contains("pt-br") {
                    return Language::PtBr;
                } else if low.starts_with("en") {
                    return Language::En;
                } else if low.starts_with("es") {
                    return Language::Es;
                } else if low.starts_with("fr") {
                    return Language::Fr;
                } else if low.starts_with("de") {
                    return Language::De;
                } else if low.starts_with("it") {
                    return Language::It;
                } else if low.starts_with("ja") {
                    return Language::Ja;
                } else if low.starts_with("zh") {
                    return Language::ZhCn;
                } else if low.starts_with("ru") {
                    return Language::Ru;
                }
            }
        }
        // Canonical international default is English
        Language::En
    }

    fn config_path() -> Option<std::path::PathBuf> {
        dirs::config_dir().map(|p| p.join("paths").join("language.toml"))
    }

    fn load_preference() -> Language {
        if let Some(path) = Self::config_path() {
            if let Ok(content) = std::fs::read_to_string(path) {
                #[derive(serde::Deserialize)]
                struct LangConf {
                    language: Option<String>,
                }
                if let Ok(conf) = toml::from_str::<LangConf>(&content) {
                    if let Some(code) = conf.language {
                        return Language::from_code(&code);
                    }
                }
            }
        }
        Language::System
    }

    fn save_preference(lang: Language) {
        if let Some(path) = Self::config_path() {
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let toml_str = format!("language = \"{}\"\n", lang.code());
            let _ = std::fs::write(path, toml_str);
        }
    }
}

/// Parses a GNU gettext PO file into a Key-Value translation map, supporting msgctxt
pub fn parse_po_catalog(content: &str) -> HashMap<String, String> {
    let mut catalog = HashMap::new();
    let mut current_ctxt: Option<String> = None;
    let mut current_msgid: Option<String> = None;
    let mut current_msgstr: Option<String> = None;
    let mut in_ctxt = false;
    let mut in_msgid = false;
    let mut in_msgstr = false;

    let commit_entry = |ctxt: &mut Option<String>,
                        id: &mut Option<String>,
                        s: &mut Option<String>,
                        cat: &mut HashMap<String, String>| {
        if let (Some(id_val), Some(s_val)) = (id.take(), s.take()) {
            if !id_val.is_empty() && !s_val.is_empty() {
                if let Some(c) = ctxt.take() {
                    cat.insert(format!("{}\x04{}", c, id_val), s_val.clone());
                    cat.entry(id_val).or_insert(s_val);
                } else {
                    cat.insert(id_val, s_val);
                }
            }
        }
    };

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            continue;
        }

        if trimmed.starts_with("msgctxt ") {
            commit_entry(
                &mut current_ctxt,
                &mut current_msgid,
                &mut current_msgstr,
                &mut catalog,
            );
            in_ctxt = true;
            in_msgid = false;
            in_msgstr = false;
            current_ctxt = Some(extract_string_literal(&trimmed[8..]));
        } else if trimmed.starts_with("msgid ") {
            if !in_ctxt {
                commit_entry(
                    &mut current_ctxt,
                    &mut current_msgid,
                    &mut current_msgstr,
                    &mut catalog,
                );
            }
            in_ctxt = false;
            in_msgid = true;
            in_msgstr = false;
            current_msgid = Some(extract_string_literal(&trimmed[6..]));
        } else if trimmed.starts_with("msgstr ") {
            in_ctxt = false;
            in_msgid = false;
            in_msgstr = true;
            current_msgstr = Some(extract_string_literal(&trimmed[7..]));
        } else if trimmed.starts_with('"') && trimmed.ends_with('"') {
            let chunk = extract_string_literal(trimmed);
            if in_ctxt {
                if let Some(ref mut c) = current_ctxt {
                    c.push_str(&chunk);
                }
            } else if in_msgid {
                if let Some(ref mut id) = current_msgid {
                    id.push_str(&chunk);
                }
            } else if in_msgstr {
                if let Some(ref mut s) = current_msgstr {
                    s.push_str(&chunk);
                }
            }
        } else if trimmed.is_empty() {
            commit_entry(
                &mut current_ctxt,
                &mut current_msgid,
                &mut current_msgstr,
                &mut catalog,
            );
            in_ctxt = false;
            in_msgid = false;
            in_msgstr = false;
        }
    }

    commit_entry(
        &mut current_ctxt,
        &mut current_msgid,
        &mut current_msgstr,
        &mut catalog,
    );

    catalog
}

fn extract_string_literal(line: &str) -> String {
    let t = line.trim();
    if t.starts_with('"') && t.ends_with('"') && t.len() >= 2 {
        let inner = &t[1..t.len() - 1];
        let mut res = String::with_capacity(inner.len());
        let mut chars = inner.chars().peekable();
        while let Some(c) = chars.next() {
            if c == '\\' {
                match chars.next() {
                    Some('n') => res.push('\n'),
                    Some('t') => res.push('\t'),
                    Some('r') => res.push('\r'),
                    Some('"') => res.push('"'),
                    Some('\\') => res.push('\\'),
                    Some(other) => {
                        res.push('\\');
                        res.push(other);
                    }
                    None => res.push('\\'),
                }
            } else {
                res.push(c);
            }
        }
        res
    } else {
        t.to_string()
    }
}

/// Initialize the i18n system
pub fn init() {
    let mut lock = I18N.write().unwrap();
    if lock.is_none() {
        *lock = Some(I18nManager::new());
    }
}

/// Translate a message id into the currently active language
pub fn gettext(msgid: &str) -> String {
    let mut lock = I18N.write().unwrap();
    let mgr = lock.get_or_insert_with(I18nManager::new);

    let lang_code = mgr.effective_language.code();
    if let Some(cat) = mgr.catalogs.get(lang_code) {
        if let Some(translated) = cat.get(msgid) {
            if !translated.is_empty() {
                return translated.clone();
            }
        }
    }

    // Fallback to msgid itself
    msgid.to_string()
}

/// Translate a message id with context into the currently active language
#[allow(dead_code)]
pub fn pgettext(context: &str, msgid: &str) -> String {
    let mut lock = I18N.write().unwrap();
    let mgr = lock.get_or_insert_with(I18nManager::new);

    let lang_code = mgr.effective_language.code();
    if let Some(cat) = mgr.catalogs.get(lang_code) {
        let key = format!("{}\x04{}", context, msgid);
        if let Some(translated) = cat.get(&key) {
            if !translated.is_empty() {
                return translated.clone();
            }
        }
        if let Some(translated) = cat.get(msgid) {
            if !translated.is_empty() {
                return translated.clone();
            }
        }
    }

    msgid.to_string()
}

/// Translate a singular or plural message based on count into the currently active language
#[allow(dead_code)]
pub fn ngettext(singular: &str, plural: &str, n: usize) -> String {
    let mut lock = I18N.write().unwrap();
    let mgr = lock.get_or_insert_with(I18nManager::new);

    let lang_code = mgr.effective_language.code();
    let target_msgid = if n == 1 { singular } else { plural };

    if let Some(cat) = mgr.catalogs.get(lang_code) {
        if let Some(translated) = cat.get(target_msgid) {
            if !translated.is_empty() {
                return translated.clone();
            }
        }
    }

    target_msgid.to_string()
}

type LanguageChangeCallback = Box<dyn Fn(Language) + Send + Sync + 'static>;
type LocalLanguageChangeCallback = Box<dyn Fn(Language) + 'static>;

static LANGUAGE_LISTENERS: RwLock<Vec<LanguageChangeCallback>> = RwLock::new(Vec::new());

thread_local! {
    static LOCAL_LANGUAGE_LISTENERS: std::cell::RefCell<Vec<LocalLanguageChangeCallback>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// Register a thread-local UI listener callback to be notified immediately whenever the language changes
pub fn on_language_change_local<F: Fn(Language) + 'static>(callback: F) {
    LOCAL_LANGUAGE_LISTENERS.with(|cell| {
        cell.borrow_mut().push(Box::new(callback));
    });
}

/// Set user preferred language, save preference, and notify all listeners
pub fn set_language(lang: Language) {
    {
        let mut lock = I18N.write().unwrap();
        let mgr = lock.get_or_insert_with(I18nManager::new);
        mgr.configured_language = lang;
        mgr.effective_language = if lang == Language::System {
            I18nManager::detect_system_language()
        } else {
            lang
        };
        I18nManager::save_preference(lang);
    }

    // Trigger all live UI listeners
    let lock = LANGUAGE_LISTENERS.read().unwrap();
    for listener in lock.iter() {
        listener(lang);
    }

    LOCAL_LANGUAGE_LISTENERS.with(|cell| {
        for listener in cell.borrow().iter() {
            listener(lang);
        }
    });
}

/// Get the currently configured language preference
pub fn get_language() -> Language {
    let mut lock = I18N.write().unwrap();
    let mgr = lock.get_or_insert_with(I18nManager::new);
    mgr.configured_language
}

/// Replaces `{}` placeholders sequentially in translated templates
pub fn format_i18n(template: &str, args: &[&dyn std::fmt::Display]) -> String {
    let mut res = template.to_string();
    for arg in args {
        if let Some(pos) = res.find("{}") {
            let arg_str = format!("{}", arg);
            res.replace_range(pos..pos + 2, &arg_str);
        }
    }
    res
}

/// Macro for internationalized strings: `i18n!("New Document")` or `i18n!("Saved to {}", path)`
#[macro_export]
macro_rules! i18n {
    ($msg:expr) => {
        $crate::core::i18n::gettext($msg)
    };
    ($msg:expr, $($arg:expr),+ $(,)?) => {
        $crate::core::i18n::format_i18n(
            &$crate::core::i18n::gettext($msg),
            &[ $( &$arg as &dyn std::fmt::Display ),+ ]
        )
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static I18N_TEST_MUTEX: Mutex<()> = Mutex::new(());

    #[test]
    fn test_po_parser() {
        let po = r#"
# Comment
msgid ""
msgstr ""
"Project-Id-Version: test\n"

msgid "Hello"
msgstr "Olá"

msgid "Multi "
"Line"
msgstr "Multi "
"Linha"
"#;
        let map = parse_po_catalog(po);
        assert_eq!(map.get("Hello").unwrap(), "Olá");
        assert_eq!(map.get("Multi Line").unwrap(), "Multi Linha");
    }

    #[test]
    fn test_embedded_catalogs_and_switching() {
        let _guard = I18N_TEST_MUTEX.lock().unwrap();
        init();

        set_language(Language::PtBr);
        assert_eq!(gettext("New Document"), "Novo Documento");
        assert_eq!(gettext("Save"), "Salvar");
        assert_eq!(gettext("Open Document"), "Abrir Documento");
        assert_eq!(gettext("Grid (Ctrl+G)"), "Grade (Ctrl+G)");
        assert_eq!(gettext("Path & Node Editor"), "Editor de Caminho e Nós");
        assert_eq!(gettext("Corner Node"), "Nó de Canto");
        assert_eq!(gettext("Smooth Node"), "Nó Suave");
        assert_eq!(gettext("Direct Curve Dragging"), "Arraste Direto de Curvas");

        set_language(Language::En);
        assert_eq!(gettext("New Document"), "New Document");
        assert_eq!(gettext("Save"), "Save");
        assert_eq!(gettext("Open Document"), "Open Document");
        assert_eq!(gettext("Path & Node Editor"), "Path & Node Editor");
        assert_eq!(gettext("Corner Node"), "Corner Node");
        assert_eq!(gettext("Smooth Node"), "Smooth Node");
        assert_eq!(gettext("Direct Curve Dragging"), "Direct Curve Dragging");

        // Reset to PtBr for test suite
        set_language(Language::PtBr);
    }

    #[test]
    fn test_missing_key_fallback() {
        let _guard = I18N_TEST_MUTEX.lock().unwrap();
        init();
        let unknown = "Some completely unknown string 12345";
        assert_eq!(gettext(unknown), unknown);
    }

    #[test]
    fn test_pgettext_and_context() {
        let po = r#"
msgid ""
msgstr ""
"Project-Id-Version: test\n"

msgid "Open"
msgstr "Abrir"

msgctxt "Path Status"
msgid "Open"
msgstr "Aberto"
"#;
        let map = parse_po_catalog(po);
        assert_eq!(map.get("Open").unwrap(), "Abrir");
        assert_eq!(map.get("Path Status\x04Open").unwrap(), "Aberto");
    }

    #[test]
    fn test_ngettext() {
        let _guard = I18N_TEST_MUTEX.lock().unwrap();
        init();
        set_language(Language::PtBr);
        assert_eq!(ngettext("Save", "Save", 1), "Salvar");
        assert_eq!(ngettext("Save", "Save", 2), "Salvar");
    }

    #[test]
    fn test_spanish_catalog() {
        let _guard = I18N_TEST_MUTEX.lock().unwrap();
        init();
        set_language(Language::Es);
        assert_eq!(gettext("File"), "Archivo");
        assert_eq!(gettext("Edit"), "Editar");
        assert_eq!(gettext("Save"), "Guardar");
        assert_eq!(gettext("Undo"), "Deshacer");
        assert_eq!(gettext("Trace Bitmap"), "Vectorizar mapa de bits");
        assert_eq!(gettext("Advanced Preview..."), "Vista previa avanzada...");

        // Reset to PtBr
        set_language(Language::PtBr);
    }
}
