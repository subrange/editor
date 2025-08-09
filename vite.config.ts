import {defineConfig} from 'vite'
import react from '@vitejs/plugin-react'
import tailwindcss from '@tailwindcss/vite'
import basicSsl from '@vitejs/plugin-basic-ssl'
import wasm from 'vite-plugin-wasm'
import topLevelAwait from 'vite-plugin-top-level-await'

// https://vite.dev/config/
export default defineConfig({
    plugins: [
        wasm(),
        topLevelAwait(),
        tailwindcss(), 
        react(),
        basicSsl()
    ],
    server: {
        headers: {
            'Cross-Origin-Opener-Policy': 'same-origin',
            'Cross-Origin-Embedder-Policy': 'require-corp',
            'Cross-Origin-Resource-Policy': 'cross-origin',
        },
        watch: {
            ignored: [
                '**/src/bf-macro-expander/**',
                '**/src/rust-bf/**',
                '**/src/ripple-asm/**',
                '**/c-code/**',
                '**/rcc-ir/**',
                '**/rcc-frontend/**',
                '**/rcc-driver/**',
                '**/rcc-common/**',
                '**/rcc-codegen/**',
                '*.asm',
                '*.bf',
                '*.asm',
                '*.pobj'
            ]
        }
    },
    worker: {
        format: 'es'
    },
    assetsInclude: ["**/*.bfm"],
})
