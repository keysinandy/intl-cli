use serde_json::{from_reader, to_writer_pretty, Map, Value};
use std::{env, fs::File, path::Path};

use super::tencent::generate_by_tencent;

pub trait Payload {
    fn to_string(&self, pair_list: &Vec<(String, Value)>) -> String;
    fn to_map(&self, pair_list: &Vec<(String, Value)>, list: Vec<String>) -> Map<String, Value>;
}

pub struct Translate<T: Payload> {
    input: String,
    output: String,
    pub pair_list: Vec<(String, Value)>,
    pub payload: T,
}

impl<T: Payload> Translate<T> {
    pub fn new(input: String, output: String, payload: T) -> Translate<T> {
        Translate {
            input,
            output,
            payload,
            pair_list: vec![],
        }
    }
    fn get_pair_list(&self, excludes: &Map<String, Value>) -> Vec<(String, Value)> {
        let input_file = File::open(env::current_dir().unwrap().join(self.input.to_string()));
        let mut list = Vec::new();
        if let Ok(file) = input_file {
            if let Ok(json_str) = from_reader::<File, Value>(file) {
                let json_obj = json_str.as_object();
                if let Some(obj) = json_obj {
                    obj.iter().for_each(|x| {
                        if !excludes.contains_key(x.0) {
                            list.push((x.0.to_owned(), x.1.to_owned()));
                        }
                    });
                }
            }
        }
        return list;
    }
    pub fn from_tencent(
        &mut self,
        secret_id: &str,
        secret_key: &str,
        write_all: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let output_path = Path::new(&self.output);
        let mut obj: Map<String, Value> = Map::new();
        match File::open(output_path) {
            Ok(file) => {
                let obj_str = from_reader::<File, Value>(file).unwrap_or(Value::Object(Map::new()));
                obj = obj_str.as_object().unwrap().to_owned();
            }
            Err(_) => {}
        }
        let mut excludes = &Map::new();
        if !write_all {
            excludes = &obj;
        }
        self.pair_list = self.get_pair_list(excludes);
        if self.pair_list.len() == 0 {
            println!("=========== Nothing needs to translate ===========",);
            return Ok(());
        }
        let result = generate_by_tencent(&self, &self.pair_list, secret_id, secret_key)?;
        let mut json = self
            .payload
            .to_map(&self.pair_list, result.Response.TargetTextList);
        obj.append(&mut json);
        to_writer_pretty(File::create(output_path)?, &obj).unwrap();
        return Ok(());
    }
}
