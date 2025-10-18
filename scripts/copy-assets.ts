import { copyFile, mkdir, writeFile } from 'fs/promises';
import { join } from 'path';
import { fileURLToPath } from 'url';
import { dirname } from 'path';
import { build } from 'esbuild';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const projectRoot = join(__dirname, '..');

async function copyAssets(): Promise<void> {
  try {
    // Ensure output directory exists
    await mkdir(join(projectRoot, 'static', 'js', 'dist'), { recursive: true });

    // Create entry points for Plotly and Arrow if they don't exist
    const plotlyEntry = join(projectRoot, 'src', 'assets', 'plotly-entry.ts');
    const arrowEntry = join(projectRoot, 'src', 'assets', 'arrow-entry.ts');

    await mkdir(join(projectRoot, 'src', 'assets'), { recursive: true });

    // Create Plotly entry point with tree-shaking
    await writeFile(plotlyEntry, `export * from 'plotly.js-dist-min';`);

    // Create Arrow entry point with tree-shaking
    await writeFile(arrowEntry, `export * from 'apache-arrow';`);

    console.log('🔨 Building optimized Plotly bundle...');
    await build({
      entryPoints: [plotlyEntry],
      bundle: true,
      minify: true,
      treeShaking: true,
      format: 'iife',
      globalName: '__PlotlyTemp',  // Use temp name
      outfile: join(projectRoot, 'static', 'js', 'dist', 'plotly.min.js'),
      external: [],
      target: 'es2015',
      logLevel: 'info',
      banner: { js: 'if (!window.Plotly) {' },  // Guard against double execution
      footer: { js: 'window.Plotly = __PlotlyTemp; }' }  // Assign and close guard
    });
    console.log('✅ Plotly bundle optimized');

    console.log('🔨 Building optimized Arrow bundle...');
    await build({
      entryPoints: [arrowEntry],
      bundle: true,
      minify: true,
      treeShaking: true,
      format: 'iife',
      globalName: '__ArrowTemp',  // Use temp name
      outfile: join(projectRoot, 'static', 'js', 'dist', 'arrow.min.js'),
      external: [],
      target: 'es2015',
      logLevel: 'info',
      banner: { js: 'if (!window.Arrow) {' },  // Guard against double execution
      footer: { js: 'window.Arrow = __ArrowTemp; }' }  // Assign and close guard
    });
    console.log('✅ Arrow bundle optimized');

    console.log('🔨 Building lazy-loader...');
    await build({
      entryPoints: [join(projectRoot, 'static', 'js', 'lazy-loader.ts')],
      bundle: false,
      minify: false,
      outfile: join(projectRoot, 'static', 'js', 'lazy-loader.js'),
      target: 'es2020',
      logLevel: 'info',
      platform: 'browser'
    });
    console.log('✅ Lazy-loader compiled');

    // Note: dist/sw.js is not needed - server serves directly from static/
    // Removed unnecessary copy operation

    console.log('🎉 All assets built and bundled successfully with tree-shaking!');
  } catch (error) {
    console.error('❌ Error building assets:', error);
    process.exit(1);
  }
}

copyAssets();
