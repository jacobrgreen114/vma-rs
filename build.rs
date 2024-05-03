// Copyright (c) 2024 Jacob R. Green
// All rights reserved.

use bindgen;
use bindgen::callbacks::EnumVariantValue;
use std::collections::HashMap;
use std::env::var;
use std::fmt::Debug;
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

macro_rules! cargo_warning {
    ($($arg:tt)*) => {
        println!("cargo:warning={}", format!($($arg)*));
    };
}

type Enum = (String, EnumVariantValue);
type EnumVec = Vec<Enum>;

type EnumMap = HashMap<String, EnumVec>;

fn main() {
    let vulkan_sdk_path = PathBuf::from(var("VULKAN_SDK").expect("VULKAN_SDK not set"));
    let vulkan_include_dir: PathBuf = vulkan_sdk_path.join("Include");

    let vma_header_rel_path = PathBuf::from("vma").join("vk_mem_alloc.h");
    let vma_header_path = vulkan_include_dir.join(vma_header_rel_path);

    (!vma_header_path.exists()).then(|| {
        panic!("VMA header not found at {:?}", vma_header_path);
    });

    let enum_map = Arc::new(Mutex::new(EnumMap::new()));

    bindgen::builder()
        .parse_callbacks(Box::new(FormatCallback {
            enum_map: enum_map.clone(),
            cargo_callbacks: bindgen::CargoCallbacks::new(),
        }))
        .clang_args(&["-I", vulkan_include_dir.to_str().unwrap()])
        .header(vma_header_path.to_str().unwrap())
        .allowlist_recursively(false)
        .allowlist_file(".*vk_mem_alloc.*")
        .allowlist_type("Vma.*")
        .blocklist_file(".*")
        .blocklist_function(".*")
        .blocklist_var(".*")
        .blocklist_type(".*")
        .blocklist_item(".*")
        .prepend_enum_name(false)
        .generate()
        .unwrap();

    let out_path = PathBuf::from(var("OUT_DIR").unwrap());

    let prefix_map = build_config_map();

    {
        let enums_path = out_path.join("enums.rs");
        let mut enums_file = std::fs::File::create(&enums_path).unwrap();

        for (enum_name, enum_vec) in enum_map
            .lock()
            .unwrap()
            .iter()
            .map(|kv| (kv.0.as_str(), kv.1))
        {
            let config = match prefix_map.get(enum_name) {
                Some(config) => config,
                None => {
                    cargo_warning!("No prefix found for enum: {}", enum_name);
                    continue;
                }
            };

            if !config.is_flags {
                write_enum(&mut enums_file, config, enum_vec.iter(), None);
            } else {
                write_flags(&mut enums_file, config, enum_vec.iter());
            }
        }
    }
}

fn format_enum_name(name: &str) -> String {
    name.trim_start_matches("Vma").to_string()
}

fn format_enum_variant_name(prefix: &str, name: &str) -> Option<String> {
    let mut formatted = match name.strip_prefix(prefix) {
        Some(stripped) => stripped,
        None => {
            cargo_warning!("Failed to strip prefix from enum: {}", name);
            return None;
        }
    };
    if formatted.chars().next().unwrap().is_digit(10) {
        formatted = name.strip_prefix(&prefix[..prefix.len() - 1]).unwrap();
    }
    Some(formatted.to_string())
}

fn format_flag_enum_name(name: &str) -> String {
    name.trim_start_matches("Vma")
        .to_string()
        .replace("FlagBits", "Flags")
}

fn format_flag_variant_name(prefix: &str, name: &str) -> Option<String> {
    let mut formatted = match name.strip_prefix(prefix) {
        Some(stripped) => stripped,
        None => return None,
    };
    if formatted.chars().next().unwrap().is_digit(10) {
        formatted = name.strip_prefix(&prefix[..prefix.len() - 1]).unwrap();
    }
    let formatted = formatted.replace("_BIT", "");
    Some(formatted.to_string())
}

fn filter_enum_variant(name: &&str) -> bool {
    !name.contains("MAX_ENUM")
}

fn write_enum<'a, W: Write, I: Iterator<Item = &'a Enum>>(
    writer: &mut W,
    enum_config: &EnumConfig,
    variants: I,
    skip: Option<&[&str]>,
) {
    let new_name = enum_config
        .custom_name
        .map(|s| s.to_string())
        .unwrap_or_else(|| format_enum_name(enum_config.name));

    writeln!(writer, "#[derive(Debug, Clone, Copy, PartialEq, Eq)]").unwrap();
    writeln!(writer, "#[repr(i32)]").unwrap();
    writeln!(writer, "pub enum {} {{", new_name).unwrap();
    for variant in variants.map(|e| e.0.as_str()).filter(filter_enum_variant) {
        // cargo_warning!("{}: {}", enum_name, variant);
        if let Some(skip) = skip {
            if skip.contains(&variant) {
                continue;
            }
        }

        let formatted = match format_enum_variant_name(enum_config.prefix, variant) {
            Some(formatted) => formatted,
            None => continue,
        };

        writeln!(writer, "    {} = {},", formatted, variant).unwrap();
    }
    writeln!(writer, "}}").unwrap();
    writeln!(
        writer,
        "assert_eq_size!({}, {});",
        new_name, enum_config.name
    )
    .unwrap();

    writeln!(writer, "impl {} {{", new_name).unwrap();
    writeln!(
        writer,
        "    pub const fn from_raw(value: i32) -> Self {{ unsafe {{ std::mem::transmute(value) }} }}"
    )
    .unwrap();
    writeln!(
        writer,
        "    pub const fn as_raw(&self) -> i32 {{ *self as i32 }}"
    )
    .unwrap();
    writeln!(writer, "}}").unwrap();
    writeln!(writer).unwrap();
}

fn write_flags<'a, W: Write, I: Iterator<Item = &'a Enum>>(
    writer: &mut W,
    enum_config: &EnumConfig,
    variants: I,
) {
    let enum_name = enum_config
        .custom_name
        .map(|s| s.to_string())
        .unwrap_or_else(|| format_flag_enum_name(enum_config.name));

    writeln!(writer, "bitflags! {{").unwrap();
    writeln!(writer, "    #[derive(Default, Clone, Copy, PartialEq, Eq)]").unwrap();
    writeln!(writer, "    pub struct {}: u32 {{", enum_name).unwrap();

    for variant in variants.map(|e| e.0.as_str()).filter(filter_enum_variant) {
        // cargo_warning!("{}: {}", enum_name, variant);
        if variant.contains("MAX_ENUM") {
            continue;
        }

        let formatted = match format_flag_variant_name(enum_config.prefix, variant) {
            Some(formatted) => formatted,
            None => continue,
        };

        writeln!(writer, "        const {} = {} as u32;", formatted, variant).unwrap();
    }

    writeln!(writer, "    }}").unwrap();
    writeln!(writer, "}}").unwrap();

    // writeln!(writer, "impl {} {{", enum_name).unwrap();
    // writeln!(
    //     writer,
    //     "    pub const fn from_raw(value: u32) -> Self {{ Self::from_bits_truncate(value) }}"
    // )
    writeln!(
        writer,
        "assert_eq_size!({}, {});",
        enum_name, enum_config.name
    )
    .unwrap();
    writeln!(writer).unwrap();
}

#[derive(Debug)]
struct FormatCallback {
    enum_map: Arc<Mutex<EnumMap>>,
    cargo_callbacks: bindgen::CargoCallbacks,
}

fn push_enum_variant(vec: &mut EnumVec, variant: &str, value: EnumVariantValue) {
    if vec.iter().map(|e| e.1).any(|e| e == value) {
        // cargo_warning!("Duplicate value found for variant: {}", variant);
        return;
    }

    vec.push((variant.to_string(), value));
}

impl bindgen::callbacks::ParseCallbacks for FormatCallback {
    fn enum_variant_name(
        &self,
        enum_name: Option<&str>,
        original_variant_name: &str,
        variant_value: EnumVariantValue,
    ) -> Option<String> {
        // macro_rules! push_to_vec {
        let trimmed_enum_name = match &enum_name {
            Some(name) => name.trim_start_matches("enum "),
            None => {
                cargo_warning!("No enum name found for variant: {}", original_variant_name);
                return None;
            }
        };

        if !trimmed_enum_name.starts_with("Vma") {
            return None;
        }

        let mut map = self.enum_map.lock().unwrap();
        let vec = map
            .entry(trimmed_enum_name.to_string())
            .or_insert_with(Vec::new);
        push_enum_variant(vec, original_variant_name, variant_value);

        None
    }

    fn header_file(&self, filename: &str) {
        self.cargo_callbacks.header_file(filename);
    }

    fn include_file(&self, filename: &str) {
        self.cargo_callbacks.include_file(filename);
    }
}

#[derive(Debug, Clone)]
struct EnumConfig<'a> {
    name: &'a str,
    custom_name: Option<&'a str>,
    prefix: &'a str,
    is_flags: bool,
}

impl Default for EnumConfig<'_> {
    fn default() -> Self {
        Self {
            name: "",
            custom_name: None,
            prefix: "",
            is_flags: false,
        }
    }
}

fn build_config_map() -> HashMap<&'static str, EnumConfig<'static>> {
    let configs: &[EnumConfig<'static>] = &[
        EnumConfig {
            name: "VmaMemoryUsage",
            prefix: "VMA_MEMORY_USAGE_",
            ..Default::default()
        },
        EnumConfig {
            name: "VmaAllocationCreateFlagBits",
            prefix: "VMA_ALLOCATION_CREATE_",
            is_flags: true,
            ..Default::default()
        },
        EnumConfig {
            name: "VmaPoolCreateFlagBits",
            prefix: "VMA_POOL_CREATE_",
            is_flags: true,
            ..Default::default()
        },
        EnumConfig {
            name: "VmaAllocatorCreateFlagBits",
            prefix: "VMA_ALLOCATOR_CREATE_",
            is_flags: true,
            ..Default::default()
        },
        EnumConfig {
            name: "VmaDefragmentationFlagBits",
            prefix: "VMA_DEFRAGMENTATION_FLAG_",
            is_flags: true,
            ..Default::default()
        },
        EnumConfig {
            name: "VmaDefragmentationMoveOperation",
            prefix: "VMA_DEFRAGMENTATION_MOVE_OPERATION_",
            ..Default::default()
        },
        EnumConfig {
            name: "VmaVirtualBlockCreateFlagBits",
            prefix: "VMA_VIRTUAL_BLOCK_CREATE_",
            is_flags: true,
            ..Default::default()
        },
        EnumConfig {
            name: "VmaVirtualAllocationCreateFlagBits",
            prefix: "VMA_VIRTUAL_ALLOCATION_CREATE_",
            is_flags: true,
            ..Default::default()
        },
    ];

    let mut map = HashMap::new();
    for config in configs {
        map.insert(config.name, config.clone()).inspect(|c| {
            cargo_warning!("Duplicate enum config found: {}", c.name);
        });
    }
    map
}
