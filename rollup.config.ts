import { nodeResolve } from '@rollup/plugin-node-resolve';
import commonjs from '@rollup/plugin-commonjs';
import terser from '@rollup/plugin-terser';
import ignore from 'rollup-plugin-ignore';
import type { RollupOptions } from 'rollup';

const config: RollupOptions[] = [
  // Plotly.js bundle
  {
    input: 'src/assets/plotly-entry.ts',
    output: {
      file: 'static/js/dist/plotly.min.js',
      format: 'iife',
      name: '__PlotlyTemp',  // Temporary name to avoid conflicts
      sourcemap: false,
      banner: 'if (!window.Plotly) {',  // Guard: only execute if Plotly doesn't exist
      footer: 'window.Plotly = __PlotlyTemp; }',  // Assign to window.Plotly
      globals: {}
    },
    plugins: [
      ignore(['*.css', '*.scss', '*.scss', '*.sass']),
      nodeResolve({ browser: true, preferBuiltins: false }),
      commonjs(),
      terser({
        compress: {
          drop_console: true,
          drop_debugger: true
        },
        mangle: {
          reserved: ['Plotly', '__PlotlyTemp']
        }
      })
    ]
  },
  // Apache Arrow bundle
  {
    input: 'src/assets/arrow-entry.ts',
    output: {
      file: 'static/js/dist/arrow.min.js',
      format: 'iife',
      name: '__ArrowTemp',  // Temporary name to avoid conflicts
      sourcemap: false,
      banner: 'if (!window.Arrow) {',  // Guard: only execute if Arrow doesn't exist
      footer: 'window.Arrow = __ArrowTemp; }',  // Assign to window.Arrow
      globals: {}
    },
    plugins: [
      ignore(['*.css', '*.scss', '*.sass']),
      nodeResolve({ browser: true, preferBuiltins: false }),
      commonjs(),
      terser({
        compress: {
          drop_console: true,
          drop_debugger: true
        },
        mangle: {
          reserved: ['Arrow', '__ArrowTemp']
        }
      })
    ]
  }
];

export default config;
