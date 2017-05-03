extern crate glob;
use glob::glob;

use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::error::Error;
use std::collections::HashMap;


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
    let res = File::open(path);
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
        let vec = vec.into_iter().map(|s| String::from_utf8(s).unwrap()).collect::<Vec<_>>();
        if &vec[3] != "abdata"{
            // println!("{:?}", vec);
            // unreachable!();
        }
        acc.push(vec);
        return acc;
    });
    return res;
}

fn main() {
    let dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let path = Path::new(&dir).join("list/characustom/*.unity3d");
    let res = glob(path.to_str().unwrap()).unwrap().into_iter().flat_map(|entry| {
        return parse_file(entry.unwrap());
    }).collect::<Vec<_>>();
    println!("{:?}", res.len());
    let mut map = HashMap::new();
    for item in res{
        let name = item[0].clone();
        if map.contains_key(&name){
            println!("{:?}", item);
            println!("{:?}", map.get(&name).unwrap());
            println!("");
        }
        map.entry(name.clone()).or_insert(item);
        // println!("{}", item[2]);
    }
    // let res = parse_file(path);
    // for item in res {
    //     println!("{:?}", item);
    // }
}
