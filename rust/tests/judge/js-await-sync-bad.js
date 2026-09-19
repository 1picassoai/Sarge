import express from 'express';
import { DatabaseSync } from 'node:sqlite';
import path from 'node:path';
import config from './config.json' with { type: 'json' };

const app = express();
app.use(express.json());
const db = new DatabaseSync(path.join(import.meta.dirname, config.database));

app.get('/tools/:id', async (req, res) => {
  const result = await db.prepare('SELECT id, name, price FROM tools WHERE id = ?').get(Number(req.params.id));
  if (!result) return res.status(404).json({ error: 'Tool not found' });
  res.json(result);
});

app.listen(process.env.PORT || 3000);
