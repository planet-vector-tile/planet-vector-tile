import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

/** @type {import('vite').UserConfig} */
export default defineConfig({
    base: process.env.IS_DEV !== 'true' ? './' : '/',
    plugins: [react()],
});
