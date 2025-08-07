# Important: Restart Vite Server

The vite.config.ts has been updated with the necessary headers for SharedArrayBuffer support:

```
Cross-Origin-Opener-Policy: same-origin
Cross-Origin-Embedder-Policy: require-corp
```

**You need to restart your Vite development server for these changes to take effect.**

1. Stop the current server (Ctrl+C or Cmd+C)
2. Run `npm run dev` again

After restarting, SharedArrayBuffer should be available and the worker-based interpreter will work correctly.