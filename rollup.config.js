import { nodeResolve } from '@rollup/plugin-node-resolve';
import commonjs from '@rollup/plugin-commonjs';
import terser from '@rollup/plugin-terser';
import ignore from 'rollup-plugin-ignore';

export default [
  // Plotly.js bundle
  {
    input: 'src/assets/plotly-entry.js',
    output: {
      file: 'static/js/dist/plotly.min.js',
      format: 'iife',
      name: 'Plotly',
      sourcemap: false
    },
    plugins: [
      ignore(['*.css', '*.scss', '*.sass']),
      nodeResolve({ browser: true, preferBuiltins: false }),
      commonjs(),
      terser({
        compress: {
          drop_console: true,
          drop_debugger: true
        }
      })
    ]
  },
  // Apache Arrow bundle
  {
    input: 'src/assets/arrow-entry.js',
    output: {
      file: 'static/js/dist/arrow.min.js',
      format: 'iife',
      name: 'Arrow',
      sourcemap: false
    },
    plugins: [
      ignore(['*.css', '*.scss', '*.sass']),
      nodeResolve({ browser: true, preferBuiltins: false }),
      commonjs(),
      terser({
        compress: {
          drop_console: true,
          drop_debugger: true
        }
      })
    ]
  }
];