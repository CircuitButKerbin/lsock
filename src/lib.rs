use mlua::prelude::*;
use std::{io::{Read, Write}, net::TcpStream};

struct LuaTcpStream {
    stream: TcpStream,
}

impl LuaUserData for LuaTcpStream {
    fn add_fields<F: LuaUserDataFields<Self>>(fields: &mut F) {
        
    }
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_method_mut("read", |_, this,  _:() | {
            let mut buf = Vec::new();
            let n = this.stream.read_to_end(&mut buf).unwrap();
            Ok(String::from_utf8(buf).unwrap())
        });
        methods.add_method_mut("write", |_, this, data: String| {
            this.stream.write_all(data.as_bytes()).unwrap();
            this.stream.flush().unwrap();
            Ok(())
        });
        methods.add_method_mut("set_timeout", |_, this, timeout: u64| {
            this.stream.set_read_timeout(Some(std::time::Duration::from_secs(timeout))).unwrap();
            Ok(())
        });
        methods.add_method("close", |_, this, _: ()| {
            this.stream.shutdown(std::net::Shutdown::Both).unwrap();
            Ok(())
        });
    }
}

fn open_tcpstream(lua: &Lua, address: String) -> LuaResult<LuaTcpStream> {
    let stream = TcpStream::connect(address)?;
    Ok(LuaTcpStream { stream })
}



#[mlua::lua_module]
fn lsock(lua: &Lua) -> LuaResult<LuaTable> {
    let exports = lua.create_table()?;
    exports.set("hi", lua.create_function(open_tcpstream)?)?;
    Ok(exports)
}
