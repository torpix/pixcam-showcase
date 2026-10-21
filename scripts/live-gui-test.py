"""Open only our synthetic specimen in a separate, temporary SolidCraft profile.

Control authentication stays inside SolidCraft/CLI and is never read or printed.
"""
import hashlib,json,os,re,shutil,signal,socket,subprocess,sys,tempfile,time,urllib.request,urllib.error
from pathlib import Path

APP=os.environ.get('PIXCAM_SOLIDCRAFT_APP') or shutil.which('solidcraft')
if not APP:
    raise SystemExit('Set PIXCAM_SOLIDCRAFT_APP to the SolidCraft 0.17.0 GUI executable (or put solidcraft on PATH)')
PLAYWRIGHT=os.environ.get('PIXCAM_PLAYWRIGHT','playwright')
specimen=Path('fixtures/bridge-part.solidcraft')
before=hashlib.sha256(specimen.read_bytes()).hexdigest()
Path('test-results').mkdir(exist_ok=True)
with tempfile.TemporaryDirectory(prefix='pixcam-synthetic-gui-') as directory:
    env=os.environ.copy()
    env.update(XDG_DATA_HOME=directory+'/data',XDG_CONFIG_HOME=directory+'/config',
               SOLIDCRAFT_CONTROL_DIR=directory+'/control',APPIMAGE_EXTRACT_AND_RUN='1')
    with socket.socket() as reserve:
        reserve.bind(('127.0.0.1',0));port=reserve.getsockname()[1]
    app=subprocess.Popen([APP,'--control',str(port),str(Path('fixtures/bridge-part.solidcraft').resolve())],env=env,
                         stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL,start_new_session=True)
    try:
        ready=False
        for _ in range(100):
            if app.poll() is not None:break
            try:
                with socket.create_connection(('127.0.0.1',port),timeout=.2):ready=True;break
            except OSError:time.sleep(.2)
        if not ready:raise RuntimeError('Isolated SolidCraft GUI failed to start its localhost control endpoint')
        env['SOLIDCRAFT_TEST_CONNECT']='127.0.0.1:'+str(port)
        raw=subprocess.check_output(['target/debug/pixcam-showcase','capture-solidcraft','--connect',env['SOLIDCRAFT_TEST_CONNECT']],env=env,timeout=65)
        snapshot=json.loads(raw)
        assert snapshot['inspect_design']['name']=='bridge-part','Unexpected document; refusing further capture'
        Path('test-results/gui-snapshot.json').write_bytes(raw)
        subprocess.run(['node','scripts/bridge-browser-test.mjs',PLAYWRIGHT],env=env,check=True,timeout=100)
        if '--record' in sys.argv:
            subprocess.run(['node','scripts/record-bridge.mjs',PLAYWRIGHT],env=env,check=True,timeout=100)
        # Prove actual CAD loss and recovery while the adapter keeps running.
        adapter=subprocess.Popen(['target/debug/pixcam-showcase','bridge','--connect',env['SOLIDCRAFT_TEST_CONNECT']],env=env,
                                 stdout=subprocess.PIPE,stderr=subprocess.DEVNULL,text=True)
        try:
            url=re.search(r'http://127\.0\.0\.1:\d+/',adapter.stdout.readline()).group(0)
            html=urllib.request.urlopen(url,timeout=5).read().decode()
            page_session=re.search(r"const session='([0-9a-f]{64})'",html).group(1)
            def refresh():
                request=urllib.request.Request(url+'api/refresh',data=b'{}',method='POST',
                    headers={'Origin':url.rstrip('/'),'Content-Type':'application/json','X-Pixcam-Session':page_session})
                return json.loads(urllib.request.urlopen(request,timeout=40).read())
            assert len(refresh()['features'])==5
            os.killpg(app.pid,signal.SIGTERM);app.wait(timeout=8)
            try:refresh()
            except urllib.error.HTTPError as error:
                assert error.code==502
                assert json.loads(error.read())['stale'] is True
            else:raise AssertionError('Disconnected CAD unexpectedly passed')
            app=subprocess.Popen([APP,'--control',str(port),str(specimen.resolve())],env=env,
                stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL,start_new_session=True)
            for _ in range(100):
                try:
                    with socket.create_connection(('127.0.0.1',port),timeout=.2):break
                except OSError:time.sleep(.2)
            assert len(refresh()['features'])==5,'CAD restart failed to recover'
            print('PASS: actual CAD app exit → HTTP 502/stale → app restart → fresh MCP capture.')
        finally:
            adapter.terminate();adapter.wait(timeout=5)
        with socket.socket() as reserve:
            reserve.bind(('127.0.0.1',0));unused_port=reserve.getsockname()[1]
        absent=subprocess.run(['target/debug/pixcam-showcase','capture-solidcraft','--connect','127.0.0.1:'+str(unused_port),'--timeout-ms','1000'],
                              env=env,stdout=subprocess.PIPE,stderr=subprocess.PIPE,timeout=8)
        assert absent.returncode!=0 and not absent.stdout,'Absent app/wrong port must fail closed'
        assert hashlib.sha256(specimen.read_bytes()).hexdigest()==before,'Synthetic CAD file changed'
        Path('test-results/gui-verification.json').write_text(json.dumps({'passed':True,'solidcraft_version':'0.17.0',
            'document':'bridge-part','transport':'stdio MCP --connect / localhost','isolated_profile':True,
            'active_user_document_modified':False,'cad_disconnect_reconnect':True,'absent_app_wrong_port_rejected':True},indent=2))
        print('PASS: real isolated SolidCraft GUI → MCP → Rust registry → Chromium CONNECTED; no CAD writes.')
    finally:
        if app.poll() is None:
            os.killpg(app.pid,signal.SIGTERM)
            try:app.wait(timeout=8)
            except subprocess.TimeoutExpired:os.killpg(app.pid,signal.SIGKILL);app.wait(timeout=5)
