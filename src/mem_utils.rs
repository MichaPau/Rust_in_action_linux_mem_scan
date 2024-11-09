use std::{fs, io::{self, IoSlice, IoSliceMut}, path::PathBuf, process::Command};

use nix::{sys::uio::{process_vm_readv, process_vm_writev, RemoteIoVec}, unistd::Pid};

#[derive(Debug)]
pub enum MemMapError {
    IoError(io::Error),
    ParseIntError(std::num::ParseIntError),
    Errno(nix::errno::Errno),
    OtherError(String),

}

impl From<io::Error> for MemMapError {
    fn from(err: io::Error) -> Self {
        MemMapError::IoError(err)
    }
}

impl From<std::num::ParseIntError> for MemMapError {
    fn from(err: std::num::ParseIntError) -> Self {
        MemMapError::ParseIntError(err)
    }
}

impl From<nix::errno::Errno> for MemMapError {
    fn from(err: nix::errno::Errno) -> Self {
        MemMapError::Errno(err)
    }
}
#[derive(Debug)]
pub struct MemMap {
    pub range: (usize, usize),
    pub size: usize,
    pub modes: Vec<String>,
    pub offset: usize,
    pub inode_id: i32,
    pub versions: (String, String),
    pub mapping: String,

}

impl MemMap {
    pub fn read_pid_maps(pid: i32) -> Result<Vec<MemMap>, MemMapError> {
        let path = PathBuf::from(format!("/proc/{}/maps", pid).as_str());
    
        let content = fs::read_to_string(path)?;
    
        let mut pmap: Vec<MemMap> = vec![];
    
        for line in content.lines() {
            
            //println!("{}", line);
            let s: Vec<&str> = line.split_whitespace().collect();

            if s.len() < 5 {
                return Err(MemMapError::OtherError("Error while parsing - not enough entries found.".into()));
            }
            let range: Vec<&str> = s[0].split('-').collect();
    
            let start = usize::from_str_radix(range[0], 16)?;
            let end = usize::from_str_radix(range[1], 16)?;
    
            let modes: Vec<String> = s[1].chars().filter_map(|c| {
                match c {
                    '-' => None,
                    _ => Some(String::from(c)),
                }
            }).collect();

            let offset = usize::from_str_radix(s[2], 16)?;

            let versions: (String, String) = match s[3].split_once(':') {
                Some((v1, v2)) => (String::from(v1), String::from(v2)),
                None => ("".into(), "".into()),
            };

            //let inode_id = i32::from_str_radix(s[4], 10)?;
            let inode_id = s[4].parse::<i32>()?;
            let mapping = match s.len() {
                n if n >= 6 => String::from(s[5]),
                _ => "".into()
            };
    
            pmap.push(
                MemMap {
                    range: (start, end),
                    modes,
                    offset,
                    versions,
                    inode_id,
                    mapping,
                    size: end - start,
                }
            );
    
            // println!("{:?}-{:?}", start, end);
        }
    
        Ok(pmap)
    }

    pub fn get_pids_for_program(name: &str) -> Result<Vec<i32>, MemMapError> {
        let output = Command::new("pidof")
        .args([name])
        .output()?;

        
        let mut output_slice = output.stdout.as_slice();
        
        if !output_slice.is_empty() && output_slice[&output_slice.len() -1] == 10 {
            output_slice = &output_slice[0..output_slice.len() -1];
        }

        let pids:Vec<&[u8]> = output_slice.split(|u8_code| *u8_code == 32).collect();

        let pids_i32 = pids.into_iter().filter_map(|p_u8| String::from_utf8(p_u8.to_vec()).ok()).filter_map(|p_string| p_string.parse::<i32>().ok()).collect();
        Ok(pids_i32)
    }

    pub fn scan_mem(pid: Pid, addr: usize, vsize: usize, buf: &[u8]) -> Vec<usize>{
        
        let mut res = vec![0_u8; vsize];
        let mut local_iov = [IoSliceMut::new(&mut res)];
       
        let remote_iov = RemoteIoVec { base: addr, len: vsize};
       
        let _ = process_vm_readv(pid, &mut local_iov, &[remote_iov]);
       
        let f_locale = &local_iov[0];
    
        let mut pos = 0;
        let r: Vec<_> = f_locale.windows(buf.len()).filter_map(|slice| {
           
            pos += 1;
            if slice == buf {
                Some(addr+pos-1)
            } else {
                None
            }
        }).collect();
    
       r
        
    }

    pub fn read_vm(pid: Pid, addr: usize, vsize: usize) -> Result<Vec<u8>, MemMapError>{
  
        let mut res = vec![0_u8; vsize];
        let mut local_iov = [IoSliceMut::new(&mut res)];
        let remote_iov = RemoteIoVec { base: addr, len: vsize};
       
        let _ = process_vm_readv(pid, &mut local_iov, &[remote_iov])?;
    
        let temp = local_iov.first().unwrap();
            
        let t2 = temp.to_vec();
        
        Ok(t2)      
    }
    pub fn write_vm(pid: Pid, addr: usize, vsize: usize, buf: Vec<u8>) -> Result<usize, MemMapError> {
        
         let local_iov = &[IoSlice::new(buf.as_slice())];
         let remote_iov = RemoteIoVec { base: addr, len: vsize};
         let result = process_vm_writev(pid, local_iov, &[remote_iov])?;
     
         Ok(result)
     }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, process::Command};

    use crate::mem_utils::MemMap;

    

    #[test]
    fn test_parsing() {
        let cmd = Command::new("ls")
           .arg("-a")
           .spawn()
           .expect("failed to execute command");

        let pid = cmd.id();
        println!("pid: {}", pid);

        let map = MemMap::read_pid_maps(pid as i32).unwrap();
        assert!(map.len() > 0);
        
        let mut _hmap:HashMap<String, Vec<MemMap>> = HashMap::new();
        for item in map.into_iter() {
            // _hmap.entry(item.mapping.clone()).and_modify(|list| list.push(item)).or_insert(vec![item]);
            _hmap.entry(item.mapping.clone()).or_insert_with(Vec::new).push(item);
        }
        println!("hmap: {:#?}", _hmap);
    }

    #[test]
    fn test_get_pids() {
        let pids1 = MemMap::get_pids_for_program("non_existant");
        assert_eq!(pids1.unwrap().len(), 0);

        let pids2 = MemMap::get_pids_for_program("firefox").unwrap();
        assert!(pids2.len() > 0);

        println!("firefox: {:?}", pids2);


    }
}