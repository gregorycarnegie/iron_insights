
async function test() {
    try {
        console.log('Testing Bun.build...');
        const result = await Bun.build({
            entrypoints: ['static/js/lazy-loader.ts'],
            outdir: 'static/js/dist',
            naming: 'lazy-loader-bun.js',
            minify: true,
            target: 'browser',
        });

        if (!result.success) {
            console.error('Build failed:', result.logs);
        } else {
            console.log('Success!');
        }
    } catch (e) {
        console.error('Error:', e);
    }
}

test();
