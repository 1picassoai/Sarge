import express from 'express';
import { DatabaseSync } from 'node:sqlite';
import path from 'node:path';

const app = express();
app.use(express.json());
const db = new DatabaseSync(path.join(import.meta.dirname, 'tools.db'));
db.exec('CREATE TABLE IF NOT EXISTS tools (id INTEGER PRIMARY KEY, name TEXT, price REAL)');

app.get('/tools', (req, res) => {
  res.json(db.prepare('SELECT id, name, price FROM tools').all());
});

app.listen(process.env.PORT || 3000);
