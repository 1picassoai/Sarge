import express from 'express';
import sqlite3 from 'sqlite3';

const app = express();
app.use(express.json());

app.get('/tools', (req, res) => {
  const db = new sqlite3.Database('tools.db');
  db.all('SELECT * FROM tools', [], (err, rows) => {
    if (err) {
      return res.status(500).json({ error: err.message });
    }
    res.json(rows);
  });
});

app.get('/tools/:id', (req, res) => {
  const db = new sqlite3.Database('tools.db');
  db.get('SELECT * FROM tools WHERE id = ?', [req.params.id], (err, row) => {
    if (err) {
      return res.status(500).json({ error: err.message });
    }
    if (!row) {
      return res.status(404).json({ error: 'not found' });
    }
    res.json(row);
  });
});

app.post('/tools', (req, res) => {
  const db = new sqlite3.Database('tools.db');
  const { name, price } = req.body;
  db.run('INSERT INTO tools (name, price) VALUES (?, ?)', [name, price], function (err) {
    if (err) {
      return res.status(500).json({ error: err.message });
    }
    res.json({ id: this.lastID, name, price });
  });
});

app.listen(3000);
