use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use std::env;

use nix::unistd::Pid;

use mem_utils::MemMap;

pub mod mem_utils;


fn main() {
    
    let args: Vec<String> = env::args().collect();
    println!("args: {:?}", args);

    let _exe_name;

    if args.len() > 1 {
        _exe_name = args[1].clone();

        //open galculator 
        //type in 123456789
        //run 'sudo cargo run -- galculator' with the function below enabled
        
        //update_extern_process(_exe_name, "123456789", "calc hack");
        
        //resize the galculator window so it gets redrawn
        
    } else {
        _exe_name = std::env::current_exe()
        .expect("can't get current_exe")
        .file_name()
        .expect("can't get filename")
        .to_string_lossy().into_owned();

        current_exe_example(_exe_name);
        
        
    }
    
}

#[allow(unused)]
fn update_extern_process(extern_exe: String, _find: &str, _replace: &str) {
    
    let find = _find.as_bytes();
    let replace =  _replace.as_bytes();

    let pids =MemMap::get_pids_for_program(&extern_exe).expect("error getting pids");
    println!("{:?}", pids);
    let pid = Pid::from_raw(pids[0]);

    let _map = MemMap::read_pid_maps(pids[0]).unwrap();
    let _filtered_map: Vec<_> = _map.iter()
        //.filter(|entry| !entry.mapping.starts_with("/usr/lib"))
        .filter(|entry| entry.modes.contains(&String::from("w")) && entry.mapping.starts_with("[heap]"))
        .collect();

    for mem_map in _filtered_map {
        // println!("{:?}", mem_map);
        let addr_start = mem_map.range.0;
        let vsize = mem_map.size;
        
       
        let scan_result = MemMap::scan_mem(pid, addr_start, vsize, find);
        for r in scan_result {
            println!("in {}", mem_map.mapping);
            println!("found {:?} at addr: {:x}", String::from_utf8_lossy(find), r);
            println!("try to replace at this address..");
            let write_result = MemMap::write_vm(pid, r, replace.len(), Vec::from(replace)).unwrap();
            println!("{} bytes written..", write_result);
        }
       
        
        
    }

}
#[allow(unused)]
fn current_exe_example(current_exe: String) {
    #[derive(Debug)]
    struct Inspect {
        _number: u32,
        _string: String,
        _boxed_string: Box<String>, //non-sense double heap allocation
        _vec_of_string: Vec<String>,
        _mutex_string: Arc<Mutex<String>>,
    }
    
    let inspect = Inspect {
        _number: 12,
        _string: String::from("ABCDE"),
        _boxed_string: Box::new(String::from("ABCDE")),
        _vec_of_string: vec!["vec".into(), "of".into(), "ABCDE".into()],
        _mutex_string: Arc::new(Mutex::new(String::from("ABCDE"))),
    };

    //using prinln macro might result in more heap allocation with the same strings 
    println!("{:?}", inspect);
   
    let lock = inspect._mutex_string.lock().unwrap();

    let _find = "ABCDE";
    let find = _find.as_bytes();

    let _replace = "EDCBA";
    let replace =  _replace.as_bytes();

    let pids =MemMap::get_pids_for_program(&current_exe).expect("error getting pids");
    //println!("{:?}", pids);
    let pid = Pid::from_raw(pids[0]);

    let _map = MemMap::read_pid_maps(pids[0]).unwrap();
    //println!("maps: {:#?}", _map);
    let _filtered_map: Vec<_> = _map.iter()
        //.filter(|entry| entry.modes.contains(&String::from("w")) && !entry.mapping.starts_with("/usr/lib"))
        .filter(|entry| entry.mapping.starts_with("[heap]"))
        .collect();

    for mem_map in _filtered_map {
        println!("{:?}", mem_map);
        let addr_start = mem_map.range.0;
        let vsize = mem_map.size;
        
       
        let scan_result = MemMap::scan_mem(pid, addr_start, vsize, find);
        for r in scan_result {
            println!("found {:?} at addr: {:x}", String::from_utf8_lossy(find), r);
            println!("try to replace at this address..");
            let write_result = MemMap::write_vm(pid, r, replace.len(), Vec::from(replace)).unwrap();
            println!("{} bytes written..", write_result);
        }
       
        
        
    }
    drop(lock);
    println!("Inspect struct is now: {:?}",inspect);

    //loop {}
    

}
