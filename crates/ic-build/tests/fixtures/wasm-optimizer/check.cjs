const assert = require('node:assert/strict');
const fs = require('node:fs');
for (const path of process.argv.slice(2)) {
  const module = new WebAssembly.Module(fs.readFileSync(path));
  assert.deepEqual(WebAssembly.Module.imports(module), [
    { module: 'ic0', name: 'msg_reply_data_append', kind: 'function' },
    { module: 'ic0', name: 'msg_reply', kind: 'function' },
  ]);
  assert.deepEqual(WebAssembly.Module.exports(module).map(e => e.name).sort(),
    ['canister_query answer', 'compute', 'memory']);
  const payload = [];
  let replies = 0;
  const { exports } = new WebAssembly.Instance(module, { ic0: {
    msg_reply_data_append(offset, length) {
      payload.push(...new Uint8Array(exports.memory.buffer, offset, length));
    },
    msg_reply() { replies++; },
  } });
  for (const x of [-2147483648, -100, -1, 0, 1, 100, 1073741824, 2147483647]) {
    assert.equal(exports.compute(x), x < 0 ? (x + 1) | 0 : x === 0 ? 42 : (x * 2) | 0);
  }
  exports['canister_query answer']();
  assert.equal(replies, 1);
  assert.deepEqual(payload, [42, 0, 0, 0]);
  console.log(`Wasm execution passed: ${path}`);
}
