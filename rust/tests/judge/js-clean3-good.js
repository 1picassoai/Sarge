import express from 'express';
import config from './config.json' with { type: 'json' };
import { DatabaseSync } from 'node:sqlite';

const db = new DatabaseSync(config.database);
const app = express();
app.disable('x-powered-by');
app.use(express.json());

app.post('/tools', (req, res) => {
  const name = (req.body.name || '').trim();
  if (!name) {
    return res.status(400).json({ error: 'name is required' });
  }
  const info = db.prepare('INSERT INTO tools (name, price) VALUES (?, ?)').run(name, req.body.price);
  res.status(201).json({ id: info.lastInsertRowid });
});

app.put('/tools/:id', (req, res) => {
  const result = db.prepare('UPDATE tools SET name = ? WHERE id = ?').run(req.body.name, req.params.id);
  if (result.changes === 0) {
    return res.status(404).json({ error: 'not found' });
  }
  res.status(200).json({ updated: true });
});

app.listen(process.env.PORT || 3000);
