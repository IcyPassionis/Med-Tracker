use iced::theme::Palette;
use iced::{Color, Theme};
use std::collections::{HashMap, HashSet};
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

const GTK_VERSIONS: [&str; 2] = ["gtk-4.0", "gtk-3.0"];
const MAX_IMPORTED_FILES: usize = 64;

const BACKGROUND_NAMES: &[&str] = &[
    "window_bg_color",
    "view_bg_color",
    "headerbar_bg_color",
    "card_bg_color",
    "popover_bg_color",
    "menu_bg_color",
    "theme_bg_color",
    "theme_base_color",
    "base_color",
    "background_color",
    "bg_color",
    "base_background_color",
    "content_view_bg_color",
];

const TEXT_NAMES: &[&str] = &[
    "window_fg_color",
    "view_fg_color",
    "headerbar_fg_color",
    "card_fg_color",
    "popover_fg_color",
    "theme_fg_color",
    "fg_color",
    "theme_text_color",
    "base_fg_color",
    "text_color",
    "foreground_color",
    "content_view_fg_color",
];

const PRIMARY_NAMES: &[&str] = &[
    "accent_bg_color",
    "accent_color",
    "selected_bg_color",
    "theme_selected_bg_color",
    "link_color",
    "primary_color",
    "suggested_action_color",
    "blue_1",
    "blue_2",
    "blue_3",
];

const SUCCESS_NAMES: &[&str] = &[
    "success_bg_color",
    "success_color",
    "success",
    "green_color",
    "green_1",
    "green_2",
    "green_3",
    "positive_color",
];

const WARNING_NAMES: &[&str] = &[
    "warning_bg_color",
    "warning_color",
    "warning",
    "yellow_color",
    "yellow_1",
    "yellow_2",
    "yellow_3",
    "caution_color",
];

const DANGER_NAMES: &[&str] = &[
    "destructive_bg_color",
    "destructive_color",
    "error_bg_color",
    "error_color",
    "danger_bg_color",
    "danger_color",
    "red_color",
    "red_1",
    "negative_color",
];

pub fn load() -> Option<Theme> {
    let config_roots = config_roots();
    let data_roots = data_roots();
    let gtk_theme = env::var("GTK_THEME").ok();

    load_from_roots(&config_roots, &data_roots, gtk_theme.as_deref())
}

struct ConfiguredTheme {
    name: String,
    prefer_dark: bool,
}

fn load_from_roots(
    config_roots: &[PathBuf],
    data_roots: &[PathBuf],
    gtk_theme: Option<&str>,
) -> Option<Theme> {
    for version in GTK_VERSIONS {
        let Some(configured_theme) = configured_theme(version, config_roots, gtk_theme) else {
            continue;
        };

        for css_path in find_theme_css(
            &configured_theme.name,
            version,
            configured_theme.prefer_dark,
            data_roots,
        ) {
            match load_theme(&configured_theme.name, &css_path) {
                Ok(theme) => return Some(theme),
                Err(error) => eprintln!(
                    "Failed to read GTK theme '{}' from {}: {}",
                    configured_theme.name,
                    css_path.display(),
                    error
                ),
            }
        }
    }

    None
}

fn configured_theme(
    version: &str,
    config_roots: &[PathBuf],
    gtk_theme: Option<&str>,
) -> Option<ConfiguredTheme> {
    if let Some(theme) = gtk_theme {
        let mut parts = theme.split(':');
        let theme = parts.next().unwrap_or_default().trim();
        if !theme.is_empty() {
            return Some(ConfiguredTheme {
                name: theme.to_string(),
                prefer_dark: parts.any(|variant| variant.eq_ignore_ascii_case("dark")),
            });
        }
    }

    config_roots.iter().find_map(|root| {
        let path = root.join(version).join("settings.ini");
        read_settings(&path)
    })
}

fn read_settings(path: &Path) -> Option<ConfiguredTheme> {
    let contents = fs::read_to_string(path).ok()?;
    let mut section = String::new();
    let mut theme_name = None;
    let mut theme_variant_is_dark = false;
    let mut application_prefer_dark = None;
    let mut interface_color_scheme = None;

    for line in contents.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            section = line[1..line.len() - 1].trim().to_ascii_lowercase();
            continue;
        }
        if section != "settings" {
            continue;
        }

        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim();
        let value = value.trim().trim_matches(['"', '\'']);
        if key.eq_ignore_ascii_case("gtk-theme-name") {
            let mut parts = value.split(':');
            let theme = parts.next().unwrap_or_default().trim();
            if !theme.is_empty() {
                theme_name = Some(theme.to_string());
                theme_variant_is_dark = parts.any(|variant| variant.eq_ignore_ascii_case("dark"));
            }
        } else if key.eq_ignore_ascii_case("gtk-application-prefer-dark-theme") {
            application_prefer_dark = Some(value.eq_ignore_ascii_case("true") || value == "1");
        } else if key.eq_ignore_ascii_case("gtk-interface-color-scheme") {
            interface_color_scheme = Some(value.eq_ignore_ascii_case("prefer-dark"));
        }
    }

    theme_name.map(|name| ConfiguredTheme {
        name,
        prefer_dark: interface_color_scheme
            .or(application_prefer_dark)
            .unwrap_or(theme_variant_is_dark),
    })
}

fn find_theme_css(
    theme_name: &str,
    version: &str,
    prefer_dark: bool,
    data_roots: &[PathBuf],
) -> Vec<PathBuf> {
    let css_files: &[&str] = if prefer_dark {
        &[
            "gtk-dark.css",
            "gtk-contained-dark.css",
            "gtk.css",
            "gtk-contained.css",
        ]
    } else {
        &["gtk.css", "gtk-contained.css", "gtk-dark.css"]
    };

    let mut paths = Vec::new();
    for root in data_roots {
        let theme_root = root.join(theme_name).join(version);
        for file in css_files {
            let path = theme_root.join(file);
            if path.is_file() {
                push_unique(&mut paths, path);
            }
        }
    }
    paths
}

fn config_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if let Some(root) = dirs::config_dir() {
        push_unique(&mut roots, root);
    }
    if let Some(path_list) = env::var_os("XDG_CONFIG_DIRS") {
        for root in env::split_paths(&path_list) {
            push_unique(&mut roots, root);
        }
    }
    push_unique(&mut roots, PathBuf::from("/etc"));
    roots
}

fn data_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if let Some(root) = dirs::data_dir() {
        push_unique(&mut roots, root.join("themes"));
        push_unique(&mut roots, root);
    }
    if let Some(path_list) = env::var_os("XDG_DATA_DIRS") {
        for root in env::split_paths(&path_list) {
            push_unique(&mut roots, root.join("themes"));
            push_unique(&mut roots, root);
        }
    }
    if let Some(home) = dirs::home_dir() {
        push_unique(&mut roots, home.join(".themes"));
    }
    push_unique(&mut roots, PathBuf::from("/usr/local/share/themes"));
    push_unique(&mut roots, PathBuf::from("/usr/share/themes"));
    push_unique(&mut roots, PathBuf::from("/usr/local/share"));
    push_unique(&mut roots, PathBuf::from("/usr/share"));
    roots
}

fn push_unique(paths: &mut Vec<PathBuf>, path: PathBuf) {
    if !paths.iter().any(|existing| existing == &path) {
        paths.push(path);
    }
}

fn load_theme(theme_name: &str, css_path: &Path) -> io::Result<Theme> {
    let mut colors = HashMap::new();
    let mut loaded_files = HashSet::new();
    load_css_file(css_path, &mut colors, &mut loaded_files)?;

    theme_from_colors(theme_name, &colors).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "GTK theme does not contain enough resolvable colors",
        )
    })
}

fn load_css_file(
    path: &Path,
    colors: &mut HashMap<String, String>,
    loaded_files: &mut HashSet<PathBuf>,
) -> io::Result<()> {
    if loaded_files.len() >= MAX_IMPORTED_FILES {
        return Ok(());
    }

    let canonical_path = fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    if !loaded_files.insert(canonical_path) {
        return Ok(());
    }

    let contents = fs::read_to_string(path)?;
    let contents = strip_css_comments(&contents);

    for import in css_imports(&contents) {
        let import_path = path.parent().unwrap_or(Path::new(".")).join(import);
        if import_path.is_file() {
            load_css_file(&import_path, colors, loaded_files)?;
        }
    }

    parse_css_declarations(&contents, colors);
    Ok(())
}

fn css_imports(contents: &str) -> Vec<PathBuf> {
    contents
        .split(';')
        .filter_map(|statement| {
            let statement = statement.trim();
            let rest = statement.strip_prefix("@import")?.trim();
            let rest = rest.strip_prefix("url(").unwrap_or(rest).trim();
            let rest = rest.strip_suffix(')').unwrap_or(rest).trim();
            let quote = rest.chars().next()?;
            if quote != '\'' && quote != '"' {
                return None;
            }
            let end = rest[1..].find(quote)? + 1;
            let import = &rest[1..end];
            if import.starts_with("resource://") || Path::new(import).is_absolute() {
                None
            } else {
                Some(PathBuf::from(import))
            }
        })
        .collect()
}

fn parse_css_declarations(contents: &str, colors: &mut HashMap<String, String>) {
    for statement in contents.split(';') {
        let statement = statement.trim();
        if let Some(rest) = statement.strip_prefix("@define-color") {
            let mut parts = rest.trim().splitn(2, char::is_whitespace);
            let Some(name) = parts.next() else {
                continue;
            };
            let Some(value) = parts.next() else {
                continue;
            };
            colors.insert(normalize_name(name), value.trim().to_string());
            continue;
        }

        let Some((left, value)) = statement.rsplit_once(':') else {
            continue;
        };
        let Some(name) = left
            .rsplit(|character| character == '{' || character == '}')
            .next()
            .and_then(|part| part.split_whitespace().last())
        else {
            continue;
        };
        if name.trim_start().starts_with("--") {
            colors.insert(normalize_name(name), value.trim().to_string());
        }
    }
}

fn strip_css_comments(contents: &str) -> String {
    let mut output = String::with_capacity(contents.len());
    let mut characters = contents.chars().peekable();
    let mut in_comment = false;

    while let Some(character) = characters.next() {
        if in_comment {
            if character == '*' && characters.peek() == Some(&'/') {
                characters.next();
                in_comment = false;
            }
        } else if character == '/' && characters.peek() == Some(&'*') {
            characters.next();
            in_comment = true;
        } else {
            output.push(character);
        }
    }

    output
}

fn theme_from_colors(theme_name: &str, colors: &HashMap<String, String>) -> Option<Theme> {
    let background = lookup_color(colors, &BACKGROUND_NAMES)?;
    let text = lookup_color(colors, &TEXT_NAMES)?;
    let primary = lookup_color(colors, &PRIMARY_NAMES)?;
    let is_dark = luminance(background) < 0.5;
    let fallback = if is_dark {
        Palette::DARK
    } else {
        Palette::LIGHT
    };

    let palette = Palette {
        background,
        text,
        primary,
        success: lookup_color(colors, &SUCCESS_NAMES).unwrap_or(fallback.success),
        warning: lookup_color(colors, &WARNING_NAMES).unwrap_or(fallback.warning),
        danger: lookup_color(colors, &DANGER_NAMES).unwrap_or(fallback.danger),
    };

    Some(Theme::custom(format!("GTK: {theme_name}"), palette))
}

fn lookup_color(colors: &HashMap<String, String>, names: &[&str]) -> Option<Color> {
    names.iter().find_map(|name| {
        let mut resolving = HashSet::new();
        resolve_color(name, colors, &mut resolving)
    })
}

fn resolve_color(
    name: &str,
    colors: &HashMap<String, String>,
    resolving: &mut HashSet<String>,
) -> Option<Color> {
    let normalized = normalize_name(name);
    if !resolving.insert(normalized.clone()) {
        return None;
    }
    let Some(value) = colors.get(&normalized).cloned() else {
        resolving.remove(&normalized);
        return None;
    };
    let color = parse_color(&value, colors, resolving);
    resolving.remove(&normalized);
    color
}

fn parse_color(
    value: &str,
    colors: &HashMap<String, String>,
    resolving: &mut HashSet<String>,
) -> Option<Color> {
    let value = value
        .trim()
        .strip_suffix("!important")
        .unwrap_or(value.trim())
        .trim();
    if let Some(name) = value.strip_prefix('@') {
        return resolve_color(name, colors, resolving);
    }
    if let Some(inner) = function_inner(value, "var") {
        let arguments = split_function_arguments(inner);
        let name = arguments.first()?.trim();
        return resolve_color(name, colors, resolving).or_else(|| {
            arguments
                .get(1)
                .and_then(|fallback| parse_color(fallback, colors, resolving))
        });
    }
    if let Some(inner) = function_inner(value, "shade") {
        let arguments = split_function_arguments(inner);
        let color = parse_color(arguments.first()?, colors, resolving)?;
        let factor = arguments.get(1)?.trim().parse::<f32>().ok()?;
        return Some(Color::from_rgba(
            (color.r * factor).clamp(0.0, 1.0),
            (color.g * factor).clamp(0.0, 1.0),
            (color.b * factor).clamp(0.0, 1.0),
            color.a,
        ));
    }
    if let Some(inner) = function_inner(value, "alpha") {
        let arguments = split_function_arguments(inner);
        let mut color = parse_color(arguments.first()?, colors, resolving)?;
        color.a = parse_alpha(arguments.get(1)?)?;
        return Some(color);
    }
    if let Some(inner) = function_inner(value, "mix") {
        let arguments = split_function_arguments(inner);
        let first = parse_color(arguments.first()?, colors, resolving)?;
        let second = parse_color(arguments.get(1)?, colors, resolving)?;
        let amount = arguments
            .get(2)
            .and_then(|value| value.trim().parse::<f32>().ok())
            .unwrap_or(0.5)
            .clamp(0.0, 1.0);
        return Some(Color::from_rgba(
            first.r + (second.r - first.r) * amount,
            first.g + (second.g - first.g) * amount,
            first.b + (second.b - first.b) * amount,
            first.a + (second.a - first.a) * amount,
        ));
    }
    if let Some(color) = parse_hex_color(value) {
        return Some(color);
    }
    if let Some(inner) = function_inner(value, "rgb").or_else(|| function_inner(value, "rgba")) {
        let mut arguments = split_function_arguments(inner)
            .into_iter()
            .flat_map(|argument| {
                argument
                    .replace('/', " ")
                    .split_whitespace()
                    .map(str::to_string)
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        if arguments.len() < 3 {
            return None;
        }
        let alpha = if arguments.len() > 3 {
            parse_alpha(&arguments.remove(3))?
        } else {
            1.0
        };
        return Some(Color::from_rgba(
            parse_channel(&arguments[0])?,
            parse_channel(&arguments[1])?,
            parse_channel(&arguments[2])?,
            alpha,
        ));
    }
    match value.to_ascii_lowercase().as_str() {
        "black" => Some(Color::BLACK),
        "white" => Some(Color::WHITE),
        "transparent" => Some(Color::TRANSPARENT),
        _ => resolve_color(value, colors, resolving),
    }
}

fn function_inner<'a>(value: &'a str, name: &str) -> Option<&'a str> {
    let rest = value.get(name.len()..)?.trim_start();
    if !value[..name.len()].eq_ignore_ascii_case(name)
        || !rest.starts_with('(')
        || !rest.ends_with(')')
        || rest.len() <= 2
    {
        return None;
    }
    Some(&rest[1..rest.len() - 1])
}

fn split_function_arguments(value: &str) -> Vec<&str> {
    let mut arguments = Vec::new();
    let mut depth = 0;
    let mut start = 0;

    for (index, character) in value.char_indices() {
        match character {
            '(' => depth += 1,
            ')' => depth -= 1,
            ',' if depth == 0 => {
                arguments.push(value[start..index].trim());
                start = index + character.len_utf8();
            }
            _ => {}
        }
    }
    arguments.push(value[start..].trim());
    arguments
}

fn parse_hex_color(value: &str) -> Option<Color> {
    let digits = value.strip_prefix('#')?;
    let expand = |digit: u8| (digit << 4) | digit;
    match digits.len() {
        3 | 4 => {
            let values = digits
                .chars()
                .map(|digit| digit.to_digit(16).map(|value| expand(value as u8)))
                .collect::<Option<Vec<_>>>()?;
            let alpha = if values.len() == 4 {
                values[3] as f32 / 255.0
            } else {
                1.0
            };
            Some(Color::from_rgba8(values[0], values[1], values[2], alpha))
        }
        6 | 8 => {
            let value = u32::from_str_radix(digits, 16).ok()?;
            let red = (value >> if digits.len() == 8 { 24 } else { 16 }) as u8;
            let green = (value >> if digits.len() == 8 { 16 } else { 8 }) as u8;
            let blue = (value >> if digits.len() == 8 { 8 } else { 0 }) as u8;
            let alpha = if digits.len() == 8 {
                (value & 0xff) as f32 / 255.0
            } else {
                1.0
            };
            Some(Color::from_rgba8(red, green, blue, alpha))
        }
        _ => None,
    }
}

fn parse_channel(value: &str) -> Option<f32> {
    if let Some(value) = value.strip_suffix('%') {
        return value
            .parse::<f32>()
            .ok()
            .map(|value| (value / 100.0).clamp(0.0, 1.0));
    }
    value
        .parse::<f32>()
        .ok()
        .map(|value| (value / 255.0).clamp(0.0, 1.0))
}

fn parse_alpha(value: &str) -> Option<f32> {
    if let Some(value) = value.trim().strip_suffix('%') {
        return value
            .parse::<f32>()
            .ok()
            .map(|value| (value / 100.0).clamp(0.0, 1.0));
    }
    value
        .trim()
        .parse::<f32>()
        .ok()
        .map(|value| value.clamp(0.0, 1.0))
}

fn normalize_name(name: &str) -> String {
    name.trim()
        .trim_start_matches('@')
        .trim_start_matches("--")
        .replace('_', "-")
        .to_ascii_lowercase()
}

fn luminance(color: Color) -> f32 {
    0.2126 * color.r + 0.7152 * color.g + 0.0722 * color.b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_supported_color_formats() {
        assert_eq!(
            parse_hex_color("#abc"),
            Some(Color::from_rgb8(0xaa, 0xbb, 0xcc))
        );
        assert_eq!(
            parse_hex_color("#11223380"),
            Some(Color::from_rgba8(0x11, 0x22, 0x33, 128.0 / 255.0))
        );
        assert_eq!(
            parse_color("rgb(255, 128, 0)", &HashMap::new(), &mut HashSet::new()),
            Some(Color::from_rgb8(255, 128, 0))
        );
        assert_eq!(
            parse_color(
                "rgba(100% 0% 50% / 25%)",
                &HashMap::new(),
                &mut HashSet::new()
            ),
            Some(Color::from_rgba(1.0, 0.0, 0.5, 0.25))
        );
    }

    #[test]
    fn resolves_aliases_references_and_gtk_transforms() {
        let mut colors = HashMap::new();
        parse_css_declarations(
            r#"
                @define-color theme_bg_color #101820;
                @define-color theme_fg_color @foreground;
                @define-color foreground rgb(240, 240, 240);
                @define-color accent_color shade(#204060, 1.5);
                :root { --danger-color: #aa0000; }
            "#,
            &mut colors,
        );

        assert_eq!(
            lookup_color(&colors, &["theme_bg_color"]),
            Some(Color::from_rgb8(0x10, 0x18, 0x20))
        );
        assert_eq!(
            lookup_color(&colors, &["theme_fg_color"]),
            Some(Color::from_rgb8(240, 240, 240))
        );
        let accent = lookup_color(&colors, &["accent_color"]).expect("accent should resolve");
        assert!((accent.r - 0x30 as f32 / 255.0).abs() < f32::EPSILON);
        assert!((accent.g - 0x60 as f32 / 255.0).abs() < f32::EPSILON);
        assert!((accent.b - 0x90 as f32 / 255.0).abs() < f32::EPSILON);
        assert_eq!(
            lookup_color(&colors, &["danger_color"]),
            Some(Color::from_rgb8(0xaa, 0, 0))
        );
    }

    #[test]
    fn builds_custom_theme_from_gtk_roles() {
        let mut colors = HashMap::new();
        parse_css_declarations(
            r#"
                @define-color window_bg_color #fafafa;
                @define-color window_fg_color #202020;
                @define-color accent_bg_color #3465a4;
                @define-color success_color #26a269;
                @define-color warning_color #e5a50a;
                @define-color destructive_color #c01c28;
            "#,
            &mut colors,
        );

        let theme = theme_from_colors("Test", &colors).expect("theme should resolve");
        assert_eq!(
            theme.palette().background,
            Color::from_rgb8(0xfa, 0xfa, 0xfa)
        );
        assert!(!theme.extended_palette().is_dark);
        assert_eq!(
            theme.extended_palette().danger.base.color,
            Color::from_rgb8(0xc0, 0x1c, 0x28)
        );
    }

    #[test]
    fn imports_are_loaded_before_local_overrides() {
        let directory = env::temp_dir().join(format!(
            "med-tracker-gtk-theme-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be valid")
                .as_nanos()
        ));
        fs::create_dir_all(&directory).expect("test directory should be created");
        fs::write(
            directory.join("base.css"),
            "@define-color window_bg_color #ffffff; @define-color window_fg_color #000000;",
        )
        .expect("base stylesheet should be written");
        fs::write(
            directory.join("gtk.css"),
            "@import \"base.css\"; @define-color accent_color #123456;",
        )
        .expect("main stylesheet should be written");

        let mut colors = HashMap::new();
        let mut loaded = HashSet::new();
        load_css_file(&directory.join("gtk.css"), &mut colors, &mut loaded)
            .expect("stylesheet should load");
        let theme = theme_from_colors("Test", &colors).expect("imported colors should resolve");

        assert_eq!(theme.palette().background, Color::WHITE);
        assert_eq!(theme.palette().primary, Color::from_rgb8(0x12, 0x34, 0x56));
        fs::remove_dir_all(directory).expect("test directory should be removed");
    }

    #[test]
    fn resolves_config_and_data_roots_without_host_theme_state() {
        let directory = env::temp_dir().join(format!(
            "med-tracker-gtk-roots-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be valid")
                .as_nanos()
        ));
        let config_root = directory.join("config");
        let theme_root = directory.join("data/themes/Adwaita/gtk-4.0");
        fs::create_dir_all(config_root.join("gtk-4.0"))
            .expect("config directory should be created");
        fs::create_dir_all(&theme_root).expect("theme directory should be created");
        fs::write(
            config_root.join("gtk-4.0/settings.ini"),
            "[Settings]\ngtk-theme-name=Adwaita\n",
        )
        .expect("settings should be written");
        fs::write(
            theme_root.join("gtk.css"),
            "@define-color window_bg_color #ffffff; @define-color window_fg_color #000000; @define-color accent_color #123456;",
        )
        .expect("stylesheet should be written");

        let theme = load_from_roots(&[config_root], &[directory.join("data/themes")], None)
            .expect("theme should resolve from injected roots");

        assert_eq!(theme.palette().primary, Color::from_rgb8(0x12, 0x34, 0x56));
        fs::remove_dir_all(directory).expect("test directory should be removed");
    }

    #[test]
    fn tries_contained_css_when_main_css_has_no_local_colors() {
        let directory = env::temp_dir().join(format!(
            "med-tracker-gtk-contained-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be valid")
                .as_nanos()
        ));
        let theme_root = directory.join("Adwaita/gtk-4.0");
        fs::create_dir_all(&theme_root).expect("theme directory should be created");
        fs::write(
            theme_root.join("gtk.css"),
            "@import url(\"resource:///org/gnome/Adwaita/gtk-contained.css\");",
        )
        .expect("main stylesheet should be written");
        fs::write(
            theme_root.join("gtk-contained.css"),
            "@define-color window_bg_color #ffffff; @define-color window_fg_color #000000; @define-color accent_color #123456;",
        )
        .expect("contained stylesheet should be written");

        let theme = load_from_roots(&[], &[directory.clone()], Some("Adwaita"))
            .expect("contained stylesheet should resolve");

        assert_eq!(theme.palette().primary, Color::from_rgb8(0x12, 0x34, 0x56));
        fs::remove_dir_all(directory).expect("test directory should be removed");
    }

    #[test]
    fn rejects_themes_without_required_roles() {
        let mut colors = HashMap::new();
        parse_css_declarations(
            "@define-color window_bg_color #ffffff; @define-color window_fg_color #000000;",
            &mut colors,
        );

        assert!(theme_from_colors("Incomplete", &colors).is_none());
    }

    #[test]
    fn breaks_cyclic_color_references() {
        let mut colors = HashMap::new();
        parse_css_declarations(
            "@define-color first @second; @define-color second @first;",
            &mut colors,
        );

        assert_eq!(lookup_color(&colors, &["first"]), None);
    }

    #[test]
    fn reads_dark_preference_from_settings() {
        let path = env::temp_dir().join(format!(
            "med-tracker-gtk-settings-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be valid")
                .as_nanos()
        ));
        fs::write(
            &path,
            "[Settings]\ngtk-theme-name=Adwaita\ngtk-application-prefer-dark-theme=false\ngtk-interface-color-scheme=prefer-dark\n",
        )
        .expect("settings should be written");

        let settings = read_settings(&path).expect("settings should resolve");
        assert_eq!(settings.name, "Adwaita");
        assert!(settings.prefer_dark);
        fs::remove_file(path).expect("settings should be removed");
    }

    #[test]
    fn reads_dark_variant_from_theme_name() {
        let path = env::temp_dir().join(format!(
            "med-tracker-gtk-settings-variant-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be valid")
                .as_nanos()
        ));
        fs::write(&path, "[Settings]\ngtk-theme-name=Adwaita:dark\n")
            .expect("settings should be written");

        let settings = read_settings(&path).expect("settings should resolve");
        assert_eq!(settings.name, "Adwaita");
        assert!(settings.prefer_dark);
        fs::remove_file(path).expect("settings should be removed");
    }
}
