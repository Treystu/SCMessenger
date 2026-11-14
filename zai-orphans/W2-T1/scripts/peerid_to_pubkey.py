import sys
A="123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz"
def b58d(s):
    n=0
    for c in s:
        n=n*58+A.index(c)
    b=n.to_bytes((n.bit_length()+7)//8,'big')
    pad=len(s)-len(s.lstrip('1'))
    return b'\x00'*pad+b
def pubkey_hex(peer_id):
    raw=b58d(peer_id)
    # multihash: <code><len><digest>; identity multihash is code 0x00
    if raw[0]!=0x00:
        return None  # hashed multihash (non-Ed25519) - key not recoverable
    digest=raw[2:2+raw[1]]
    # protobuf PublicKey: field1 varint type (1=Ed25519), field2 bytes key
    i=0
    keytype=None; key=None
    while i<len(digest):
        tag=digest[i]; i+=1
        if tag==0x08:
            keytype=digest[i]; i+=1
        elif tag==0x12:
            ln=digest[i]; i+=1
            key=digest[i:i+ln]; i+=ln
        else:
            break
    if keytype!=1 or not key or len(key)!=32:
        return None
    return key.hex()
if __name__=="__main__":
    for pid in sys.argv[1:]:
        print(pid, pubkey_hex(pid))
