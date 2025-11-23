
import { build } from 'esbuild';
import { join } from 'path';

async function test() {
    try {
        console.log('Testing esbuild...');
        await build({
            entryPoints: ['static/js/lazy-loader.ts'],
            bundle: false,
            minify: true,
            outfile: 'static/js/dist/lazy-loader-test.js',
            target: 'es2020',
            logLevel: 'info',
            platform: 'browser'
        });
        console.log('Success!');
    } catch (e) {
        console.error('Error:', e);
    }
}

test();
