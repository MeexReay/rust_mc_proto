udata = "fn read_uTYPE_varint(&mut self) -> Result<uTYPE, ProtocolError> { read_varint!(uTYPE, self) }\n"
idata = "fn read_iTYPE_varint(&mut self) -> Result<iTYPE, ProtocolError> { Ok(self.read_uTYPE_varint()?.zigzag()) }\n"

types = ["size", "8", "16", "32", "64", "128"]

all_data = ""

for t in types:
    all_data += udata.replace("TYPE", t)
    
print(all_data)

all_data = ""
    
for t in types:
    all_data += idata.replace("TYPE", t)
    
print(all_data)