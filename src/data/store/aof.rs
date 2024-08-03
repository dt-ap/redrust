use std::{
    fs::File,
    io::{BufWriter, Write},
};

use crate::{common::Value, core::resp::encode};

use super::{Store, StoreObject};

impl Store {
    fn dump_key(&mut self, file: &mut File, key: String, store_value: StoreObject) {
        let cmd = format!("SET {0} {1}", key, store_value.value);
        let tokens = cmd.split_whitespace().map(str::to_string).collect();

        let _ = file.write_all(&encode(Value::VectorString(tokens), false));
    }

    pub fn dump_all_aof(&mut self) {
        let mut f = match File::create(self.config.aof_file.clone()) {
            Ok(res) => res,
            Err(err) => {
                println!("error {:?}", err);
                return;
            }
        };
        println!("rewriting AOF file at {0}", self.config.aof_file);

        let mut tuples: Vec<(String, StoreObject)> = Vec::new();
        for (k, sv) in self.inner.iter() {
            tuples.push((k.clone(), sv.clone()));
        }
        for (k, sv) in tuples {
            self.dump_key(&mut f, k, sv);
        }

        println!("AOF File rewrite complete");
    }
}
