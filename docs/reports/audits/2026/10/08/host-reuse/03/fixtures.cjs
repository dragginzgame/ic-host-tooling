const fs = require('fs');
const root = __dirname;
const leb = value => { const out=[]; do {let b=value&127; value >>>=7; out.push(b|(value?128:0));} while(value); return Buffer.from(out); };
const section=(id, bytes)=>Buffer.concat([Buffer.from([id]),leb(bytes.length),bytes]);
const header=Buffer.from([0,97,115,109,1,0,0,0]);
const type=section(1,Buffer.from([1,96,0,0]));
const wasmModule=(imports,count)=>Buffer.concat([header,type,section(2,Buffer.from(imports)),section(3,Buffer.concat([leb(count),Buffer.alloc(count)])),section(10,Buffer.concat([leb(count),Buffer.from(Array.from({length:count},()=>[2,0,11]).flat())]))]);
const fixtures={
  'imported-function-limit': wasmModule([1,1,109,1,102,0,0],50000),
  'global-before-function': wasmModule([2,1,109,1,103,3,127,0,1,109,1,102,0,0],1),
  'truncated-code': Buffer.concat([header,Buffer.from([10,5,1])]),
};
for(const [name,bytes] of Object.entries(fixtures)) {
  const path=`${root}/${name}.wasm`; fs.writeFileSync(path,bytes);
}
