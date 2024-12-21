use mlua::prelude::*;
use std::{io::{Read, Write}, net::{TcpListener, TcpStream}, vec};
use json::{self, JsonValue};

struct LuaTcpStream {
    stream: TcpStream,
}

struct LuaTcpListener {
    listener: TcpListener
}

struct LuaSockAddr {
    addr: std::net::SocketAddr
}

impl LuaUserData for LuaTcpStream {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_method_mut("read", |_, this,  _:() | {
            let mut buf = vec![0; 1024];
            this.stream.read(&mut buf).unwrap();
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

impl LuaUserData for LuaSockAddr {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("ip", |_, this, _: ()| {
            Ok(this.addr.ip().to_string())
        });
        methods.add_method("port", |_, this, _: ()| {
            Ok(this.addr.port())
        });
    }
}

impl LuaUserData for LuaTcpListener {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_method_mut("accept", |_, this, _: ()| {
            let (stream, addr) = this.listener.accept().unwrap();
            Ok((LuaTcpStream { stream }, LuaSockAddr { addr }))
        })
    }
}

fn jsonvalue_tolua(lua: &Lua, value: json::JsonValue) -> LuaResult<LuaValue> {
    match value {
        JsonValue::Null => Ok(LuaValue::Nil),
        JsonValue::Short(s) => Ok(LuaValue::String(lua.create_string(s.as_bytes())?)),
        JsonValue::String(s) => Ok(LuaValue::String(lua.create_string(&s)?)),
        JsonValue::Boolean(b) => Ok(LuaValue::Boolean(b)),
        JsonValue::Number(n) => Ok(LuaValue::Number(n.into())),
        JsonValue::Array(a) => {
            let table = lua.create_table()?;
            for (i, v) in a.iter().enumerate() {
                table.set(i + 1, jsonvalue_tolua(lua, v.clone())?)?;
            }
            Ok(LuaValue::Table(table))
        }
        json::JsonValue::Object(o) => {
            let table = lua.create_table()?;
            for (k, v) in o.iter() {
                table.set(k, jsonvalue_tolua(lua, v.clone())?)?;
            }
            Ok(LuaValue::Table(table))
        }
    }
}

fn parse_json(lua: &Lua, data: String) -> LuaResult<LuaValue> {
    Ok(jsonvalue_tolua(lua,json::parse(&data).unwrap()).unwrap())
}


fn bind_tcplistener(_lua: &Lua, address:String) -> LuaResult<LuaTcpListener> {
    let listener = TcpListener::bind(address).unwrap();
    Ok(LuaTcpListener { listener })
}

fn open_tcpstream(_lua: &Lua, address: String) -> LuaResult<LuaTcpStream> {
    let stream = TcpStream::connect(address)?;
    Ok(LuaTcpStream { stream })
}



#[mlua::lua_module]
fn lsock(lua: &Lua) -> LuaResult<LuaTable> {
    let exports = lua.create_table()?;
    exports.set("bind", lua.create_function(bind_tcplistener)?)?;
    exports.set("open", lua.create_function(open_tcpstream)?)?;
    exports.set("parse_json", lua.create_function(parse_json)?)?;
    Ok(exports)
}
