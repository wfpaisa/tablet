//! Locale lookup with English fallback and classic debranding.
use std::collections::BTreeMap;
#[derive(Clone, Default)]
pub struct Locale {
    entries: BTreeMap<String, String>,
    cjk: bool,
}
impl Locale {
    pub fn new(language: &str) -> Self {
        let source = match language {
            "de" => include_str!("../../../assets/locales/de.json"),
            "fr" => include_str!("../../../assets/locales/fr.json"),
            "nl" => include_str!("../../../assets/locales/nl.json"),
            "ja" => include_str!("../../../assets/locales/ja.json"),
            "zh-CN" => include_str!("../../../assets/locales/zh-CN.json"),
            "pl" => include_str!("../../../assets/locales/pl.json"),
            "zh-TW" => include_str!("../../../assets/locales/zh-TW.json"),
            "es" => include_str!("../../../assets/locales/es.json"),
            "ru" => include_str!("../../../assets/locales/ru.json"),
            "ko" => include_str!("../../../assets/locales/ko.json"),
            "tr" => include_str!("../../../assets/locales/tr.json"),
            "it" => include_str!("../../../assets/locales/it.json"),
            "pt" => include_str!("../../../assets/locales/pt.json"),
            "en" => include_str!("../../../assets/locales/en.json"),
            _ => "{}",
        };
        Self {
            entries: serde_json::from_str(source).unwrap_or_default(),
            cjk: matches!(language, "ja" | "ko" | "zh-CN" | "zh-TW"),
        }
    }
    pub fn is_cjk(&self) -> bool {
        self.cjk
    }
    /// Text drops the brand name, as `t()` does in app.js.
    pub fn t(&self, key: &str) -> String {
        self.entries
            .get(key)
            .map(String::as_str)
            .unwrap_or(key)
            .pipe_debrand()
    }
}
trait ClassicText {
    fn pipe_debrand(self) -> String;
}
impl ClassicText for &str {
    fn pipe_debrand(self) -> String {
        let mut result = String::new();
        let mut start = 0;
        for (index, _) in self.char_indices() {
            if index < start {
                continue;
            }
            let Some(word) = self.get(index..index + 5) else {
                continue;
            };
            if !word.eq_ignore_ascii_case("wacom") {
                continue;
            }
            let before = self[..index].chars().next_back();
            let after = self[index + 5..].chars().next();
            if before.is_some_and(|c| c.is_ascii_alphanumeric() || c == '_')
                || after.is_some_and(|c| c.is_ascii_alphanumeric() || c == '_')
            {
                continue;
            }
            let tail = &self[index + 5..];
            if tail
                .get(..4)
                .is_some_and(|s| s.eq_ignore_ascii_case(" one"))
                && !tail[4..]
                    .chars()
                    .next()
                    .is_some_and(|c| c.is_ascii_alphanumeric() || c == '_')
            {
                continue;
            }
            result.push_str(&self[start..index]);
            result.push_str(if word == "WACOM" {
                "TABLET"
            } else if word.starts_with('W') {
                "Tablet"
            } else {
                "tablet"
            });
            start = index + 5;
        }
        result.push_str(&self[start..]);
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fallback_and_debrand() {
        let l = Locale::new("en");
        assert_eq!(l.t("Wacom Pen"), "Tablet Pen");
        assert_eq!(l.t("Tip feel"), "Pressure to click");
        assert_ne!(Locale::new("es").t("Devices"), "Devices");
        assert_eq!(
            l.t("Wacom One / WACOM / wacom / Awacom"),
            "Wacom One / TABLET / tablet / Awacom"
        );
    }
}
