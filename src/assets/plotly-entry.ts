// Entry point for Plotly.js bundle
import Plotly from 'plotly.js-dist-min';

declare global {
  interface Window {
    Plotly: typeof Plotly;
  }
}

window.Plotly = Plotly;
export default Plotly;
