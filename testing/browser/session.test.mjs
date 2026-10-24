import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtemp, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { withFixtureSession } from './session.mjs';

async function fixture(body, run, overrides = {}) {
  const root = await mkdtemp(join(tmpdir(), 'testkit-browser-'));
  const readyPath = join(root, 'ready.json'), logPath = join(root, 'fixture.log');
  try {
    return await withFixtureSession({command:process.execPath,
      args:['--input-type=module', '-e', `import {writeFileSync} from 'node:fs'; ${body}`],
      env:{...process.env, READY:readyPath}, readyPath, logPath,
      startupMs:2000, stopMs:100, pollMs:10, ...overrides}, run);
  } finally { await rm(root, { recursive:true, force:true }); }
}
const ready = "writeFileSync(process.env.READY, JSON.stringify({pid:process.pid}));";
test('ready session returns result and gracefully reaps child', async () => {
  let pid;
  assert.equal(await fixture(`${ready} process.stdin.once('data',()=>process.exit(0));`, async value => {pid=value.pid; return 42;}), 42);
  assert.throws(()=>process.kill(pid, 0), {code:'ESRCH'});
});
test('body failure survives forced shutdown of a SIGTERM-ignoring child', async () => {
  let pid;
  await assert.rejects(fixture(`process.on('SIGTERM',()=>{}); ${ready} setInterval(()=>{},100);`, value=>{pid=value.pid; throw new Error('body failure');}), /body failure/);
  assert.throws(()=>process.kill(pid, 0), {code:'ESRCH'});
});
test('startup exit, timeout and spawn errors refuse before callback', async () => {
  const unexpected=()=>assert.fail('must not enter body');
  await assert.rejects(fixture('process.exit(7);', unexpected), /before readiness/);
  await assert.rejects(fixture('setInterval(()=>{},100);', unexpected, {startupMs:100}), /timeout/);
  await assert.rejects(fixture('', unexpected, {command:'/missing/testkit-executable'}), {code:'ENOENT'});
});
test('partial readiness JSON is retried', async () => {
  await fixture(`writeFileSync(process.env.READY,'{'); setTimeout(()=>{${ready}},50); console.log('fixture log'); process.stdin.once('data',()=>process.exit());`, async value=>assert.ok(value.pid));
});

test('falsy body exceptions are preserved', async () => {
  let caught = false;
  try { await fixture(ready + " process.stdin.once('data',()=>process.exit());", () => { throw undefined; }); }
  catch (error) { caught = true; assert.equal(error, undefined); }
  assert.ok(caught);
});

test('nonzero fixture exit after readiness refuses successful and failed bodies', async () => {
  for (const bodyFails of [false, true]) {
    const bodyError = new Error('independent body failure');
    const session = fixture(`process.on('SIGUSR1',()=>process.exit(23)); ${ready} setInterval(()=>{},100);`, async ({pid}) => {
      process.kill(pid, 'SIGUSR1');
      const deadline = performance.now() + 2000;
      for (;;) {
        try { process.kill(pid, 0); }
        catch (error) { if (error.code === 'ESRCH') break; throw error; }
        assert.ok(performance.now() < deadline, 'fixture must exit');
        await new Promise(resolve => setTimeout(resolve, 10));
      }
      if (bodyFails) throw bodyError;
      return 42;
    });
    await assert.rejects(session, error => {
      if (bodyFails) {
        assert.ok(error instanceof AggregateError);
        assert.equal(error.errors[0], bodyError);
        assert.match(error.errors[1].message, /fixture exited unsuccessfully \(23\)/);
      } else {
        assert.match(error.message, /fixture exited unsuccessfully \(23\)/);
      }
      return true;
    });
  }
});

test('nonzero exit during requested graceful shutdown is retained', async () => {
  await assert.rejects(fixture(`${ready} process.stdin.once('data',()=>process.exit(17));`, () => 42), /fixture exited unsuccessfully \(17\)/);
});
