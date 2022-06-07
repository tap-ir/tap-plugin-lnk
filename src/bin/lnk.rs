//! Prefetch export windows prefetch file to json
extern crate tap_plugin_lnk;

use std::sync::Arc;
use std::env;
use std::fs::File;
use std::io::BufReader;

use tap_plugin_lnk::Lnk;
use tap::value::Value;

fn main() 
{
   if env::args().len() != 2 
   {
     println!("lnk input_file");
     return ;
   }

   let args: Vec<String> = env::args().collect();
   let file_path = &args[1];

   match File::open(file_path)
   {
      Err(_) => println!("Can't open file {}", file_path),
      Ok(file) => 
      {
         let mut buffered = BufReader::new(file);
         let lnk_parser = match Lnk::from_file(&mut buffered)
         {
           Ok(lnk_parser) => lnk_parser,
           Err(err) => {eprintln!("{}", err); return },
         };
      
         let value : Value = Value::ReflectStruct(Arc::new(lnk_parser));
         println!("{}", serde_json::to_string(&value).unwrap());
      },
   }
}
