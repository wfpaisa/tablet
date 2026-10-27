// Capture supplementary Settings states from the unmodified HTML frontend.
import {spawn} from 'node:child_process';
import {mkdir,writeFile} from 'node:fs/promises';
const output='target/settings-reference';await mkdir(output,{recursive:true});
const server=spawn('python3',['-m','http.server','18765','--bind','127.0.0.1','--directory','/home/projects/ia/wacom/app/ui'],{stdio:'ignore'});
const browser=spawn('chromium',['--headless=new','--disable-gpu','--no-sandbox','--hide-scrollbars','--force-dark-mode','--remote-debugging-port=19222','--remote-debugging-address=127.0.0.1','--user-data-dir='+process.cwd()+'/target/settings-chromium','about:blank'],{stdio:'ignore'});
let ws;
try {
 let tabs;for(let i=0;i<100;i++){try{tabs=await(await fetch('http://127.0.0.1:19222/json')).json();if(tabs.length)break;}catch{} await new Promise(r=>setTimeout(r,50));}
 ws=new WebSocket(tabs[0].webSocketDebuggerUrl);await new Promise(r=>ws.addEventListener('open',r,{once:true}));
 let id=0;const pending=new Map();ws.addEventListener('message',e=>{const m=JSON.parse(e.data);if(m.method==='Runtime.exceptionThrown')console.error(JSON.stringify(m.params));if(m.id){const p=pending.get(m.id);pending.delete(m.id);m.error?p.reject(m.error):p.resolve(m.result);}});
 const send=(method,params={})=>new Promise((resolve,reject)=>{const n=++id;pending.set(n,{resolve,reject});ws.send(JSON.stringify({id:n,method,params}));});
 const evaluate=async expression=>{const r=await send('Runtime.evaluate',{expression,awaitPromise:true,returnByValue:true});if(r.exceptionDetails)throw r.exceptionDetails;return r.result.value;};
 await send('Page.enable');await send('Runtime.enable');await send('Emulation.setEmulatedMedia',{features:[{name:'prefers-color-scheme',value:'dark'}]});
 for(const scenario of ['language-open','toast','reset-confirmation','delete-confirmation','create-dialog','general-es','general-ja']) {
  for(const height of scenario.startsWith('general-')?[720,1600]:[720]) {
   await send('Emulation.setDeviceMetricsOverride',{width:1100,height,deviceScaleFactor:1,mobile:false});
   await send('Page.navigate',{url:'http://127.0.0.1:18765/index.html#/settings/'+(scenario.includes('confirmation')||scenario==='create-dialog'?'backups':'general')});
   await new Promise(r=>setTimeout(r,300));
   await evaluate(`localStorage.setItem('language',${JSON.stringify(scenario==='general-es'?'es':scenario==='general-ja'?'ja':'en')});location.reload()`);
   for(let i=0;i<100;i++){if(await evaluate('typeof openDropdown === "function" && !!document.querySelector(".tabs")'))break;await new Promise(r=>setTimeout(r,50));}await evaluate('document.fonts.ready.then(()=>true)');
   const expression={
    'language-open':`openDropdown(document.querySelector('#language'))`,
    'toast':`toast('Driver restarted')`,
    'reset-confirmation':`void settingsDialog('Reset all settings to their defaults?', 'Reset all settings')`,
    'delete-confirmation':`void settingsDialog('Delete this backup? This cannot be undone.', 'Delete')`,
    'create-dialog':`void settingsDialog('New backup','Create',true)`,
   }[scenario];
   if(expression){await evaluate(expression);}
   await new Promise(r=>setTimeout(r,100));
   const {data}=await send('Page.captureScreenshot',{format:'png',captureBeyondViewport:false});
   const name=`classic-dark-settings-${scenario}${height===1600?'-tall':''}`;
   await writeFile(`${output}/${name}.png`,Buffer.from(data,'base64'));console.log(name);
  }
 }
} finally {ws?.close();browser.kill();server.kill();}
