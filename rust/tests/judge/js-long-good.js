import express from 'express';
import config from './config.json' with { type: 'json' };
import { DatabaseSync } from 'node:sqlite';
const db = new DatabaseSync(config.database);
const app = express();
app.disable('x-powered-by');
app.use(express.json());

app.get('/tools/1', (req, res) => {
  const rows = db.prepare('SELECT id, name, price FROM tools WHERE cat = ?').all(1);
  res.json(rows);
});

app.get('/tools/2', (req, res) => {
  const rows = db.prepare('SELECT id, name, price FROM tools WHERE cat = ?').all(2);
  res.json(rows);
});

app.get('/tools/3', (req, res) => {
  const rows = db.prepare('SELECT id, name, price FROM tools WHERE cat = ?').all(3);
  res.json(rows);
});

app.get('/tools/4', (req, res) => {
  const rows = db.prepare('SELECT id, name, price FROM tools WHERE cat = ?').all(4);
  res.json(rows);
});

app.get('/tools/5', (req, res) => {
  const rows = db.prepare('SELECT id, name, price FROM tools WHERE cat = ?').all(5);
  res.json(rows);
});

app.get('/tools/6', (req, res) => {
  const rows = db.prepare('SELECT id, name, price FROM tools WHERE cat = ?').all(6);
  res.json(rows);
});

app.get('/tools/7', (req, res) => {
  const rows = db.prepare('SELECT id, name, price FROM tools WHERE cat = ?').all(7);
  res.json(rows);
});

app.get('/tools/8', (req, res) => {
  const rows = db.prepare('SELECT id, name, price FROM tools WHERE cat = ?').all(8);
  res.json(rows);
});

app.get('/tools/9', (req, res) => {
  const rows = db.prepare('SELECT id, name, price FROM tools WHERE cat = ?').all(9);
  res.json(rows);
});

app.get('/tools/10', (req, res) => {
  const rows = db.prepare('SELECT id, name, price FROM tools WHERE cat = ?').all(10);
  res.json(rows);
});

app.get('/tools/11', (req, res) => {
  const rows = db.prepare('SELECT id, name, price FROM tools WHERE cat = ?').all(11);
  res.json(rows);
});

app.get('/tools/12', (req, res) => {
  const rows = db.prepare('SELECT id, name, price FROM tools WHERE cat = ?').all(12);
  res.json(rows);
});

app.listen(process.env.PORT || 3000);
