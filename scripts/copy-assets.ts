import { copyFile, mkdir } from 'fs/promises';
import { join } from 'path';
import { fileURLToPath } from 'url';
import { dirname } from 'path';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const projectRoot = join(__dirname, '..');

async function copyAssets(): Promise<void> {
  try {
    // Ensure output directory exists
    await mkdir(join(projectRoot, 'static', 'js', 'dist'), { recursive: true });

    // Copy Plotly.js
    await copyFile(
      join(projectRoot, 'node_modules', 'plotly.js-dist-min', 'plotly.min.js'),
      join(projectRoot, 'static', 'js', 'dist', 'plotly.min.js')
    );
    console.log('✅ Copied plotly.min.js');

    // Copy Apache Arrow
    await copyFile(
      join(projectRoot, 'node_modules', 'apache-arrow', 'Arrow.es2015.min.js'),
      join(projectRoot, 'static', 'js', 'dist', 'arrow.min.js')
    );
    console.log('✅ Copied arrow.min.js');

    console.log('🎉 All assets copied successfully!');
  } catch (error) {
    console.error('❌ Error copying assets:', error);
    process.exit(1);
  }
}

copyAssets();
