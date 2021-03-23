use std::{collections::HashMap, sync::MutexGuard};

struct MapValue {
  a: u32
}

struct Config {
  map: HashMap<String, MapValue>
}

struct Data {
  value: u32,
  config: Config,
}

fn another(data: &mut Data) {
  data.value = 2;

  for (k, v) in &mut data.config.map {
    v.a = 5;
  }
}

fn run(mut data: MutexGuard<Data>) {
  data.value = 1;
  another(&mut data);
  data.value = 3;
}

#[cfg(test)]
mod tests {
  use std::sync::{Arc, Mutex};

  use super::*;

  #[test]
  fn test() {
    let mut map = HashMap::new();
    map.insert("Hello".to_string(), MapValue { a: 0});
    let data = Arc::new(Mutex::new(Data { value: 0, config: Config { map: map } }));

    let rs = Arc::clone(&data);
    run(rs.lock().unwrap());

    println!("Data.value is {}", data.lock().unwrap().value);
    println!("Map value is {:?}", data.lock().unwrap().config.map.get("Hello").unwrap().a);
  }
}
