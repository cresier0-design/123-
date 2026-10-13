// Keep accelerated compositing enabled by default. Disabling it makes CSS
// painting and scrolling expensive. The non-DMABuf path is a narrower driver
// workaround; the software fallback is opt-in for machines with EGL errors.
const BALANCED: &[(&str, &str)] = &[
    ("WEBKIT_DISABLE_DMABUF_RENDERER", "1"),
    ("WEBKIT_DISABLE_COMPOSITING_MODE", "0"),
];
const GPU: &[(&str, &str)] = &[
    ("WEBKIT_DISABLE_DMABUF_RENDERER", "0"),
    ("WEBKIT_DISABLE_COMPOSITING_MODE", "0"),
];
const SOFTWARE: &[(&str, &str)] = &[
    ("WEBKIT_DISABLE_DMABUF_RENDERER", "1"),
    ("WEBKIT_DISABLE_COMPOSITING_MODE", "1"),
];

fn defaults(mode: &str) -> &'static [(&'static str, &'static str)] {
    match mode {
        "software" => SOFTWARE,
        "gpu" => GPU,
        _ => BALANCED,
    }
}

pub fn configure() {
    let mode = std::env::var("CGS_RENDERER").unwrap_or_default();
    for &(name, value) in defaults(&mode) {
        if std::env::var_os(name).is_none() {
            std::env::set_var(name, value);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accelerated_default_and_software_fallback_preserve_overrides() {
        assert_eq!(defaults(""), BALANCED);
        assert_eq!(defaults("unknown"), BALANCED);
        assert_eq!(defaults("gpu"), GPU);
        assert_eq!(defaults("software"), SOFTWARE);
        assert_eq!(BALANCED[1].1, "0");
        assert_eq!(SOFTWARE[1].1, "1");

        let saved: Vec<_> = BALANCED
            .iter()
            .map(|(name, _)| (*name, std::env::var_os(name)))
            .collect();

        let saved_mode = std::env::var_os("CGS_RENDERER");
        std::env::remove_var("CGS_RENDERER");
        for &(name, _) in BALANCED {
            std::env::remove_var(name);
        }
        configure();
        for &(name, value) in BALANCED {
            assert_eq!(std::env::var(name).unwrap(), value);
        }

        for &(name, _) in BALANCED {
            std::env::set_var(name, "0");
        }
        configure();
        for &(name, _) in BALANCED {
            assert_eq!(std::env::var(name).unwrap(), "0");
        }

        std::env::set_var("CGS_RENDERER", "software");
        for &(name, _) in SOFTWARE {
            std::env::remove_var(name);
        }
        configure();
        for &(name, value) in SOFTWARE {
            assert_eq!(std::env::var(name).unwrap(), value);
        }

        for (name, value) in saved {
            match value {
                Some(value) => std::env::set_var(name, value),
                None => std::env::remove_var(name),
            }
        }
        match saved_mode {
            Some(value) => std::env::set_var("CGS_RENDERER", value),
            None => std::env::remove_var("CGS_RENDERER"),
        }
    }
}
