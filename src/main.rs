use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::Path;

trait ReadContent{
    fn read_content(&mut self)->Vec<u8>;
}

impl ReadContent for File{
    fn read_content(&mut self)->Vec<u8>{
        let mut ret = Vec::new();
        self.read_to_end(&mut ret);
        ret
    }
}

struct ParserIter<'a> {
    pos: usize,
    arr: &'a [u8],
}

impl<'a> ParserIter<'a> {
    fn new(arr: &'a [u8]) -> Self {
        ParserIter { arr: arr, pos: 0 }
    }
    fn get_line(arr: &[u8]) -> (usize, usize, Vec<String>) {
        let find_flag = |&i: &usize| unsafe {
            if i + 2 >= arr.len() {
                return false;
            }
            let comp = std::mem::transmute::<[u8; 2], u16>([arr[i], arr[i + 1]]);
            return comp == 0x00 || comp == 0x0a0d;
            // return true;
        };
        let find_u3d = |&i: &usize| {
            return i + 8 < arr.len() && &arr[i..i + 8] == ".unity3d".as_bytes();
        };
        let u3d_pos = (0..arr.len()).find(find_u3d);
        if u3d_pos.is_none() {
            return (0, 0, Vec::new());
        }
        let u3d_pos = u3d_pos.unwrap();
        let start = (0..u3d_pos)
            .rev()
            .find(&find_flag)
            .unwrap() + 2;
        let end = (start..arr.len())
            .find(&find_flag)
            .unwrap();
        // println!("{:?}", (start, end));
        let splits = (start..end).fold(vec![Vec::new()], |mut acc, i| {
            if arr[i] == 0x09 {
                acc.push(Vec::new());
                // acc.push(i)
            } else {
                acc.last_mut().unwrap().push(arr[i]);
            }
            return acc;
        });
        let line = splits.into_iter()
            .map(|vec| String::from_utf8(vec).unwrap())
            .collect::<Vec<_>>();
        return (start, end, line);
    }
}

impl<'a> Iterator for ParserIter<'a> {
    type Item = Vec<String>;

    fn next(&mut self) -> Option<Vec<String>> {
        if self.pos >= self.arr.len() {
            return None;
        }
        let res = Self::get_line(&self.arr[self.pos..]);
        let end = res.1;
        if end == 0 {
            self.pos == self.arr.len();
            // self.arr = &self.arr[self.arr.len()..];
            return None;
        } else if res.2.len() > 10 {
            self.pos = self.pos + end;
            // self.arr = &self.arr[end..];
            return Some(res.2);
        } else {
            // println!("{:x}", self.pos);
            self.pos = self.pos + end;
            return self.next();
        }
    }
}

struct DirIter{

}


fn main() {
    let dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let path = Path::new(&dir).join("doal.unity3d");
    let mut file = File::open(path).unwrap();
    let mut vec = Vec::new();
    let len = file.read_to_end(&mut vec).unwrap();
    let arr = vec.as_slice();
    println!("{:?}", ParserIter::new(arr).collect::<Vec<_>>().len());
    for vec in ParserIter::new(arr) {
        println!("{:?}", vec);
    }
}
