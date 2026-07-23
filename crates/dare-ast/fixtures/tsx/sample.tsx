import React from 'react';

export function ItemsPage() {
  fetch.get?.('/'); // not a route
  return <div />;
}

// Nest-style decorator in TSX file context
class ItemsController {
  // simulated
}

// express-style for extractor
const app = {
  get(path: string, _h: unknown) {
    return path;
  },
};
app.get('/tsx-items', () => null);

export class Catalog {}
