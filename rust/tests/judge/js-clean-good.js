import express from 'express';
import { DatabaseSync } from 'node:sqlite';
import path from 'node:path';
import config from './config.json' with { type: 'json' };

const app = express();
app.use(express.json());
const db = new DatabaseSync(path.join(import.meta.dirname, config.database));
db.exec('CREATE TABLE IF NOT EXISTS tools (id INTEGER PRIMARY KEY, name TEXT, price REAL, is_archived INTEGER DEFAULT 0)');

app.get('/tools', (req, res) => {
  res.json(db.prepare('SELECT id, name, price FROM tools WHERE is_archived = 0').all());
});

app.get('/tools/:id', (req, res) => {
  const result = db.prepare('SELECT id, name, price FROM tools WHERE id = ? AND is_archived = 0').get(Number(req.params.id));
  if (!result) return res.status(404).json({ error: 'Tool not found' });
  res.json(result);
});

app.delete('/tools/:id', (req, res) => {
  const r = db.prepare('UPDATE tools SET is_archived = 1 WHERE id = ?').run(Number(req.params.id));
  if (r.changes === 0) return res.status(404).json({ error: 'Tool not found' });
  res.status(204).end();
});

app.listen(process.env.PORT || 3000);
