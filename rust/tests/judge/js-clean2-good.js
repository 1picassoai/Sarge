import express from 'express';
import config from './config.json' with { type: 'json' };
import { DatabaseSync } from 'node:sqlite';

const db = new DatabaseSync(config.database);
const app = express();
app.disable('x-powered-by');
app.use(express.json());

app.use((req, res, next) => {
  res.setHeader('Access-Control-Allow-Origin', config.allowedOrigin);
  next();
});

function authenticate(req, res, next) {
  if (!req.headers.authorization) {
    return res.status(401).end();
  }
  if (!req.user || !req.user.canEdit) {
    return res.status(403).end();
  }
  next();
}

app.get('/report', async (req, res) => {
  try {
    const [tools, categories] = await Promise.all([
      fetchTools(),
      fetchCategories()
    ]);
    res.setHeader('Content-Type', 'text/csv');
    res.send(toCsv(tools, categories));
  } catch (err) {
    console.error(err);
    res.status(500).json({ error: 'could not build the report' });
  }
});

app.delete('/tools/:id', authenticate, (req, res) => {
  db.prepare('UPDATE tools SET is_archived = 1 WHERE id = ?').run(req.params.id);
  res.status(204).end();
});

app.listen(process.env.PORT || 3000);
