pub mod epc{
    use winapi::um::fileapi::{CreateFileA, ReadFile, WriteFile};
    use winapi::um::handleapi::INVALID_HANDLE_VALUE;
    use winapi::um::winnt::{FILE_SHARE_READ, FILE_SHARE_WRITE, GENERIC_READ, GENERIC_WRITE, HANDLE};
    use std::fs::File;
    use std::io::{Read, Write};
    use winapi::ctypes::c_void;
    use std::process::Command;
    use std::{env, thread, time};
    use std::sync::Arc;
    use winapi::um::winbase::{CreateNamedPipeA, PIPE_ACCESS_DUPLEX, PIPE_TYPE_BYTE, PIPE_TYPE_MESSAGE, PIPE_WAIT};
    use winapi::um::namedpipeapi::{ConnectNamedPipe};
    use std::thread::sleep;
    use std::time::Duration;
    use winapi::um::errhandlingapi::GetLastError;

    pub struct ServerPipe{
        name : String,
        handle : HANDLE,
        message_size : usize
    }

    pub struct ClientPipe{
        name : String,
        handle : HANDLE,
        message_size : usize
    }

    impl ClientPipe{
        pub fn new(name : String,message_size : usize) -> Option<Arc<Self>>{
            unsafe{
                let mut mypipe = format!("\\\\.\\pipe\\{}", name);
                let clientpipe = CreateFileA(mypipe.as_ptr() as * const i8,
                                             GENERIC_WRITE | GENERIC_READ,0,std::ptr::null_mut(),3,0,std::ptr::null_mut());
                match clientpipe{

                    INVALID_HANDLE_VALUE => {println!("CLIENT: Could not connect to pipe..."); None},
                    _ => Some(Arc::new(Self{name,handle : clientpipe,message_size}))




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
            while omessage.len()<self.message_size{
                omessage.push(' ');
            }
            unsafe{
                match WriteFile(self.handle, omessage.as_bytes().as_ptr() as *const c_void, self.message_size as u32, std::ptr::null_mut(), std::ptr::null_mut()){
                    0 => Err("Error sending message...".to_string()),
                    _ => Ok(())
                }
            }
        }

        pub fn receive(&self) -> Result<String,String>{
            unsafe{
                let mut inbuffer : [u8;4096] = [0;4096];
                match ReadFile(self.handle, inbuffer.as_mut_ptr() as *mut c_void, self.message_size as u32, std::ptr::null_mut(), std::ptr::null_mut()){
                    0 => Err("Error while reading message".to_string()),
                    _ => Ok(String::from_utf8_lossy(&inbuffer[0..self.message_size].trim_ascii()).to_string())
                }
            }
        }
    }

    impl ServerPipe {
        pub fn new(name: String,message_size : usize) -> Option<Arc<Self>> {
            unsafe {
                let mut mypipe = format!("\\\\.\\pipe\\{}", name);
                let serverpipe = CreateNamedPipeA(mypipe.as_ptr() as *const i8,
                                                  PIPE_ACCESS_DUPLEX,
                                                  PIPE_TYPE_BYTE,
                                                  1,
                                                  0,
                                                  0,
                                                  0,
                                                  std::ptr::null_mut());
                Some(Arc::new(Self { name, handle: serverpipe,message_size }))
            }
        }

        pub fn connect(&self) -> Result<(), String> {
            unsafe {
                match ConnectNamedPipe(self.handle, std::ptr::null_mut()){
                    0 => Err("Error during connect...".to_string()),
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
                    0 => Err(GetLastError().to_string()),
                    _ => Ok(())
                }
            }
        }

        pub fn receive(&self) -> Result<String,String>{
            unsafe{
                let mut inbuffer : [u8;4096] = [0;4096];
                match ReadFile(self.handle, inbuffer.as_mut_ptr() as *mut c_void, self.message_size as u32, std::ptr::null_mut(), std::ptr::null_mut()){
                    0 => Err("Error while reading message".to_string()),
                    _ => Ok(String::from_utf8_lossy(&inbuffer[0..self.message_size].trim_ascii()).to_string())
                }
            }
        }
    }
}