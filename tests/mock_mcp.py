#!/usr/bin/env python3
"""Fault injection peer; never opens CAD or a network socket."""
import json,sys,time
mode=sys.argv[-1]
for line in sys.stdin:
    q=json.loads(line)
    if 'id' not in q: continue
    if mode=='timeout': time.sleep(3);continue
    if mode=='disconnect': sys.exit(0)
    if mode=='malformed': print('invalid',flush=True);continue
    r={'jsonrpc':'2.0','id':q['id']}
    if q['method']=='initialize':
        r['result']={'serverInfo':{'name':'solidcraft','version':'0.16.2' if mode=='version' else '0.17.0'},'protocolVersion':'2025-03-26'}
    elif mode=='tool_error':r['result']={'isError':True,'content':[{'type':'text','text':'secret-must-not-appear'}]}
    elif mode=='rpc_error':r['error']={'code':-32602,'message':'secret-must-not-appear'}
    else:r['result']={'content':[{'type':'text','text':'{"ok":true}'}]}
    if mode=='wrong_id':r['id']=999
    if mode=='both':r['error']={'code':-32600}
    print(json.dumps(r),flush=True)
