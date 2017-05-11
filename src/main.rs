extern crate orm;
extern crate glob;
use glob::glob;

use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::error::Error;
use std::collections::HashMap;

mod entity;
use entity::*;
use orm::Entity;
use orm::Insert;
use orm::Select;
use orm::Db;
use orm::JoinCond;
use orm::Cond;

trait ReadContent {
    fn read_content(&mut self) -> Vec<u8>;
}

impl ReadContent for File {
    fn read_content(&mut self) -> Vec<u8> {
        let mut ret = Vec::new();
        self.read_to_end(&mut ret);
        ret
    }
}

fn parse_file(path: PathBuf) -> Vec<Vec<String>> {
    let res = File::open(path.clone());
    let list_path = path.to_str().unwrap();
    let mut file = res.unwrap();
    let content = file.read_content();
    let arr = content.as_slice();

    let res = (0..arr.len()).fold(Vec::new(), |mut acc, i| {
        if i + 8 >= arr.len() {
            return acc;
        }
        if &arr[i..i + 8] != ".unity3d".as_bytes() {
            return acc;
        }
        let check = |j: usize| {
            match arr[j - 7] {
                0x0a | 0x00 => {}
                _ => {
                    // println!("Invalid Check 1 Start: 0x{:x}, U3d: 0x{:x}", j - 7, i);
                    return false;
                }
            };
            let check_no = (j - 6..j).all(|j| ('0' as u8) <= arr[j] && arr[j] <= ('9' as u8));
            if !check_no {
                println!("Invalid Check 2 0x{:x}", j - 7);
            }
            return check_no;
        };
        // 找到u3d的位置了，向前找编号和类型，这是第五列，前面四个09，在前面是0a或者00
        let mut last = 4;
        let mut vec = Vec::new();
        let mut start = (0..i)
            .rev()
            .find(|&j: &usize| {
                if arr[j] == 0 {
                    last = -1;
                    return true;
                }
                if arr[j] == 0x09 {
                    last = last - 1;
                    vec.push(format!("0x{:x}", j));
                }
                if last == 0 && check(j) {
                    // println!("Check Succ: 0x{:x}", j);
                    return true;
                }
                if last == 0 {
                    // println!("Last == 0 But Check Fail, Start: 0x{:x}, U3d: 0x{:x}",
                    //          j - 7,
                    //          i);
                    // println!("{:?}", vec);
                    last = -1;
                    return true;
                }
                return false;
            })
            .unwrap();
        // 没有找到
        if last == -1 {
            return acc;
        }
        start = start - 6;
        // acc.push(start);
        // 从start开始向后找5个09
        let mut vec = vec![Vec::new()];
        (start..).find(|&j|{
            if arr[j] == 0x09 && vec.len() < 5{
                vec.push(Vec::new());
                return false;
            }else if arr[j] == 0x09 && vec.len() == 5{
                return true;
            }else{
                vec.last_mut().unwrap().push(arr[j]);
                return false;
            }
        }).unwrap();
        let mut vec = vec.into_iter().map(|s| String::from_utf8(s).unwrap()).collect::<Vec<_>>();
        if &vec[3] != "abdata"{
            // println!("{:?}", vec);
            // unreachable!();
        }
        vec.push(list_path.to_string());
        acc.push(vec);
        return acc;
    });
    return res;
}

fn insert_list(base: &str, sys: &str, db: &Db) {
    let dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let pattern = format!("{}/characustom/*.unity3d", base);
    // let path = Path::new(&dir).join("list/characustom/*.unity3d");
    let path = Path::new(&dir).join(pattern);
    let res = glob(path.to_str().unwrap())
        .unwrap()
        .into_iter()
        .flat_map(|entry| {
            return parse_file(entry.unwrap());
        })
        .collect::<Vec<_>>();


    // let session = db.open_session();
    for item in res.iter() {
        let mut m = Mod::default();
        m.set_no(&item[0]);
        m.set_kind(&item[1]);
        m.set_name(&item[2]);
        m.set_dir(&item[3]);
        m.set_file(&item[4]);
        m.set_sys(sys);
        m.set_list(&item[5]);
        db.insert(&m).unwrap();
        // println!("{:?}", m);
        // session.insert(&m).unwrap();
    }
    // session.close();
}

fn select_dup() {
    let db = orm::open("root", "root", "172.16.16.224", 3306, "zod", orm_meta()).unwrap();
    let mut select = Select::<Mod>::new();
    select.wher(&Cond::by_eq("sys", "old"));
    select.join::<Mod>(JoinCond::by_eq("no", "no").eq("sys", "sys").lt("id", "id"));
    let res = db.query_ex(&select).unwrap();
    // println!("{:?}", res[0].len());
    for i in 0..res[0].len() - 1 {
        res[0][i].debug();
        res[1][i].debug();
        println!("");
    }
    println!("{:?}", res[0].len());
}

fn select_diff() {
    let db = orm::open("root", "root", "172.16.16.224", 3306, "zod", orm_meta()).unwrap();
    let mut select = Select::<Mod>::new();
    select.wher(&Cond::by_eq("sys", "new"));
    select.left_join::<Mod>(&JoinCond::by_eq("name", "name"))
        .on(&Cond::by_eq("sys", "old"))
        .wher(&Cond::by_is_null("id"));
    let res = db.query_ex(&select).unwrap();
    for item in res[0].iter() {
        item.debug();
    }

    // let mut select = Select::<Mod>::new();
    // select.wher(&Cond::by_eq("sys", "old"));
    // let old_res = db.query(&select).unwrap();
    // println!("old: {:?}", old_res.len());

    // let mut select = Select::<Mod>::new();
    // select.wher(&Cond::by_eq("sys", "new"));
    // let new_res = db.query(&select).unwrap();
    // println!("new: {:?}", new_res.len());

    // let new_map = new_res.iter()
    //     .map(|m| (m.get_name().to_string(), m))
    //     .collect::<HashMap<_, _>>();

    // let old_diff = old_res.iter()
    //     .filter(|m| !new_map.contains_key(&m.get_name()))
    //     .collect::<Vec<_>>();
    // println!("old_diff: {:?}", old_diff.len());

    // let old_map = old_res.iter()
    //     .map(|m| (m.get_name().to_string(), m))
    //     .collect::<HashMap<_, _>>();

    // let new_diff = new_res.iter()
    //     .filter(|m| !old_map.contains_key(&m.get_name()))
    //     .collect::<Vec<_>>();
    // println!("new_diff: {:?}", new_diff.len());



    // println!("{:?}", res[0].len());
    // for i in 0..res[0].len() - 1 {
    //     res[0][i].debug();
    //     println!("");
    // }
}

fn rebuild() {
    let db = orm::open("root", "root", "172.16.16.224", 3306, "zod", orm_meta()).unwrap();
    db.rebuild();
    insert_list("list", "new", &db);
    insert_list("list2", "old", &db);
}

fn main() {
    select_diff();
    // let mut counter = 0;
    // let mut map = HashMap::new();
    // for item in res.iter() {
    //     let name = item[0].clone();
    //     if map.contains_key(&name) {
    //         println!("{:?}", item);
    //         println!("{:?}", map.get(&name).unwrap());
    //         println!("");
    //         counter = counter + 1;
    //     }
    //     map.entry(name.clone()).or_insert(item);
    // }
    // println!("{:?}", res.len());
    // println!("{:?}", counter);
}
