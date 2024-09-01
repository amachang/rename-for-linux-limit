use std::{path::PathBuf, fs, io, collections::{HashSet, HashMap}};
use dirs::config_dir;
use clap::{Parser, crate_name};
use anyhow::Result;
use serde::{Serialize, Deserialize};
use unicode_normalization::UnicodeNormalization;

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    ignored_tags: HashSet<String>,
    conversions: HashMap<String, String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            ignored_tags: HashSet::new(),
            conversions: HashMap::new(),
        }
    }
}


#[derive(Parser, Debug)]
struct Args {
    #[clap(short = 's', long, default_value = "false")]
    only_show_new_filename: bool,
    path: PathBuf,
}

const N_FILENAME_BYTES: usize = 255;
const N_MAX_EXTENSION_BYTES: usize = 5;

#[derive(thiserror::Error, Debug)]
enum Error {
    #[error("Rename error: {0} -> {1}: {2}")]
    RenameError(PathBuf, PathBuf, io::Error),
    #[error("Config directory not found")]
    ConfigDirNotFound,
    #[error("Filename not found in path: {0}")]
    FilenameNotFound(PathBuf),
}

fn main() -> Result<()> {
    env_logger::init();
    let args = Args::parse();
    let config = prepare_config()?;

    // NFC normalization
    let ignored_tags = config.ignored_tags.iter().map(|s| normalize_str(s)).collect();
    let tag_conversion_map = config.conversions.iter().map(|(k, v)| {
        (normalize_str(k), normalize_str(v))
    }).collect();

    let path = args.path;
    let filename = match path.file_name() {
        Some(filename) => filename,
        None => return Err(Error::FilenameNotFound(path).into()),
    };

    if filename.as_encoded_bytes().len() <= N_FILENAME_BYTES {
        let filename = filename.to_string_lossy().to_string();
        if args.only_show_new_filename {
            println!("{}", filename);
        } else {
            log::info!("Filename is already short enough: {}", filename);
        }
    } else {
        let filename = filename.to_string_lossy();
        let mut n_retries = 0;
        loop {
            let new_filename = new_filename(&filename, &ignored_tags, &tag_conversion_map, n_retries);
            if args.only_show_new_filename {
                println!("{}", new_filename);
                break;
            } else {
                log::trace!("New filename: {}", new_filename);
                let new_path = path.with_file_name(&new_filename);
                if new_path.exists() {
                    n_retries += 1;
                    continue;
                } else {
                    match fs::rename(&path, &new_path) {
                        Ok(_) => log::info!("Renamed: {} -> {}", filename, new_filename),
                        Err(e) => return Err(Error::RenameError(path, new_path, e).into()),
                    };
                    break;
                }
            }
        }
    }
    Ok(())
}

fn new_filename(filename: impl AsRef<str>, ignored_tags: &HashSet<String>, tag_conversion_map: &HashMap<String, String>, n_retries: usize) -> String {
    let filename = filename.as_ref();
    assert!(!filename.is_empty());

    let mut split = filename.rsplitn(2, '.');
    let ext = split.next().expect("first element is not empty");
    let slug = split.next();
    assert!(split.next().is_none());

    let (ext, slug) = if let Some(slug) = slug {
        if slug.is_empty() {
            // in case filename starts with dot
            (None, format!(".{}", ext))
        } else {
            (Some(ext), slug.to_string())
        }
    } else {
        // in case no dot in filename
        (None, ext.to_string())
    };

    let (ext, slug) = if let Some(ext) = ext {
        if ext.len() > N_MAX_EXTENSION_BYTES {
            (None, format!("{}.{}", slug, ext))
        } else {
            (Some(ext), slug)
        }
    } else {
        (None, slug)
    };

    let ext = if let Some(ext) = ext {
        if n_retries == 0 {
            Some(ext.to_string())
        } else {
            Some(format!("{}.{}", n_retries, ext))
        }
    } else {
        if n_retries == 0 {
            None
        } else {
            Some(format!("{}", n_retries))
        }
    };

    let (mut n_remaining_slug_bytes, slug, ext) = if let Some(ext) = &ext {
        let ext_len = ext.len() + 1;
        assert!(ext_len <= usize::MAX.to_string().as_bytes().len() + N_MAX_EXTENSION_BYTES + 2);
        assert!(ext_len <= N_FILENAME_BYTES);
        let n_remaining_slug_bytes = N_FILENAME_BYTES.checked_sub(ext_len).expect("checked");
        (n_remaining_slug_bytes, slug, format!(".{}", ext))
    } else {
        (N_FILENAME_BYTES, filename.to_string(), "".to_string())
    };

    log::trace!("Remaining slug bytes (subtract extention): {}", n_remaining_slug_bytes);

    let (first_component, remaining_components) = split_into_components(&slug, tag_conversion_map);

    let mut new_slug = String::new();
    if first_component.as_bytes().len() > n_remaining_slug_bytes {
        for char in first_component.chars() {
            if n_remaining_slug_bytes < char.len_utf8() {
                break;
            }
            n_remaining_slug_bytes -= char.len_utf8();
            new_slug.push(char);
        }
    } else {
        n_remaining_slug_bytes -= first_component.as_bytes().len();
        new_slug.push_str(first_component);

        // (len, index)
        let mut len_indecies = remaining_components.iter().enumerate().map(|(i, c)| {
            let len = c.n_bytes();
            (len, i)
        }).collect::<Vec<_>>();

        // shorter components prefered
        len_indecies.sort_by(|(len1, _), (len2, _)| len1.cmp(len2));

        let mut seen_tags = HashSet::new();
        let mut converted_components = vec![String::new(); remaining_components.len()];
        for (len, i) in len_indecies {
            let component = &remaining_components[i];
            let delimiter = component.delimiter;
            let raw_tag = &component.tag;
            let normalized_tag = normalize_str(raw_tag);
            if ignored_tags.contains(&normalized_tag) {
                continue;
            }
            if seen_tags.contains(&normalized_tag) {
                continue;
            }
            if n_remaining_slug_bytes == 0 {
                break;
            }
            if n_remaining_slug_bytes < len {
                let mut new_component = String::new();
                if n_remaining_slug_bytes < delimiter.len_utf8() {
                    break;
                }
                n_remaining_slug_bytes -= delimiter.len_utf8();
                new_component.push(delimiter);

                for char in raw_tag.chars() {
                    if n_remaining_slug_bytes < char.len_utf8() {
                        break;
                    }
                    n_remaining_slug_bytes -= char.len_utf8();
                    new_component.push(char);
                }

                converted_components[i] = new_component;
                break;
            }
            n_remaining_slug_bytes -= len;
            converted_components[i] = delimiter.to_string() + &raw_tag;
            seen_tags.insert(normalized_tag);
        }

        for component in converted_components {
            new_slug.push_str(&component);
            log::trace!("New slug pushed ({1}) {0}", new_slug, new_slug.as_bytes().len());
        }
    }

    let new_filename = format!("{}{}", new_slug, ext);
    log::trace!("New filename: ({1}) {0}", new_filename, new_filename.as_bytes().len());
    assert!(new_filename.as_bytes().len() <= N_FILENAME_BYTES);
    return new_filename;
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SlugComponent {
    delimiter: char,
    tag: String,
}

impl SlugComponent {
    fn n_bytes(&self) -> usize {
        self.tag.as_bytes().len() + self.delimiter.len_utf8()
    }
}

impl std::fmt::Display for SlugComponent {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}{}", self.delimiter, self.tag)
    }
}

const DELIMITERS: [char; 1] = ['.'];

fn split_into_components<'a>(slug: &'a str, tag_conversion_map: &HashMap<String, String>) -> (&'a str, Vec<SlugComponent>) {
    assert!(!slug.is_empty());
    let mut components = Vec::new();

    // first character is not delimiter
    let mut char_indices = slug.char_indices();
    let mut start;

    let first_component = loop {
        if let Some((i, c)) = char_indices.next() {
            if 0 < i && DELIMITERS.contains(&c) {
                start = i;
                break &slug[..i];
            }
        } else {
            start = slug.len();
            break slug;
        }
    };

    while let Some((i, c)) = char_indices.next() {
        if DELIMITERS.contains(&c) {
            let tag = &slug[start + c.len_utf8() .. i];
            components.push(SlugComponent { delimiter: c, tag: tag.to_string() });
            start = i;
        }
    }
    if start < slug.len() {
        let c = slug[start..].chars().next().expect("checked");
        let tag = &slug[start + c.len_utf8()..];
        components.push(SlugComponent { delimiter: c, tag: tag.to_string() });
    }

    let components = components.into_iter().map(|c| {
        let delimiter = c.delimiter;
        let tag = tag_conversion_map.get(&normalize_str(&c.tag)).unwrap_or(&c.tag);
        SlugComponent { delimiter, tag: tag.to_string() }
    }).collect();

    (first_component, components)
}

fn normalize_str(s: impl AsRef<str>) -> String {
    // NFD normalization for interportability
    s.as_ref().nfd().collect()
}

fn prepare_config() -> Result<Config> {
    let config_parent_dir = config_dir().ok_or(Error::ConfigDirNotFound)?;
    let config_dir = config_parent_dir.join(crate_name!());
    fs::create_dir_all(&config_dir)?;

    let config_path = config_dir.join("config.toml");
    if !config_path.exists() {
        let default_config = Config::default();
        let toml = toml::to_string_pretty(&default_config)?;
        std::fs::write(&config_path, toml)?;
        log::debug!("Default config written to {:?}", config_path);
    }
    let config = config::Config::builder()
        .add_source(config::File::from(config_path))
        .build()?;
    let config = config.try_deserialize::<Config>()?;

    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_into_components() {
        let slug = "a.b.c..d";
        let components = split_into_components(slug, &HashMap::new());
        assert_eq!(components, ("a", vec![
            SlugComponent { delimiter: '.', tag: "b".to_string() },
            SlugComponent { delimiter: '.', tag: "c".to_string() },
            SlugComponent { delimiter: '.', tag: "".to_string() },
            SlugComponent { delimiter: '.', tag: "d".to_string() },
        ]));

        let slug = ".あああ.いいい.ううう";
        let components = split_into_components(slug, &HashMap::new());
        assert_eq!(components, (".あああ", vec![
            SlugComponent { delimiter: '.', tag: "いいい".to_string() },
            SlugComponent { delimiter: '.', tag: "ううう".to_string() },
        ]));
    }

    #[test]
    fn test_new_filename() {
        let ignored_tags = HashSet::new();
        let tag_conversion_map = HashMap::new();
        assert_eq!(new_filename("a.b.c..d", &ignored_tags, &tag_conversion_map, 0), "a.b.c..d");
        assert_eq!(new_filename("a.b.c..d", &ignored_tags, &tag_conversion_map, 1), "a.b.c..1.d");
        assert_eq!(new_filename("一二三四五六七八九十一二三四五六七八九十一二三四五六七八九十一二三四五六七八九十一二三四五六七八九十一二三四五六七八九十一二三四五六七八九十一二三四五六七八九十一二三四五", &ignored_tags, &tag_conversion_map, 0), "一二三四五六七八九十一二三四五六七八九十一二三四五六七八九十一二三四五六七八九十一二三四五六七八九十一二三四五六七八九十一二三四五六七八九十一二三四五六七八九十一二三四五");
        assert_eq!(new_filename(".一二三四五六七八九十一二三四五六七八九十一二三四五六七八九十一二三四五六七八九十一二三四五六七八九十一二三四五六七八九十一二三四五六七八九十一二三四五六七八九十一二三四五", &ignored_tags, &tag_conversion_map, 0), ".一二三四五六七八九十一二三四五六七八九十一二三四五六七八九十一二三四五六七八九十一二三四五六七八九十一二三四五六七八九十一二三四五六七八九十一二三四五六七八九十一二三四");
        assert_eq!(new_filename("一二三四五六七八九十一二三四五六七八九十一二三四五六七八九十一二三四五六七八九十一二三四五六七八九十一二三四五六七八九十一二三四五六七八九十一二三四五六七八九十一二三四五六七八九十", &ignored_tags, &tag_conversion_map, 0), "一二三四五六七八九十一二三四五六七八九十一二三四五六七八九十一二三四五六七八九十一二三四五六七八九十一二三四五六七八九十一二三四五六七八九十一二三四五六七八九十一二三四五");
        assert_eq!(new_filename("一二三四五六七八九十一二三四五六七八九十一二三四五六七八九十一二三四五六七八九十一二三四五六七八九十一二三四五六七八九十一二三四五六七八九十一二三四五六七八九十一二三四五", &ignored_tags, &tag_conversion_map, 1), "一二三四五六七八九十一二三四五六七八九十一二三四五六七八九十一二三四五六七八九十一二三四五六七八九十一二三四五六七八九十一二三四五六七八九十一二三四五六七八九十一二三四.1");
        assert_eq!(new_filename(".一二三四五六七八九十一二三四五六七八九十一二三四五六七八九十一二三四五六七八九十一二三四五六七八九十一二三四五六七八九十一二三四五六七八九十一二三四五六七八九十一二三四五", &ignored_tags, &tag_conversion_map, 11), ".一二三四五六七八九十一二三四五六七八九十一二三四五六七八九十一二三四五六七八九十一二三四五六七八九十一二三四五六七八九十一二三四五六七八九十一二三四五六七八九十一二三.11");
    }
}

