import express from 'express';
import { readFile } from 'node:fs/promises';

const app = express();

app.get('/report', async (req, res) => {
  res.setHeader('Content-Type', 'text/csv');
  res.send(await readFile('/var/data/huge-report.csv'));
});

export default app;
