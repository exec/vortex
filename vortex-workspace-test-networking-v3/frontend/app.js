// Simple Node.js frontend test
const express = require('express');
const axios = require('axios');
const app = express();

app.get('/', async (req, res) => {
  try {
    // Test connection to backend
    const response = await axios.get('http://localhost:8000/test');
    res.send(`<h1>Frontend Test</h1><p>Backend response: ${response.data}</p>`);
  } catch (error) {
    res.send(`<h1>Frontend Test</h1><p>Error connecting to backend: ${error.message}</p>`);
  }
});

app.listen(3000, () => {
  console.log('Frontend running on port 3000');
});