// Entry point for Apache Arrow bundle
import * as Arrow from 'apache-arrow';

declare global {
  interface Window {
    Arrow: typeof Arrow;
  }
}

window.Arrow = Arrow;
export default Arrow;
