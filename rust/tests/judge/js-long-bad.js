import express from 'express';
import sqlite3 from 'sqlite3';
const app = express();
app.use(express.json());

// helper block to push the faults deep into the file
function pad1() { return 1; }
function pad2() { return 2; }
function pad3() { return 3; }
function pad4() { return 4; }
function pad5() { return 5; }
function pad6() { return 6; }
function pad7() { return 7; }
function pad8() { return 8; }
function pad9() { return 9; }
function pad10() { return 10; }
function pad11() { return 11; }
function pad12() { return 12; }
function pad13() { return 13; }
function pad14() { return 14; }
function pad15() { return 15; }
function pad16() { return 16; }
function pad17() { return 17; }
function pad18() { return 18; }
function pad19() { return 19; }
function pad20() { return 20; }

app.get('/tools', (req, res) => {
  const db = new sqlite3.Database(
    './tools.db'
  );
  db.all(
    'SELECT * FROM tools',
    [],
    (err, rows) => {
      if (err) {
        return res.status(500).json({ error: err.message, stack: err.stack });
      }
      res.json(rows);
    }
  );
});

function pad21() { return 21; }
function pad22() { return 22; }
function pad23() { return 23; }
function pad24() { return 24; }

app.listen(3000);
