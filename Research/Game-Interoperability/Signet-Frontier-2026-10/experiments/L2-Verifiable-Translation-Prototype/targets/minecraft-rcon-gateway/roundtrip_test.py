#!/usr/bin/env python3
import socket,threading
from rcon_frame import *
PASSWORD="research-only"
COMMANDS=["fill 0 0 0 1 0 1 stone","data get entity @p Pos","tp @e[tag=signet_bot,limit=1] 1 2 3","damage @p 1 generic","kill @e[tag=signet_dead,limit=1]"]
def recv_frame(s):
 h=b""
 while len(h)<4:h+=s.recv(4-len(h))
 n=int.from_bytes(h,"little",signed=True);r=b""
 while len(r)<n:r+=s.recv(n-len(r))
 return h+r
def server(s,seen):
 pid,t,b=decode(recv_frame(s));assert t==AUTH and b==PASSWORD;s.sendall(encode(pid,RESPONSE_VALUE,""));s.sendall(encode(pid,AUTH_RESPONSE,""))
 for _ in COMMANDS:
  pid,t,b=decode(recv_frame(s));assert t==EXEC_COMMAND;seen.append(b);s.sendall(encode(pid,RESPONSE_VALUE,"OK"))
def main():
 c,p=socket.socketpair();seen=[];th=threading.Thread(target=server,args=(p,seen));th.start();c.sendall(encode(1,AUTH,PASSWORD));assert decode(recv_frame(c))[1]==RESPONSE_VALUE;assert decode(recv_frame(c))[1]==AUTH_RESPONSE
 for i,cmd in enumerate(COMMANDS,10):c.sendall(encode(i,EXEC_COMMAND,cmd));rid,t,b=decode(recv_frame(c));assert (rid,t,b)==(i,RESPONSE_VALUE,"OK")
 th.join();assert seen==COMMANDS;print("source-rcon-framing: PASS");print("commands-roundtripped:",len(seen))
if __name__=="__main__":main()
