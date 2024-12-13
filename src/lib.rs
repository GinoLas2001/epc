




    #[cfg(target_os="windows")]
    use winapi::um::fileapi::{CreateFileA, ReadFile, WriteFile};
    #[cfg(target_os="windows")]
    use winapi::um::handleapi::INVALID_HANDLE_VALUE;
    #[cfg(target_os="windows")]
    use winapi::um::winnt::{FILE_SHARE_READ, FILE_SHARE_WRITE, GENERIC_READ, GENERIC_WRITE, HANDLE};
    #[cfg(target_os="windows")]
    use std::fs::File;
    #[cfg(target_os="windows")]
    use std::io::{Read, Write};
    #[cfg(target_os="windows")]
    use winapi::ctypes::c_void;
    #[cfg(target_os="windows")]
    use std::process::Command;
    #[cfg(target_os="windows")]
    use std::{env, thread, time};
    #[cfg(target_os="windows")]
    use std::sync::Arc;
    #[cfg(target_os="windows")]
    use winapi::um::winbase::{CreateNamedPipeA, CallNamedPipeA,PIPE_ACCESS_DUPLEX, PIPE_NOWAIT, PIPE_TYPE_BYTE, PIPE_TYPE_MESSAGE, PIPE_WAIT};
    #[cfg(target_os="windows")]
    use winapi::um::namedpipeapi::{ConnectNamedPipe,DisconnectNamedPipe};
    #[cfg(target_os="windows")]
    use std::thread::sleep;
    #[cfg(target_os="windows")]
    use std::time::Duration;
    #[cfg(target_os="windows")]
    use winapi::um::errhandlingapi::GetLastError;

    #[cfg(target_os="windows")]
    pub struct ServerPipe{
        name : String,
        handle : HANDLE,
        message_size : usize
    }
    #[cfg(target_os="windows")]
    pub struct ClientPipe{
        name : String,
        handle : HANDLE,
        message_size : usize
    }
    #[cfg(target_os="windows")]
    impl ClientPipe{
        pub fn new(name : String,message_size : usize) -> Result<Arc<Self>,String>{
            unsafe{
                let mypipe = format!("\\\\.\\pipe\\{}\0", name);
                let clientpipe = CreateFileA(mypipe.as_ptr() as * const i8,
                                             GENERIC_WRITE | GENERIC_READ,0,std::ptr::null_mut(),3,0,std::ptr::null_mut());
                match clientpipe{

                    INVALID_HANDLE_VALUE => {Err(GetLastError().to_string())},
                    _ => {println!("Creating client pipe");Ok(Arc::new(Self{name,handle : clientpipe,message_size}))}
                }
            }
        }

        pub fn send(&self, message : String) -> Result<(),String>{
            if message.len() > self.message_size{
            }
            let mut omessage = message.to_owned();
            omessage.push('\0');
            if omessage.len()>self.message_size{
                return Err("Message is too big!".to_string())
            }
            while omessage.len()<self.message_size{
                omessage.push(' ');
            }
            unsafe{
                match WriteFile(self.handle, omessage.as_bytes().as_ptr() as *const c_void, self.message_size as u32, std::ptr::null_mut(), std::ptr::null_mut()){
                    0 => Err(GetLastError().to_string()),
                    _ => Ok(())
                }
            }
        }

        pub fn receive(&self) -> Result<String,String>{
            unsafe{
                let mut inbuffer : [u8;4096] = [0;4096];
                match ReadFile(self.handle, inbuffer.as_mut_ptr() as *mut c_void, self.message_size as u32, std::ptr::null_mut(), std::ptr::null_mut()){
                    0 => Err(GetLastError().to_string()),
                    _ => {
                            let mut result = String::from_utf8_lossy(&inbuffer[0..self.message_size].trim_ascii()).to_string();
                            result.pop();
                            Ok(result)
                        }
                }
            }
        }
    }

    #[cfg(target_os="windows")]
    impl ServerPipe {
        pub fn new(name: String,message_size : usize) -> Result<Arc<Self>,String> {
            unsafe {
                let mypipe_s = format!("\\\\.\\pipe\\{}\0", name);
                let mypipe = mypipe_s.as_bytes();
                let serverpipe = CreateNamedPipeA(mypipe.as_ptr() as *const i8,
                                                  PIPE_ACCESS_DUPLEX,
                                                  PIPE_TYPE_MESSAGE ,
                                                  1,
                                                  0,
                                                  0,
                                                  0,
                                                  std::ptr::null_mut());
                match serverpipe{
                    INVALID_HANDLE_VALUE => Err(format!("Could not open server:{}",GetLastError().to_string())),
                    _ => {println!("Created server pipe");Ok(Arc::new(Self { name, handle: serverpipe,message_size }))}

                }
            }
        }

        pub fn connect(&self) -> Result<(), String> {
            unsafe {
                match ConnectNamedPipe(self.handle, std::ptr::null_mut()){
                    0 => Err(format!("Could not connect to the pipe:{})",GetLastError().to_string())),
                    _ => {
                        Ok(())
                    }
                }
            }
        }

        pub fn disconnect(&self) -> Result<(),String>{
            unsafe{
                match DisconnectNamedPipe(self.handle){
                    0 => Err(format!("Error during disconnection:{}",GetLastError().to_string())),
                    _ => Ok(())
                }
            }
        }

        pub fn send(&self, message : String) -> Result<(),String>{
            if(message.len() > self.message_size){
            }
            let mut omessage = message.to_owned();
            omessage.push('\0');
            if(omessage.len()>self.message_size){
                return Err("Message is too big!".to_string())
            }
            while omessage.len()<self.message_size {
                omessage.push(' ');
            }
            unsafe{
                match WriteFile(self.handle, omessage.as_bytes().as_ptr() as *const c_void, self.message_size as u32, std::ptr::null_mut(), std::ptr::null_mut()){
                    0 => Err(format!("Error sending message: {}",GetLastError().to_string())),
                    _ => Ok(())
                }
            }
        }

        pub fn receive(&self) -> Result<String,String>{
            unsafe{
                let mut inbuffer : [u8;4096] = [0;4096];
                match ReadFile(self.handle, inbuffer.as_mut_ptr() as *mut c_void, self.message_size as u32, std::ptr::null_mut(), std::ptr::null_mut()){
                    0 => Err(format!("Error while reading:{}",GetLastError())),
                    _ => {
                        let mut res = String::from_utf8_lossy(&inbuffer[0..self.message_size]).trim_ascii_end().to_string();
                            let _ = res.pop();
                            return Ok(res);
                    }
                }
            }
        }
    }

    impl Drop for ClientPipe{
        fn drop(&mut self) {
            println!("Dropping client pipe...");
        }
    }

    
    impl Drop for ServerPipe{
        fn drop(&mut self) {
            #[cfg(target_os = "windows")]
            let _ = self.disconnect();
            #[cfg(target_os = "linux")]
            self.close();
            println!("Dropping server pipe...");
        }
    }






    #[cfg(target_os="linux")]
    use libc::{c_int, mkfifo, mode_t, EACCES, EEXIST, ENOENT, O_RDONLY, O_WRONLY, S_IWUSR,S_IRUSR};
    #[cfg(target_os="linux")]
    use std::ffi::CString;
    #[cfg(target_os="linux")]
    use std::fs;
    #[cfg(target_os="linux")]
    use std::io::{self, Error};
    #[cfg(target_os="linux")]
    use std::os::raw::c_void;
    #[cfg(target_os="linux")]
    use std::os::unix::fs::OpenOptionsExt;
    #[cfg(target_os="linux")]
    use std::path::Path;
    #[cfg(target_os="linux")]
    pub struct ServerPipe{
        name : String,
        message_size : usize,
        path_cs : String,
        path_sc : String
    }

    #[cfg(target_os="linux")]
    pub struct ClientPipe{
        name : String, 
        message_size : usize,
        path_cs : String,
        path_sc : String
    }
    #[cfg(target_os="linux")]
    impl ServerPipe{
        pub fn new(name : String,message_size : usize)->Result<Self,String>{
            let path_sc = format!("/tmp/{}_sc.tmp",name);
            let path_cs = format!("/tmp/{}_cs.tmp",name);
            let res_cs : c_int = unsafe{mkfifo(path_cs.as_ptr() as *mut i8, S_IWUSR | S_IRUSR as mode_t)};
            let res_sc : c_int = unsafe{mkfifo(path_sc.as_ptr() as *mut i8, S_IWUSR | S_IRUSR as mode_t)};
            let result_sc : i32 = res_sc.into();
            let result_cs : i32 = res_cs.into();

            if result_sc == 0 && result_cs == 0{
                return Ok(Self{name : name.clone(),message_size,path_cs,path_sc});
            }

            let error = errno::errno();
            match error.0{
                EACCES => {
                    return Err(format!("Could not open pipe : {}",error));
                },
                EEXIST => {
                    return Err(format!("Could not open pipe : {}",error));
                },
                ENOENT => {
                    return Err(format!("Could not open pipe : {}",error));
                },
                _ => {
                    return Err(format!("Could not open pipe : {}",error));
                }
            }

        }

        pub fn close(&self){
            let res = match fs::remove_file(format!("/tmp/{}_cs.tmp",self.name)){
                Ok(_) => Ok(()),
                Err(e) => Err(e.to_string()),
            };
            if let Err(e) = res{
                println!("Error deleting pipe:{e}");
            }
            let res = match fs::remove_file(format!("/tmp/{}_sc.tmp",self.name)){
                Ok(_) => Ok(()),
                Err(e) => Err(e.to_string()),
            };
            if let Err(e) = res{
                println!("Error deleting pipe:{e}");
            }
        }

        pub fn connect(&self) -> Result<(),String>{
            Ok(())
        }
        #[cfg(target_os="linux")]
        pub fn disconnect(&self) -> Result<(),String>{
            Ok(())
        }
        #[cfg(target_os="linux")]
        pub fn receive(&self) -> Result<String,String>{
            unsafe {
                let fd : c_int = libc::open(self.path_cs.as_ptr() as * mut i8, O_RDONLY);
                match fd {
                    -1 => return Err(format!("Could not connect: {}",errno::errno())),
                    _ => {
                        let mut read_buffer:[u8;4096] = [0;4096];
                        let res = libc::read(fd,read_buffer.as_mut_ptr() as *mut c_void,self.message_size);
                        if res == self.message_size as isize{
                            let mut res = String::from_utf8_lossy(&read_buffer[0..self.message_size]).trim_ascii_end().to_string();
                            let _ = res.pop();
                            return Ok(res);
                        }
                        else{
                            return Err(format!("Error while reading"));
                        }

                    }
                }
            }
        }

        pub fn send(&self,message : String) -> Result<(),String>{
            if message.len() > self.message_size{
            }
            let mut omessage = message.to_owned();
            omessage.push('\0');
            if omessage.len()>self.message_size{
                return Err("Message is too big!".to_string())
            }
            while omessage.len()<self.message_size{
                omessage.push(' ');
            }
            unsafe {
                let fd : c_int = libc::open(self.path_sc.as_ptr() as * mut i8, O_WRONLY);
                match fd {
                    -1 => return Err(format!("Could not connect: {}",errno::errno())),
                    _ => {
                        let res = libc::write(fd,omessage.as_mut_ptr() as *mut c_void,self.message_size);
                        if res == self.message_size as isize{
                            return Ok(());
                        }
                        else{
                            return Err(format!("Error while writing"));
                        }

                    }
                }
            }
        }

        
    }
    #[cfg(target_os="linux")]
    impl ClientPipe{
        pub fn new(name : String, message_size : usize) -> Result<Self,String>{
            return Ok(Self{name : name.clone(),message_size,path_cs : format!("/tmp/{}_cs.tmp",name),path_sc : format!("/tmp/{}_sc.tmp",name) });
        }

        pub fn send(&self,message : String) -> Result<(),String>{
            if message.len() > self.message_size{
            }
            let mut omessage = message.to_owned();
            omessage.push('\0');
            if omessage.len()>self.message_size{
                return Err("Message is too big!".to_string())
            }
            while omessage.len()<self.message_size{
                omessage.push(' ');
            }
            unsafe {
                let fd : c_int = libc::open(self.path_cs.as_ptr() as * mut i8, O_WRONLY);
                match fd {
                    -1 => return Err(format!("Could not connect: {}",errno::errno())),
                    _ => {
                        let res = libc::write(fd,omessage.as_mut_ptr() as *mut c_void,self.message_size);
                        if res == self.message_size as isize{
                            return Ok(());
                        }
                        else{
                            return Err(format!("Error while writing"));
                        }

                    }
                }
            }
        }

        pub fn receive(&self) -> Result<String,String>{
            unsafe {
                let fd : c_int = libc::open(self.path_sc.as_ptr() as * mut i8, O_RDONLY);
                match fd {
                    -1 => return Err(format!("Could not connect: {}",errno::errno())),
                    _ => {
                        let mut read_buffer:[u8;4096] = [0;4096];
                        let res = libc::read(fd,read_buffer.as_mut_ptr() as *mut c_void,self.message_size);
                        if res == self.message_size as isize{
                            let mut res = String::from_utf8_lossy(&read_buffer[0..self.message_size]).trim_ascii_end().to_string();
                            let _ = res.pop();
                            return Ok(res);
                        }
                        else{
                            return Err(format!("Error while reading"));
                        }

                    }
                }
            }
        }
    }

