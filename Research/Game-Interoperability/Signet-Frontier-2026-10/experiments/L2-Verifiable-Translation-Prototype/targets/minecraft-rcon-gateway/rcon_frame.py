#!/usr/bin/env python3
import struct
AUTH=3;AUTH_RESPONSE=2;EXEC_COMMAND=2;RESPONSE_VALUE=0
class RconError(ValueError): pass
def encode(packet_id,packet_type,body):
 payload=body.encode("ascii");size=8+len(payload)+2
 return struct.pack("<iii",size,packet_id,packet_type)+payload+b"\x00\x00"
def decode(frame):
 if len(frame)<14: raise RconError("frame too short")
 size,pid,ptype=struct.unpack("<iii",frame[:12])
 if size!=len(frame)-4: raise RconError("size mismatch")
 if frame[-2:]!=b"\x00\x00": raise RconError("invalid padding")
 return pid,ptype,frame[12:-2].decode("ascii")
