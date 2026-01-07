import { cpSync, existsSync, mkdirSync } from 'fs';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const root = join(__dirname, '..');

const src = join(root, 'src-tauri', 'android-assets');
const dest = join(root, 'src-tauri', 'gen', 'android', 'app', 'src', 'main', 'res');

if (existsSync(src)) {
  if (!existsSync(dest)) {
    mkdirSync(dest, { recursive: true });
  }
  cpSync(src, dest, { recursive: true });
  console.log('Android assets copied successfully');
} else {
  console.warn('Android assets source directory not found:', src);
}
