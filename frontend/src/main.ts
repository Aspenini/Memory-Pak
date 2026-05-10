import { mount } from 'svelte';
import './styles.css';
import App from './App.svelte';

const target = document.getElementById('app');
if (!target) {
  throw new Error('Memory Pak: missing #app mount target');
}

const app = mount(App, { target });

if ('serviceWorker' in navigator && !window.__TAURI_INTERNALS__) {
  window.addEventListener('load', () => {
    navigator.serviceWorker.register('./service-worker.js').catch((error) => {
      console.warn('Memory Pak service worker registration failed', error);
    });
  });
}

export default app;
