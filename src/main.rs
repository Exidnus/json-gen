use std::env;
use std::fs;
use rand::{Rng, SeedableRng};
use serde_json::{json, Map, Value as Json, Value};
use rand::distributions::{Alphanumeric, Standard};
use rand::rngs::StdRng;

fn main() {
    println!("Hello, world!");
    let args: Vec<String> = env::args().collect();
    let params = Params::parse(&args);

    if let Some(ref path_to_file) = params.schema_from_file {
        let schema = read_schema_from_file(&path_to_file);
        let generated = generate(schema, params.count);
        write_result(&params, &generated)
    } else {
        println!("No schema specified, do nothing.")
    }
}

fn write_result(params: &Params, result: &Vec<Json>) {
    match params.output_file.as_ref() {
        Some(path) => {
            println!("Printing content to specified file: {}", path);
            for json in result {
                let str = json.to_string();
                let path_ref: &str = path.as_ref();
                fs::write(path_ref, str);
            }
        },
        None => {
            for json in result {
                println!("{}", json)
            }
        }
    }
}

fn read_schema_from_file(path_to_file: &str) -> Json {
    let file = fs::File::open(path_to_file).expect("Error during file open");
    serde_json::from_reader(file).expect("Error during reading json schema from file")
}

//expect here that schema is valid
fn generate(schema: Json, count: u32) -> Vec<Json> {
    let mut result: Vec<Json> = Vec::new();
    let mut index = 0;
    while index < count {
        let json = generate_one_json(extract_properties_map(&schema));
        result.push(json);
        index += 1;
    }
    result
}

fn extract_properties_map(json: &Json) -> &Map<String, Value> {
    json["properties"]
        .as_object()
        .expect("properties should be object")
}

fn generate_one_json(schema_map: &Map<String, Value>) -> Json {
    let mut fields: Map<String, Json> = Map::new();
    for (k, v) in schema_map {
        let value_json = if let Json::String(t) = &v["type"] {
            match t.as_ref() {
                "integer" => generate_int(),
                "string" => generate_string(),
                "number" => generate_double(),
                "boolean" => generate_boolean(),
                "object" => generate_one_json(extract_properties_map(&v)),
                _ => panic!("unsupported type")
            }
        } else {
            panic!("No type")
        };
        fields.insert(k.to_string(), value_json);
    }
    Json::Object(fields)
}

fn generate_int() -> Json {
    let number: i32 = StdRng::from_entropy().sample(Standard);
    json!(number)
}

//TODO numbers are without fraction like 123213.0, 4545345.0 and so on
fn generate_double() -> Json {
    let number: f32 = StdRng::from_entropy().sample(Standard);
    let m: i32 = StdRng::from_entropy().sample(Standard);
    let result: f32 = number * m as f32;
    json!(result)
}

fn generate_string() -> Json {
    let string_value: String = rand::thread_rng()
        .sample_iter(Alphanumeric)
        .take(10)
        .map(char::from)
        .collect();
    json!(string_value)
}

fn generate_boolean() -> Json {
    let boolean: bool = StdRng::from_entropy().sample(Standard);
    json!(boolean)
}

struct Params {
    count: u32,
    schema_from_file: Option<String>,
    output_file: Option<String>
}

impl Params {
    fn parse(args: &Vec<String>) -> Params {
        let mut index = 0;
        let mut count: u32 = 100;
        let mut schema_from_file: Option<String> = None;
        let mut output_file: Option<String> = None;
        while index < args.len() - 1 {
            let current = &args[index];
            if current == "--count" {
                let count_str = &args[index + 1];
                count = count_str.parse::<u32>().expect("--count param should be int");
            } else if current == "--schema-from-file" {
                let path_to_file = &args[index + 1];
                schema_from_file = Some(path_to_file.to_string());
            } else if current == "--output-file" {
                let path_to_file = &args[index + 1];
                output_file = Some(path_to_file.to_string())
            }
            index += 2;
        }

        Params { count, schema_from_file, output_file }
    }
}