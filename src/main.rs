extern crate yaml_rust;
use linked_hash_map::LinkedHashMap;
use podfile_lock_reader::modules;
use std::collections::HashSet;
use std::env;
use std::fs;
use std::vec::Vec;
use yaml_rust::Yaml;
use yaml_rust::YamlLoader;

fn main() {
    let modules = modules();
    let args: Vec<String> = env::args().collect();
    let filename = &args[1];
    let contents = fs::read_to_string(filename).expect("Something went wrong reading the file");
    let docs = YamlLoader::load_from_str(&contents).unwrap();
    let pods = &docs[0]
        .as_hash()
        .unwrap()
        .get(&Yaml::String("PODS".to_string()))
        .unwrap()
        .as_vec()
        .unwrap();

    let mut map = LinkedHashMap::<String, Vec<String>>::new();
    let mut root_keys: Vec<String> = vec![];

    for pod_hash in pods.iter() {
        match pod_hash {
            Yaml::Hash(hash) => {
                let mut keys = hash.keys();
                let key = keys.next().unwrap();
                let key_string = key.as_str().unwrap();
                let name = key_string.split(" ").next().unwrap();
                if modules.contains(&name) {
                    let mut values: Vec<String> = vec![];
                    hash[key]
                        .as_vec()
                        .unwrap()
                        .iter()
                        .for_each(|yaml| match yaml {
                            Yaml::String(value) => {
                                let pod_name = value.split(" ").next().unwrap();
                                if modules.contains(&pod_name) {
                                    values.push(pod_name.to_string());
                                }
                            }
                            _ => {}
                        });
                    if map.contains_key(name) {
                        panic!();
                    }
                    map.insert(name.to_string(), values);
                    root_keys.push(name.to_string());
                }
            }
            Yaml::String(string) => {
                let name = string.split(" ").next().unwrap();
                if modules.contains(&name) {
                    if map.contains_key(name) {
                        panic!();
                    }
                    map.insert(name.to_string(), vec![]);
                    root_keys.push(name.to_string());
                }
            }
            _ => {}
        }
    }
    // rec(&map, 0, root_keys);

    // for key in root_keys {
    //     print!("* {}", key);
    //     let deps = rec2(&map, key);
    //     print!(" ({})", deps.len());
    //     println!();
    //     for dep in deps {
    //         println!("  * {}", dep);
    //     }
    // }

    let mut count = LinkedHashMap::<String, u8>::new();
    for key in root_keys {
        let deps = rec2(&map, key);
        for dep in deps {
            let counter = count.entry(dep).or_insert(0);
            *counter += 1;
        }
    }
    let mut entries: Vec<_> = count.entries().collect();
    entries.sort_by(|a, b| b.get().cmp(a.get()) );
    for entry in entries {
        println!("{} ({})", entry.key(), entry.get());
    }
}

fn rec(map: &LinkedHashMap<String, Vec<String>>, depth: u8, keys: Vec<String>) {
    for key in keys {
        for _ in 0..depth {
            print!("  ");
        }
        print!("* {}", key);
        println!();
        let mut next_keys: Vec<String> = vec![];
        for next_key in &map[&key] {
            next_keys.push(next_key.as_str().to_string());
        }
        rec(map, depth + 1, next_keys);
    }
}

fn rec2(map: &LinkedHashMap<String, Vec<String>>, key: String) -> HashSet<String> {
    let mut set: HashSet<String> = HashSet::new();
    for next_key in &map[&key] {
        set.insert(next_key.clone());
        for deep_name in rec2(map, next_key.clone()) {
            set.insert(deep_name);
        }
    }
    set
}
