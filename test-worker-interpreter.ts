// Test file for worker-based interpreter
// NOTE: SharedArrayBuffer requires HTTPS or specific headers:
// - Cross-Origin-Opener-Policy: same-origin
// - Cross-Origin-Embedder-Policy: require-corp
// 
// If SharedArrayBuffer is not available, the implementation will fall back
// to regular ArrayBuffer with message passing.

import { interpreterStore } from './src/components/debugger/interpreter-facade.store';

async function testTurboMode() {
  console.log('Testing turbo mode with worker...');
  console.log('SharedArrayBuffer available:', typeof SharedArrayBuffer !== 'undefined');
  
  // Wait for initialization
  await new Promise(resolve => setTimeout(resolve, 1000));
  
  // Run a simple test program
  console.log('Running turbo mode test...');
  await interpreterStore.runTurbo();
  
  const state = interpreterStore.state.getValue();
  console.log('Test completed. Check console for any errors.');
}

testTurboMode().catch(console.error);