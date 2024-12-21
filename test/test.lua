local lsock = assert(package.loadlib("./target/release/lsock.dll", "luaopen_lsock"))()
print(pcall(lsock.parse_json, '{"a": 1, "b": 2'))
print("apple")

print(pcall(lsock.open, "unwrap"))
print("banana")