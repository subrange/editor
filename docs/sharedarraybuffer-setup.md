# SharedArrayBuffer Setup for Worker-Based Interpreter

The worker-based interpreter uses SharedArrayBuffer for efficient memory sharing between the main thread and the worker thread. This allows the interpreter to handle very large tapes (up to 150M cells) without copying data between threads.

## Requirements

SharedArrayBuffer requires specific security headers to be enabled:

### Development Server (Vite)

Add these headers to your `vite.config.ts`:

```typescript
export default {
  server: {
    headers: {
      'Cross-Origin-Opener-Policy': 'same-origin',
      'Cross-Origin-Embedder-Policy': 'require-corp'
    }
  }
}
```

### Production Server

For production, configure your web server to send these headers:

**Nginx:**
```nginx
add_header Cross-Origin-Opener-Policy "same-origin";
add_header Cross-Origin-Embedder-Policy "require-corp";
```

**Apache:**
```apache
Header set Cross-Origin-Opener-Policy "same-origin"
Header set Cross-Origin-Embedder-Policy "require-corp"
```

## Fallback Behavior

If SharedArrayBuffer is not available (e.g., on insecure contexts or without proper headers), the implementation will automatically fall back to regular ArrayBuffer. However, this will impact performance with very large tapes as data needs to be copied between threads.

## Testing

You can check if SharedArrayBuffer is available in your environment:

```javascript
console.log('SharedArrayBuffer available:', typeof SharedArrayBuffer !== 'undefined');
```

## Performance Benefits

With SharedArrayBuffer:
- No data copying between threads
- Instant state updates
- Can handle 150M+ cell tapes efficiently
- Real-time tape visualization during execution

Without SharedArrayBuffer:
- Falls back to message passing
- May experience delays with large tapes
- Limited by message size constraints