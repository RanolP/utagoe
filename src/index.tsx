import { setup } from 'goober';
import { prefix } from 'goober/prefixer';
import { render } from 'preact';
import { App } from './App';
import './index.css';

// Automatically resolved by vite.
declare const h: any;
setup(h, prefix);

const app = document.getElementById('app');

if (app) {
  render(<App />, app);
} else {
  document.write('Cannot find app element');
}
