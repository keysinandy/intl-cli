use crate::intl::extract::extract_text;
use globset::{Glob, GlobSet, GlobSetBuilder};
use serde_json::Value;
use serde_json::{from_reader, to_writer_pretty, Map};
use std::borrow::Borrow;
use std::collections::HashMap;
use std::fs::{self, File};
use std::path::Path;
use std::{env, io};

use super::extract::IntlInfo;

fn visit_dirs(
    dir: &Path,
    includes: &GlobSet,
    excludes: &GlobSet,
    existed_map: &Map<String, Value>,
    intl_map: &mut IntlInfo,
) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(&path, includes, excludes, existed_map, intl_map)?;
            } else {
                if includes.is_match(entry.path()) && !excludes.is_match(entry.path()) {
                    extract_text(entry.path().to_str().unwrap(), existed_map, intl_map);
                }
            }
        }
    }
    Ok(())
}

pub fn run_extract(
    output: Option<String>,
    excludes: Option<Vec<String>>,
    includes: Option<Vec<String>>,
    delete_unreached: bool,
) -> () {
    let mut includes_builder = GlobSetBuilder::new();
    let mut excludes_builder = GlobSetBuilder::new();

    if let Some(includes) = includes {
        for includes_path in includes {
            includes_builder.add(
                Glob::new(includes_path.as_str())
                    .expect("Failed to read includes glob pattern, please check includes path"),
            );
        }
    }
    if let Some(excludes) = excludes {
        for excludes_path in excludes {
            excludes_builder.add(
                Glob::new(excludes_path.as_str())
                    .expect("Failed to read excludes glob pattern, please check excludes path"),
            );
        }
    }
    let includes_set = includes_builder
        .build()
        .expect("Failed to build includes glob set");
    let excludes_set = excludes_builder
        .build()
        .expect("Failed to build excludes glob set");
    let current_dir = env::current_dir().expect("Failed to get current directory");
    let mut existed_map: Map<String, Value> = Map::new();

    if let Some(s) = output.borrow() {
        let output = Path::new(current_dir.to_str().unwrap()).join(s);
        if let Ok(file) = File::open(output) {
            if let Ok(json_str) = from_reader::<File, Value>(file) {
                let json_obj = json_str.as_object();
                if let Some(obj) = json_obj {
                    existed_map = obj.to_owned();
                }
            }
        }
    }
    let mut intl_map = IntlInfo {
        info_map: HashMap::new(),
        err_map: HashMap::new(),
        repeat_key_list: Vec::new(),
    };

    visit_dirs(
        &current_dir,
        &includes_set,
        &excludes_set,
        &existed_map,
        &mut intl_map,
    )
    .expect("Failed to visit directory");

    println!("{}", intl_map);
    if delete_unreached {
        existed_map.clear();
        for (key, default) in intl_map.repeat_key_list {
            existed_map.insert(key, default);
        }
    }

    for (_, value) in intl_map.info_map.iter() {
        existed_map.insert(
            value.key.to_string(),
            Value::String(value.default.to_string()),
        );
    }
    if let Some(s) = output {
        let output = Path::new(current_dir.to_str().unwrap()).join(s);
        if let Ok(file) = File::create(output) {
            to_writer_pretty(file, &existed_map).unwrap();
        }
    }
}
