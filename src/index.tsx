import { setup } from 'goober';
import { render } from 'preact';
import { App } from './app';
import './index.css';

// Automatically resolved by vite.
declare const h: any;
setup(h);

const app = document.getElementById('app');

if (app) {
  render(<App />, app);
} else {
  document.write('Cannot find app element');
}
