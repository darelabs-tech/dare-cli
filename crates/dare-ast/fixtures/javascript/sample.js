// fixtures/javascript/sample.js
const express = require('express');
const app = express();

app.get('/users', (req, res) => res.json([]));
app.post('/users', (req, res) => res.status(201).end());

class User {
  constructor(id) {
    this.id = id;
  }
}

module.exports = { app, User };
