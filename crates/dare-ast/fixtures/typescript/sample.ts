import { Router } from 'express';

const router = Router();

router.get('/items', (_req, res) => {
  res.json([]);
});

export class Item {
  constructor(public id: string) {}
}

export interface ItemDto {
  id: string;
}
