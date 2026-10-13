// This application uses HTML/CSS rather than WebGL. Prefer WebKit's compatible
// renderer on Linux, where DMABuf/EGL can abort on some Mesa and NVIDIA setups.
// Preserve explicit user settings so GPU rendering can still be enabled.
const RENDER_DEFAULTS: &[(&str, &str)] = &[
    ("WEBKIT_DISABLE_DMABUF_RENDERER", "1"),
    ("WEBKIT_DISABLE_COMPOSITING_MODE", "1"),
];

pub fn configure() {
    for &(name, value) in RENDER_DEFAULTS {
        if std::env::var_os(name).is_none() {
            std::env::set_var(name, value);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compatible_defaults_preserve_explicit_overrides() {
        let saved: Vec<_> = RENDER_DEFAULTS
            .iter()
            .map(|(name, _)| (*name, std::env::var_os(name)))
            .collect();

        for &(name, _) in RENDER_DEFAULTS {
            std::env::remove_var(name);
        }
        configure();
        for &(name, value) in RENDER_DEFAULTS {
            assert_eq!(std::env::var(name).unwrap(), value);
        }

        for &(name, _) in RENDER_DEFAULTS {
            std::env::set_var(name, "0");
        }
        configure();
        for &(name, _) in RENDER_DEFAULTS {
            assert_eq!(std::env::var(name).unwrap(), "0");
        }

        for (name, value) in saved {
            match value {
                Some(value) => std::env::set_var(name, value),
                None => std::env::remove_var(name),
            }
        }
    }
}
