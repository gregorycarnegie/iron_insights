import { copyFile, mkdir, writeFile, readFile } from 'fs/promises';
import { join } from 'path';
import { createHash } from 'crypto';

async function copyAssets() {
  try {
    console.log('🏗️ Starting build process with Bun...');

    // Ensure output directory exists
    await mkdir('static/js/dist', { recursive: true });
    await mkdir('src/assets', { recursive: true });

    // 1. Build lazy-loader
    console.log('🔨 Building lazy-loader...');
    const lazyResult = await Bun.build({
      entrypoints: ['static/js/lazy-loader.ts'],
      outdir: 'static/js/dist',
      naming: 'lazy-loader.js',
      minify: true,
      target: 'browser',
    });
    if (!lazyResult.success) throw new Error(`Lazy loader build failed: ${lazyResult.logs}`);
    console.log('✅ Lazy-loader compiled');

    // 2. Copy pre-built Plotly
    console.log('📦 Copying pre-built Plotly bundle...');
    const plotlySource = 'node_modules/plotly.js-dist-min/plotly.min.js';
    const plotlyDest = 'static/js/dist/plotly.min.js';
    await copyFile(plotlySource, plotlyDest);
    console.log('✅ Plotly bundle copied');

    // 3. Copy pre-built Arrow
    console.log('📦 Copying pre-built Arrow bundle...');
    const arrowSource = 'node_modules/apache-arrow/Arrow.es2015.min.js';
    const arrowDest = 'static/js/dist/arrow.min.js';
    await copyFile(arrowSource, arrowDest);
    console.log('✅ Arrow bundle copied');

    // 4. Build Application Bundle
    console.log('🔨 Building application bundle...');
    const appFiles = [
      'init.js',
      'utils.js',
      'calculations.js',
      'websocket.js',
      'data.js',
      'charts.js',
      'ui.js',
      'main.js'
    ];

    // Create a temporary entry file
    const appEntryContent = appFiles.map(file => `import './app/${file}';`).join('\n');
    const appEntryPath = 'static/js/app-entry.js';
    await writeFile(appEntryPath, appEntryContent);

    const appResult = await Bun.build({
      entrypoints: [appEntryPath],
      outdir: 'static/js/dist',
      naming: 'app.js',
      minify: true,
      target: 'browser',
    });
    if (!appResult.success) throw new Error(`App build failed: ${appResult.logs}`);
    console.log('✅ Application bundle compiled');

    // 5. Generate content hashes and manifest
    console.log('🔐 Generating content hashes...');
    const manifest: Record<string, string> = {};

    const filesToHash = [
      { name: 'app.js', path: 'static/js/dist/app.js' },
      { name: 'lazy-loader.js', path: 'static/js/dist/lazy-loader.js' }
    ];

    for (const file of filesToHash) {
      const content = await readFile(file.path);
      const hash = createHash('md5').update(content).digest('hex').slice(0, 8);
      const hashedName = file.name.replace('.js', `.${hash}.js`);
      const hashedPath = `static/js/dist/${hashedName}`;

      await copyFile(file.path, hashedPath);
      manifest[file.name] = hashedName;
      console.log(`   ${file.name} -> ${hashedName}`);
    }

    // Write manifest
    await writeFile(
      'static/js/dist/manifest.json',
      JSON.stringify(manifest, null, 2)
    );
    console.log('✅ Manifest generated');

    console.log('🎉 All assets built and bundled successfully!');
  } catch (error) {
    console.error('❌ Error building assets:', error);
    process.exit(1);
  }
}

copyAssets();
