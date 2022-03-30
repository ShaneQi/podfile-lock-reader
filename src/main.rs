extern crate yaml_rust;
use clap::{arg, command, Command};
use linked_hash_map::LinkedHashMap;
use std::collections::HashSet;
use std::env;
use std::fs;
use std::vec::Vec;
use yaml_rust::Yaml;
use yaml_rust::YamlLoader;

fn main() {
    let matches = command!()
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("dependencies")
                .about("Get a list of denepdencies of each module.")
                .arg(arg!(-r --recursive "Whether to count indirect dependencies recursively"))
                .arg(arg!(-c --confluence "Whether to output in Confluence-friendly format"))
                .arg(
                    arg!(-p --podfilelock <PODFILE_LOCK> "The path of the Podfile.lock.")
                        .required(false)
                        .default_value("Podfile.lock"),
                ),
        )
        .get_matches();

    match matches.subcommand() {
        Some(("dependencies", sub_matches)) => {
            let podfile_lock_path = sub_matches.value_of("podfilelock").unwrap();
            let contents = fs::read_to_string(podfile_lock_path)
                .expect("Something went wrong reading the file");
            let docs = YamlLoader::load_from_str(&contents).unwrap();
            let dependencies = &docs[0]
                .as_hash()
                .unwrap()
                .get(&Yaml::String("DEPENDENCIES".to_string()))
                .unwrap()
                .as_vec()
                .unwrap();
            let modules = filter_modules_from(dependencies);
            let pods = &docs[0]
                .as_hash()
                .unwrap()
                .get(&Yaml::String("PODS".to_string()))
                .unwrap()
                .as_vec()
                .unwrap();
            let map = direct_dependency_map_from(pods, &modules);
            let recursive = sub_matches.is_present("recursive");
            let confluence = sub_matches.is_present("confluence");
            if confluence {
                println!(
                    r#"
<table class="wrapped">
    <colgroup> <col/> <col/> </colgroup>
    <tbody>
        <tr>
            <th colspan="1">Module</th>
            <th colspan="1">Depends on these modules ({})</th>
            <th colspan="1">Count</th>
        </tr>"#, if recursive { "including indirect" } else { "direct only" }
                );
                for key in map.keys() {
                    let dependencies = if recursive {
                        recursively_find_dependencies(&map, key.to_owned())
                    } else {
                        map[key].to_owned()
                    };
                    let dependencies_string: Vec<String> = dependencies.iter().map(|d|{ format!("* {}", d)}).collect();
                    println!(
                        r#"
            <tr>
                <td colspan="1">{}</td>
                <td colspan="1">{}</td>
                <td colspan="1">{}</td>
            </tr>"#,
                        key,
                        dependencies_string.join("<br/>"),
                        dependencies.len()
                    );
                }
                println!(
                    r#"
    </tbody>
</table>"#
                )
            } else {
                for key in map.keys() {
                    let dependencies = if recursive {
                        recursively_find_dependencies(&map, key.to_owned())
                    } else {
                        map[key].to_owned()
                    };
                    print!("* {}", key);
                    println!(" ({})", dependencies.len());
                    for dep in dependencies {
                        println!("  * {}", dep);
                    }
                    println!();
                }
            }
        }
        _ => unreachable!("Exhausted list of subcommands and subcommand_required prevents `None`"),
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

    // let mut count = LinkedHashMap::<String, u8>::new();
    // for key in root_keys {
    //     let deps = rec2(&map, key);
    //     for dep in deps {
    //         let counter = count.entry(dep).or_insert(0);
    //         *counter += 1;
    //     }
    // }
    // let mut entries: Vec<_> = count.entries().collect();
    // entries.sort_by(|a, b| b.get().cmp(a.get()));
    // for entry in entries {
    //     println!("{} ({})", entry.key(), entry.get());
    // }
}

fn filter_modules_from(dependencies: &&std::vec::Vec<yaml_rust::Yaml>) -> Vec<String> {
    let mut modules = vec![];
    for dependency in dependencies.iter() {
        let string = dependency.as_str().unwrap();
        if string.contains("from") {
            let end_bytes = string.find(" (from").unwrap_or(string.len());
            let module = &string[0..end_bytes];
            modules.push(module.to_owned());
        }
    }
    modules
}

fn direct_dependency_map_from(
    pods: &&std::vec::Vec<yaml_rust::Yaml>,
    modules: &std::vec::Vec<std::string::String>,
) -> LinkedHashMap<String, Vec<String>> {
    let mut map = LinkedHashMap::<String, Vec<String>>::new();
    for pod_hash in pods.iter() {
        match pod_hash {
            Yaml::Hash(hash) => {
                let mut keys = hash.keys();
                let key = keys.next().unwrap();
                let key_string = key.as_str().unwrap();
                let name = key_string.split(" ").next().unwrap();
                if modules.contains(&name.to_owned()) {
                    let mut values: Vec<String> = vec![];
                    hash[key]
                        .as_vec()
                        .unwrap()
                        .iter()
                        .for_each(|yaml| match yaml {
                            Yaml::String(value) => {
                                let dependency = value.split(" ").next().unwrap();
                                if modules.contains(&dependency.to_owned()) {
                                    values.push(dependency.to_owned());
                                }
                            }
                            _ => {}
                        });
                    if map.contains_key(name) {
                        panic!();
                    }
                    map.insert(name.to_owned(), values);
                }
            }
            Yaml::String(string) => {
                let name = string.split(" ").next().unwrap();
                if modules.contains(&name.to_owned()) {
                    if map.contains_key(name) {
                        panic!();
                    }
                    map.insert(name.to_owned(), vec![]);
                }
            }
            _ => {}
        }
    }
    map
}

// fn rec(map: &LinkedHashMap<String, Vec<String>>, depth: u8, keys: Vec<String>) {
//     for key in keys {
//         for _ in 0..depth {
//             print!("  ");
//         }
//         print!("* {}", key);
//         println!();
//         let mut next_keys: Vec<String> = vec![];
//         for next_key in &map[&key] {
//             next_keys.push(next_key.as_str().to_string());
//         }
//         rec(map, depth + 1, next_keys);
//     }
// }

fn recursively_find_dependencies(
    map: &LinkedHashMap<String, Vec<String>>,
    key: String,
) -> Vec<String> {
    let mut set: HashSet<String> = HashSet::new();
    for next_key in &map[&key] {
        set.insert(next_key.clone());
        for deep_name in recursively_find_dependencies(map, next_key.clone()) {
            set.insert(deep_name);
        }
    }
    set.into_iter().collect()
}
